use super::{user, Channel, Connection, User};
use crate::schema::*;
use crate::voicemail::Voicemail as ParsedVoicemail;
use chrono::prelude::*;
use diesel::prelude::*;
use humantime::format_duration;
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Getting receiver (name: {}): {}", name, source))]
    GetReceiverByName {
        name: String,
        source: user::Error,
    },

    #[snafu(display("Getting receiver (id: {}): {}", id, source))]
    GetReceiverByID {
        id: i32,
        source: user::Error,
    },

    #[snafu(display("Receiver not found (id: {})", id))]
    ReceiverNotFound {
        id: i32,
    },

    #[snafu(display("Getting creator (id: {}): {}", id, source))]
    GetCreatorByID {
        id: i32,
        source: user::Error,
    },

    #[snafu(display("Getting creator (twitch_id: {}): {}", twitch_id, source))]
    GetCreatorByTwitchID {
        twitch_id: i64,
        source: user::Error,
    },

    #[snafu(display("Creator not found (twitch_id: {})", twitch_id))]
    CreatorNotFoundTID {
        twitch_id: i64,
    },

    #[snafu(display("Creator not found (id: {})", id))]
    CreatorNotFoundID {
        id: i32,
    },

    #[snafu(display("Creating receiver (name: {}): ", source))]
    CreateReceiverWithName {
        name: String,
        source: user::Error,
    },

    #[snafu(display("Inserting new voicemails (voicemails: {:#?}): {}", voicemails, source))]
    InsertVoicemails {
        voicemails: Vec<NewVoicemail>,
        source: diesel::result::Error,
    },

    #[snafu(display("Getting voicemail (id: {}): {}", id, source))]
    GetVoicemailByID {
        id: i32,
        source: diesel::result::Error,
    },

    UpdateActiveVoicemail {
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Queryable, Identifiable, AsChangeset, Associations, Debug)]
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
    pub fn new(
        conn: &Connection,
        parsed_voicemail: &ParsedVoicemail,
        twitch_id: i64,
        channel_id: i32,
        now: NaiveDateTime,
    ) -> Result<Vec<Voicemail>> {
        trace!("Creating new voicemails");

        let mut new_voicemails: Vec<NewVoicemail> = Vec::new();
        let creator = User::by_twitch_id(conn, twitch_id)
            .context(GetCreatorByTwitchID { twitch_id })?
            .context(CreatorNotFoundTID { twitch_id })?;

        for name in &parsed_voicemail.recipients {
            let receiver = match User::by_name(conn, name).context(GetReceiverByName { name })? {
                Some(u) => u,
                None => User::with_name(conn, name).context(CreateReceiverWithName { name })?,
            };

            new_voicemails.push(NewVoicemail {
                creator_id: creator.id,
                receiver_id: receiver.id,
                channel_id,
                created: now,
                scheduled: parsed_voicemail.schedule,
                message: parsed_voicemail.message.clone(),
            })
        }

        diesel::insert_into(voicemails::table)
            .values(&new_voicemails)
            .get_results(conn)
            .context(InsertVoicemails {
                voicemails: new_voicemails,
            })
    }

    pub fn by_id(conn: &Connection, id: i32) -> Result<Option<Voicemail>> {
        trace!("Getting voicemail (id: {})", id);

        voicemails::table
            .filter(voicemails::id.eq(id))
            .get_result(conn)
            .optional()
            .context(GetVoicemailByID { id })
    }

    pub fn to_string(&self, conn: &Connection) -> String {
        match Self::format(conn, self) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        }
    }

    pub fn set_active(&self, conn: &Connection, active: bool) -> Result<()> {
        trace!("Disable voicemail (id: {})", self.id);

        diesel::update(voicemails::table)
            .filter(voicemails::id.eq(self.id))
            .set(voicemails::active.eq(active))
            .execute(conn)
            .context(UpdateActiveVoicemail)
            .map(|_| ())
    }

    fn format(conn: &Connection, voicemail: &Voicemail) -> Result<String> {
        let creator = User::by_id(conn, voicemail.creator_id)
            .context(GetCreatorByID {
                id: voicemail.creator_id,
            })?
            .context(CreatorNotFoundID {
                id: voicemail.creator_id,
            })?;

        Ok(format!(
            "{}, {} ago: {}",
            creator.display_name_or_name(),
            format_duration(
                Utc::now()
                    .naive_utc()
                    .signed_duration_since(voicemail.created)
                    .to_std()
                    .unwrap_or_default()
            ),
            voicemail.message
        ))
    }

    pub fn format_vec(conn: &Connection, voicemails: Vec<Voicemail>) -> Result<String> {
        let receiver_id = voicemails[0].receiver_id;
        let receiver = User::by_id(conn, receiver_id)
            .context(GetReceiverByID { id: receiver_id })?
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
    pub channel_id: i32,
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
