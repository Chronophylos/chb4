//! Format Specification
//!
//! ```ABNF
//! voicemail = recipients [schedule] SP message
//!
//! recipients = recipient-name *(recpipent-sep recipient-name)
//! recipient-name = ["@"] *ALPHA
//! recipient-sep = reci-sep-and / reci-sep-comma
//! reci-sep-and = SP ("and" / "und" / "&&") SP
//! reci-sep-comma = [SP] "," SP
//!
//! message = *ALPHA *(SP *ALPHA)
//! schedule = SP absolute-schedule / relative-schedule
//!
//! absolute-schedule = "on" / "at" SP absolute-schedule-spec
//! absolute-schedule-spec = rfc2822 ; see https://tools.ietf.org/html/rfc2822#section-3.3
//! absolute-schedule-spec =/ rfc3339.date ; see https://tools.ietf.org/html/rfc3339#appendix-A
//! absolute-schedule-spec =/ rfc3339.time
//! absolute-schedule-spec =/ rfc3339.iso-date-time
//!
//! relative-schedule = "in" SP relative-schedule-spec *(SP relative-schedule-spec)
//! relative-schedule-spec = amount [SP] time-unit
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
//! fortnight = "fort" ("night" / "nite") ["s"]
//! month = "month" ["s"]
//! quatal = "q" ["atal" ["s"]]
//! year = "y" ["ear" ["s"]]
//! decade = "decade" ["s"]
//! century = "century" ["s"]
//!
//! ```

//                                           what kind of error occured
//                                           |
//                                           vvvvvvvvvvvvvvvv
// how to use nom errors: Err(Err::Error((i, ErrorKind::Digit)))
//                                        ^
//                                        |
//                     left over characters

use super::Voicemail;
use chrono::prelude::*;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while_m_n},
    character::complete::digit1,
    combinator::{map, map_res, opt},
    multi::{fold_many0, separated_list},
    sequence::pair,
    IResult,
};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Units {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Fortnight,
    Month,
    Quartal,
    Year,
    Decade,
    Century,
}

#[cfg(not(test))]
fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

#[cfg(test)]
fn now() -> NaiveDateTime {
    NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0)
}

/// century = "century" ["s"]
fn parse_century<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("centurys"), tag_no_case("century")))(i)?;

    Ok((i, Units::Century))
}

/// decade = "decade" ["s"]
fn parse_decade<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("decades"), tag_no_case("decade")))(i)?;

    Ok((i, Units::Decade))
}

/// year = "y" ["ear" ["s"]]
fn parse_year<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("years"), tag_no_case("year")))(i)?;

    Ok((i, Units::Year))
}

/// quatal = "q" ["atal" ["s"]]
fn parse_quatal<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((
        tag_no_case("quatals"),
        tag_no_case("qatal"),
        tag_no_case("q"),
    ))(i)?;

    Ok((i, Units::Quartal))
}

/// month = "month" ["s"]
fn parse_month<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("months"), tag_no_case("month")))(i)?;

    Ok((i, Units::Month))
}

/// fortnight = "fort" ("night" / "nite") ["s"]
fn parse_fortnite<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((
        tag_no_case("fortnights"),
        tag_no_case("fortnight"),
        tag_no_case("fortnites"),
        tag_no_case("fortnite"),
    ))(i)?;

    Ok((i, Units::Fortnight))
}

/// week = "w" ["eek" ["s"]]
fn parse_week<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("weeks"), tag_no_case("week"), tag_no_case("w")))(i)?;

    Ok((i, Units::Week))
}

/// day = "d" ["ay" ["s"]]
fn parse_day<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("days"), tag_no_case("day"), tag_no_case("d")))(i)?;

    Ok((i, Units::Day))
}

/// hour = "h" ["our" ["s"]]
fn parse_hour<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((tag_no_case("hours"), tag_no_case("hour"), tag_no_case("h")))(i)?;

    Ok((i, Units::Hour))
}

/// minute = "m" ["in" ["ute" ["s"]]]
fn parse_minute<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((
        tag_no_case("minutes"),
        tag_no_case("minute"),
        tag_no_case("min"),
        tag_no_case("m"),
    ))(i)?;

    Ok((i, Units::Minute))
}

/// second = "s" ["ec" ["ond" ["s"]]]
fn parse_second<'a>(i: &'a str) -> IResult<&'a str, Units> {
    let (i, _) = alt((
        tag_no_case("seconds"),
        tag_no_case("second"),
        tag_no_case("sec"),
        tag_no_case("s"),
    ))(i)?;

    Ok((i, Units::Second))
}

fn parse_unit<'a>(i: &'a str) -> IResult<&'a str, Units> {
    alt((
        parse_century,
        parse_decade,
        parse_year,
        parse_decade,
        parse_quatal,
        parse_month,
        parse_fortnite,
        parse_week,
        parse_day,
        parse_hour,
        parse_minute,
        parse_second,
    ))(i)
}

fn parse_amount<'a>(i: &'a str) -> IResult<&'a str, u64> {
    map_res(digit1, |digit_str: &str| digit_str.parse::<u64>())(i)
}

fn take_space<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    take_while(char::is_whitespace)(i)
}

/// relative-schedule-spec = amount [SP] time-unit
fn parse_relative_schedule_spec<'a>(i: &'a str) -> IResult<&'a str, chrono::Duration> {
    let (i, amount) = parse_amount(i)?;
    let (i, _) = opt(take_space)(i)?;
    let (i, unit) = parse_unit(i)?;

    let amount = Duration::from_secs(amount);

    Ok((
        i,
        // Time is hard and having accurate times is even harder.
        // There will always be some inaccuracy and I hate it but can't really do anything about
        // it. In one of the tests the inaccuracy is at 0.008%. I think this is acceptable for what
        // I need it to be. If I have some time I may come back to these numbers and tweak them.
        to_chrono_duration(match unit {
            // easy stuff
            Units::Second => amount,
            Units::Minute => amount * 60,
            Units::Hour => amount * 3_600,
            Units::Day => amount * 86_400,
            Units::Week => amount * 604_800,
            Units::Fortnight => amount * 1_209_600,
            // hard stuff
            // a month is 30 days and 10 hours
            Units::Month => amount * 2_628_000,
            Units::Quartal => amount * 7_884_000,
            // a year is 356 days
            Units::Year => amount * 31_536_000,
            // a decade is 3562 days
            Units::Decade => amount * 315_568_800,
            // a centry is 35624 days
            Units::Century => amount * 3_155_695_200,
        }),
    ))
}

fn to_chrono_duration(d: Duration) -> chrono::Duration {
    match chrono::Duration::from_std(d) {
        Ok(d) => d,
        Err(_) => {
            if d.as_secs() > 0 {
                chrono::Duration::max_value()
            } else {
                chrono::Duration::min_value()
            }
        }
    }
}

/// relative-schedule = "in" SP relative-schedule-spec *(SP relative-schedule-spec)
fn parse_relative_schedule<'a>(i: &'a str) -> IResult<&'a str, chrono::Duration> {
    let (i, _) = tag_no_case("in")(i)?;
    let (i, _) = take_space(i)?;

    let (i, dur) = parse_relative_schedule_spec(i)?;
    fold_many0(
        |i| {
            let (i, _) = take_space(i)?;
            parse_relative_schedule_spec(i)
        },
        dur,
        |acc, dur| acc + dur,
    )(i)
}

fn is_number(c: char) -> bool {
    c.is_digit(10)
}

fn num_4<'a, T>(i: &'a str) -> IResult<&'a str, T>
where
    T: std::str::FromStr,
{
    map_res(take_while_m_n(4, 4, is_number), str::parse)(i)
}

fn num_2<'a, T>(i: &'a str) -> IResult<&'a str, T>
where
    T: std::str::FromStr,
{
    map_res(take_while_m_n(2, 2, is_number), str::parse)(i)
}

#[allow(dead_code)]
fn parse_rfc2822<'a>(_i: &'a str) -> IResult<&'a str, DateTime<Utc>> {
    unimplemented!()
}

fn parse_rfc3339_date<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, year) = num_4::<i32>(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, month) = num_2::<u32>(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, day) = num_2::<u32>(i)?;

    Ok((i, NaiveDate::from_ymd(year, month, day).and_hms(0, 0, 0)))
}

fn parse_rfc3339_time<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, hour) = num_2::<u32>(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, minute) = num_2::<u32>(i)?;
    let (i, maybe_second) = opt(pair(tag(":"), num_2::<u32>))(i)?;
    let (_, second) = maybe_second.unwrap_or(("", 0));

    Ok((i, Utc::today().naive_utc().and_hms(hour, minute, second)))
}

fn parse_rfc3339_date_time<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, year) = num_4::<i32>(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, month) = num_2::<u32>(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, day) = num_2::<u32>(i)?;

    let (i, _) = alt((tag("T"), take_space))(i)?;

    let (i, hour) = num_2::<u32>(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, minute) = num_2::<u32>(i)?;
    let (i, maybe_second) = opt(pair(tag(":"), num_2::<u32>))(i)?;
    let (_, second) = maybe_second.unwrap_or(("", 0));

    Ok((
        i,
        NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second),
    ))
}

/// absolute-schedule-spec = rfc2822 ; see https://tools.ietf.org/html/rfc2822#section-3.3
/// absolute-schedule-spec =/ rfc3339.date ; see https://tools.ietf.org/html/rfc3339#appendix-A
/// absolute-schedule-spec =/ rfc3339.time
/// absolute-schedule-spec =/ rfc3339.iso-date-time
fn parse_absoulute_schedule_spec<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, date) = alt((
        //parse_rfc2822, // TODO: implement
        parse_rfc3339_date_time,
        parse_rfc3339_date,
        parse_rfc3339_time,
    ))(i)?;

    Ok((i, date))
}

/// absolute-schedule = "on" / "at" SP absolute-schedule-spec
fn parse_absoulute_schedule<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, _) = alt((tag_no_case("on"), tag_no_case("at")))(i)?;
    let (i, _) = take_space(i)?;

    parse_absoulute_schedule_spec(i)
}

/// schedule = SP absolute-schedule / relative-schedule
fn parse_schedule<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    alt((
        parse_absoulute_schedule,
        map(parse_relative_schedule, |d: chrono::Duration| {
            println!("{:?}", d);
            now() + d
        }),
    ))(i)
}

/// message = *ALPHA *(SP *ALPHA)
fn parse_message<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let (i, _) = take_space(i)?;
    Ok(("", i))
}

fn is_recipent_name(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// recipient-name = ["@"] *ALPHA
fn parse_recipient_name<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let (i, _) = opt(tag("@"))(i)?;
    take_while(is_recipent_name)(i)
}

/// recipient-sep = reci-sep-and / reci-sep-comma
fn parse_recipient_sep<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    alt((parse_recipient_sep_and, parse_recipient_sep_comma))(i)
}

/// reci-sep-and = SP ("and" / "und" / "&&") SP
fn parse_recipient_sep_and<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let (i, _) = take_space(i)?;
    let (i, sep) = alt((tag_no_case("and"), tag_no_case("und"), tag("&&")))(i)?;
    let (i, _) = take_space(i)?;

    Ok((i, sep))
}

/// reci-sep-comma = [SP] "," SP
fn parse_recipient_sep_comma<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let (i, _) = opt(take_space)(i)?;
    let (i, comma) = tag(",")(i)?;
    let (i, _) = take_space(i)?;

    Ok((i, comma))
}

/// recipients = recipient-name *(recipipent-sep recipient-name)
fn parse_recipients<'a>(i: &'a str) -> IResult<&'a str, Vec<&'a str>> {
    separated_list(parse_recipient_sep, parse_recipient_name)(i)
}

fn parse_schedule_with_space<'a>(i: &'a str) -> IResult<&'a str, NaiveDateTime> {
    let (i, _) = take_space(i)?;

    parse_schedule(i)
}

/// voicemail = recipients [schedule] SP message
pub fn parse_voicemail<'a>(i: &'a str) -> IResult<&'a str, Voicemail> {
    let (i, recipients) = parse_recipients(i)?;
    let (i, schedule) = opt(parse_schedule_with_space)(i)?;
    let (i, message) = parse_message(i)?;

    Ok((
        i,
        Voicemail {
            recipients: recipients.iter().map(|&x| x.to_owned()).collect(),
            message: message.to_owned(),
            schedule,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schedule() {
        assert_eq!(
            parse_schedule("on 2020-10-10"),
            Ok(("", NaiveDate::from_ymd(2020, 10, 10).and_hms(0, 0, 0)))
        );

        assert_eq!(
            parse_schedule("on 2020-10-10 12:34:56"),
            Ok(("", NaiveDate::from_ymd(2020, 10, 10).and_hms(12, 34, 56)))
        );

        assert_eq!(
            parse_schedule("in 2 days 2 week 4 months 1 hour 3 decades"),
            Ok(("", NaiveDate::from_ymd(2030, 5, 17).and_hms(23, 0, 0)))
        )
    }

    #[test]
    fn test_parse_voicemail() {
        assert_eq!(
            parse_voicemail("some_weeb weebSlam"),
            Ok((
                "",
                Voicemail {
                    recipients: vec![String::from("some_weeb")],
                    message: String::from("weebSlam"),
                    schedule: None
                }
            ))
        );

        assert_eq!(
            parse_voicemail("coroner on 2020-10-24 does corona still exist?"),
            Ok((
                "",
                Voicemail {
                    recipients: vec![String::from("coroner")],
                    message: String::from("does corona still exist?"),
                    schedule: Some(NaiveDate::from_ymd(2020, 10, 24).and_hms(0, 0, 0))
                }
            ))
        );

        assert_eq!(
            parse_voicemail("nizzlenils, nizzlenico in 1 fortnight Pepeja"),
            Ok((
                "",
                Voicemail {
                    recipients: vec![String::from("nizzlenils"), String::from("nizzlenico")],
                    message: String::from("Pepeja"),
                    schedule: Some(NaiveDate::from_ymd(2000, 1, 15).and_hms(0, 0, 0)),
                }
            ))
        )
    }
}
