#[macro_use]
extern crate log;
use chb4::{actions, commands, manpages};
use config::{Config, Environment, File, FileFormat};
use flexi_logger::Logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create logger with custom format (`chb4::format`)
    Logger::with_env_or_str("chb4=trace, rustls=info, debug")
        .format(chb4::format)
        .start()?;

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

    info!("Loaded config");
    let action_index = actions::all();
    let command_index = commands::all();

    let mut manpage_index = manpages::Index::new();
    manpage_index.populate(action_index.clone());
    manpage_index.populate(command_index.clone());
    debug!(
        "Created and populated Manpages (count: {})",
        manpage_index.page_count()
    );

    let path = config.get_str("manpages.path")?;

    info!("Writing manpages (output: {})", path);

    manpage_index.write(path)?;

    Ok(())
}
