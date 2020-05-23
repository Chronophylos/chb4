use super::prelude::*;
use bytes::buf::BufExt as _;
use bytes::Buf;
use hyper::Client;
use hyper::Uri;
use serde_json::Value;
use simple_error::SimpleError;
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
struct FontStats {
    pub totalViews: u64,
}

pub fn command() -> Arc<Command> {
    let uri = Uri::from_static("https://fonts.google.com/analytics");

    // todo cache
    async fn stats(uri: Uri) -> Result<FontStats> {
        let client = Client::new();
        let resp = client.get(uri).await?;

        if !resp.status().is_success() {
            SimpleError::new("Status code is not Ok");
        }

        let body = hyper::body::aggregate(resp).await?;

        // yank first 5 bytes
        body.advance(5);

        // try to parse as json with serde_json
        let stats: FontStats = serde_json::from_reader(body.reader())?;

        Ok(stats)
    }

    fn font(name: &str) -> MessageResult {
        MessageResult::Message(String::from("Not implemented"))
    }

    fn total() -> MessageResult {
        let stats = async {
            stats(uri).await;
        };
        MessageResult::Message(format!("Total views: {}", stats.totalViews))
    }

    Command::with_name("font")
        .command(move |_context, args, msg| match args.get(0) {
            Some(name) => font(name),
            None => total(),
        })
        .done()
}
