use super::{Channel, User};
use crate::context::{Connection, Context};
use crate::database::user;
use crate::schema::*;
use chrono::prelude::*;
use snafu::{OptionExt, ResultExt, Snafu};
use std::sync::Arc;

#[derive(Snafu, Debug)]
enum Error {
    #[snafu(display("Getting receiver: {}", source))]
    GetReceiver { source: user::Error },

    #[snafu(display("Receiver not found (id: {})", id))]
    ReceiverNotFound { id: i32 },

    #[snafu(display("Getting creator: {}", source))]
    GetCreator { source: user::Error },

    #[snafu(display("Creator not found (id: {})", id))]
    CreatorNotFound { id: i32 },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Queryable, Identifiable, Associations, Debug)]
#[belongs_to(User, foreign_key = "receiver_id")]
#[belongs_to(Channel)]
#[table_name = "voicemails"]
pub struct Voicemail {
    pub id: i32,
    pub creator_id: i32,
    pub receiver_id: i32,
    pub channel_id: i32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub active: bool,
    pub message: String,
}

impl Voicemail {
    pub fn to_string(&self, conn: &Connection) -> String {
        match Self::format(conn, self) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        }
    }

    fn format(conn: &Connection, voicemail: &Voicemail) -> Result<String> {
        let creator = user::by_id(conn, voicemail.creator_id)
            .context(GetCreator)?
            .context(CreatorNotFound {
                id: voicemail.creator_id,
            })?;

        Ok(format!(
            "{}, {}: {}",
            creator.display_name_or_name(),
            voicemail.created,
            voicemail.message
        ))
    }

    fn format_vec(context: Arc<Context>, voicemails: Vec<Voicemail>) -> Result<String> {
        let conn = &context.conn();
        let receiver_id = voicemails[0].receiver_id;
        let receiver = user::by_id(conn, receiver_id)
            .context(GetReceiver)?
            .context(ReceiverNotFound { id: receiver_id })?;

        let voicemails: Vec<_> = voicemails
            .iter()
            .map(|v| Self::format(conn, v))
            .collect::<Result<Vec<_>>>()?;

        Ok(format!(
            "{}, {} message(s) for you: {}",
            receiver.display_name_or_name(),
            voicemails.len(),
            voicemails.join(" – ")
        ))
    }
}

#[derive(Insertable, Debug)]
#[table_name = "voicemails"]
pub struct NewVoicemail {
    pub creator_id: i32,
    pub receiver_id: i32,
    pub created: NaiveDateTime,
    pub scheduled: Option<NaiveDateTime>,
    pub message: String,
}

#[derive(Identifiable, AsChangeset)]
#[table_name = "voicemails"]
pub struct SetActiveVoicemail {
    pub id: i32,
    pub active: bool,
}
