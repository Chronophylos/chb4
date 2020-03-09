pub struct Voicemail {
    receivers: Vec<String>,
    scheduled: Option<NaiveDateTime>,
    message: String,
}

mod parser;
