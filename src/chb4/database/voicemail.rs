use super::user;
use crate::models::{NewVoicemail, Voicemail};
use crate::schema::*;
use crate::voicemail::Voicemail as ParsedVoicemail;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::PgConnection;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    GetReceiverByName {
        source: user::Error,
    },

    CreateReceiverWithName {
        source: user::Error,
    },

    InsertVoicemails {
        source: diesel::result::Error,
    },

    GetActiveVoicemailsForUser {
        twitch_id: i32,
        source: diesel::result::Error,
    },

    SetVoicemailToInactive {
        twitch_id: i32,
        source: diesel::result::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn new(
    conn: &PgConnection,
    parsed_voicemail: &ParsedVoicemail,
    creator_id: i32,
    now: NaiveDateTime,
) -> Result<Vec<Voicemail>> {
    trace!("Creating new voicemails");

    let mut new_voicemails: Vec<NewVoicemail> = Vec::new();

    for receiver_name in &parsed_voicemail.recipients {
        let receiver = match user::by_name(conn, receiver_name).context(GetReceiverByName)? {
            Some(u) => u,
            None => user::with_name(conn, receiver_name).context(CreateReceiverWithName)?,
        };

        new_voicemails.push(NewVoicemail {
            creator_id,
            receiver_id: receiver.id,
            created: now,
            scheduled: parsed_voicemail.schedule,
            message: parsed_voicemail.message.clone(),
        })
    }

    diesel::insert_into(voicemails::table)
        .values(&new_voicemails)
        .get_results(conn)
        .context(InsertVoicemails)
}

pub fn pop(conn: &PgConnection, twitch_id: i32) -> Result<Vec<Voicemail>> {
    trace!("Popping voicemails (twitch_id: {})", twitch_id);

    let vms = voicemails::table
        .filter(
            voicemails::receiver_id
                .eq(twitch_id)
                .and(voicemails::active.eq(true)),
        )
        .get_results(conn)
        .context(GetActiveVoicemailsForUser { twitch_id })?;

    for voicemail in &vms {
        diesel::update(voicemail)
            .set(voicemails::active.eq(false))
            .execute(conn)
            .context(SetVoicemailToInactive { twitch_id })?;
    }

    Ok(vms)
}
