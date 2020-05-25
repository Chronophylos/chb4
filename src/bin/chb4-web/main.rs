#[macro_use]
extern crate log;
use anyhow::{ensure, Context, Result};
use chb4::{actions, commands, manpages};
use config::{Config, Environment, File, FileFormat};
use flexi_logger::Logger;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Method, Request, Response, Server, StatusCode,
};
use lru::LruCache;
use std::{
    convert::TryInto,
    io::prelude::*,
    net::IpAddr,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

fn convert_asciidoc(text: String) -> Result<String> {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("GIT_HASH");

    let asciidoctor = Command::new("asciidoctor")
        .arg("-a")
        .arg(format!("pkg_version={}", version))
        .arg("-a")
        .arg(format!("git_hash={}", git_hash))
        .arg("-o")
        .arg("-")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    asciidoctor
        .stdin
        .context("Stdin is missing")?
        .write_all(text.as_bytes())?;

    let mut buf = String::new();
    asciidoctor
        .stdout
        .context("Stdout is missing")?
        .read_to_string(&mut buf)?;

    Ok(buf)
}

fn handle_conn(
    manpage_index: Arc<manpages::Index>,
    cache: Arc<Mutex<LruCache<String, String>>>,
    request: Request<Body>,
) -> Result<Response<Body>, Box<dyn std::error::Error>> {
    if request.method() != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("This Method is not allowed!"))?);
    }

    let path = String::from(request.uri().path());
    let splitted: Vec<&str> = path.splitn(3, "/").collect();

    if splitted.len() != 3 {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Malformed path"))?);
    }

    let chapter = splitted[1];
    let pagename = splitted[2];

    if chapter.is_empty() || pagename.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Malformed path"))?);
    }

    let key = format!("{}/{}", chapter, pagename);

    {
        let mut cache = cache.lock().unwrap();
        if let Some(html) = cache.get(&key) {
            debug!("Cache hit! (key: {})", key);
            return Ok(Response::new(Body::from(html.to_owned())));
        }
    }

    debug!("Cache miss! Generating {}", key);

    let chapter = manpages::ChapterName::from(chapter.to_owned());

    let resp = {
        match manpage_index.whatis(Some(chapter.clone()), pagename) {
            Some(page) => {
                debug!("Rendering page {}", page);
                let rendered_page = page.render()?;

                debug!("Converting page to html5");
                let html = convert_asciidoc(rendered_page)?;

                {
                    let mut cache = cache.lock().unwrap();
                    cache.put(format!("{}/{}", chapter, pagename), html.clone());
                }

                Response::new(Body::from(html))
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Page not found"))?,
        }
    };

    Ok(resp)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn check_asciidoctor() -> Result<bool> {
    Ok(Command::new("asciidoctor")
        .arg("--version")
        .stdout(Stdio::null())
        .status()?
        .success())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // Create logger with custom format (`chb4::format`)
    Logger::with_env_or_str("chb4=trace, rustls=info, debug")
        .format(chb4::format)
        .start()?;

    ensure!(check_asciidoctor()?, "Missing dependency `asciidoctor`");

    // Get crate version and git hash from environment.
    // Both env vars are set in `build.rs`.
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = env!("GIT_HASH");

    info!("Starting CHB4 Webserver {} ({})", version, git_hash);

    // Load config
    let mut config = Config::new();
    config
        // look for config in system config directory
        .merge(
            File::with_name("/etc/chb4/config")
                .format(FileFormat::Toml)
                .required(false),
        )?
        // look for config in working directory
        .merge(
            File::with_name("config")
                .format(FileFormat::Toml)
                .required(false),
        )?
        // look for config in environment
        .merge(Environment::with_prefix("CHB4").separator("_"))?;

    config.set_default("manserver.host", "localhost")?;
    config.set_default("manserver.port", 3000)?;

    debug!("Loaded config");
    let action_index = actions::all();
    let command_index = commands::all();

    let mut manpage_index = manpages::Index::new();
    manpage_index.populate(action_index.clone());
    manpage_index.populate(command_index.clone());
    debug!(
        "Created and populated Manpages (count: {})",
        manpage_index.page_count()
    );

    let manpage_index = Arc::new(manpage_index);

    let cache: LruCache<String, String> = LruCache::new(manpage_index.page_count());
    let cache = Arc::new(Mutex::new(cache));

    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(move |_| {
        // While the state was moved into the make_service closure,
        // we need to clone it here because this closure is called
        // once for every connection.
        //
        // Each connection could send multiple requests, so
        // the `Service` needs a clone to handle later requests.
        let manpage_index = manpage_index.clone();
        let cache = cache.clone();

        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(move |req| {
                let manpage_index = manpage_index.clone();
                let cache = cache.clone();

                async move {
                    match handle_conn(manpage_index, cache, req) {
                        Ok(resp) => Ok::<_, Error>(resp),
                        Err(err) => {
                            error!("Handling Connection: {}", err);
                            Ok::<_, Error>(Response::new(Body::from(format!(
                                "Internal Server Error: {}",
                                err
                            ))))
                        }
                    }
                }
            }))
        }
    });

    let host: IpAddr = {
        let entry = config.get_str("manserver.host")?;

        if entry == "localhost" {
            [127, 0, 0, 1].into()
        } else {
            entry.parse()?
        }
    };
    let port = config.get_int("manserver.port")?.try_into()?;
    let addr = (host, port).into();

    let server = Server::bind(&addr).serve(make_service);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    info!("Listening on http://{}", addr);

    graceful.await?;

    Ok(())
}
