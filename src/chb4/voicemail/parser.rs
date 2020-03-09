//! Format Specification
//!
//! ```ABNF
//! voicemail = recipients SP message [SP schedule]
//!
//! recipients = recipient-name *(recpipent-sep recipient-name)
//! recipient-name = ["@"] *ALPHA
//! recipient-sep = reci-sep-and / reci-sep-comma
//! reci-sep-and = SP ("and" / "und" / "&&") SP
//! reci-sep-comma = [SP] "," SP
//!
//! message = *ALPHA *(SP *ALPHA)
//! schedule = absolute-schedule / relative-schedule
//!
//! absolute-schedule = "on" / "at" relative-schedule-spec
//! absolute-schedule-spec = rfc2822 ; see https://tools.ietf.org/html/rfc2822#section-3.3
//! absolute-schedule-spec =/ rfc3339.date ; see https://tools.ietf.org/html/rfc3339#appendix-A
//! absolute-schedule-spec =/ rfc3339.time
//! absolute-schedule-spec =/ rfc3339.iso-date-time
//!
//! relative-schedule = "in" relative-schedule-spec *(SP relative-schedule-spec)
//! relative-schedule-spec = amount time-unit
//!
//! amount = 1*4DIGIT
//!
//! time-unit = second / minute / hour
//! time-unit =/ day / week / fortnite
//! time-unit =/ month / quatal
//! time-unit =/ year / decade / century
//!
//! second = "s" ["ec" ["ond" ["s"]]]
//! minute = "m" ["in" ["ute" ["s"]]]
//! hour = "h" ["our" ["s"]]
//! day = "d" ["ay" ["s"]]
//! week = "w" ["eek" ["s"]]
//! fortnite = "fortnite" ["s"]
//! month = "month" ["s"]
//! quatal = "q" ["atal" ["s"]]
//! year = "y" ["ear" ["s"]]
//! decade = "decade" ["s"]
//! century = "century" ["s"]
//!
//! ```

use nom::IResult;

fn parse_recipents<'a>(i: &'a str) -> IResult<&'a str, Vec<String>> {}
fn parse_message<'a>(i: &'a str) -> IResult<&'a str, String> {}
fn parse_schedule<'a>(i: &'a str) -> IResult<&'a str, DateTime> {}

pub fn parse_voicemail<'a>(i: &'a str) -> IResult<&'a str, Voicemail> {}
