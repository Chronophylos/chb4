use super::prelude::*;
use chrono::prelude::*;
use chrono_tz::Tz;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("Unkown Offset: {0}")]
    UnknownOffset(String),
}

pub fn command() -> Arc<Command> {
    Command::with_name("time")
        .command(|_context, args, _msg, _user| match args.get(0) {
            None => Ok(MessageResult::Message(format!(
                "Current Time: {}",
                Utc::now()
            ))),
            Some(tz) => {
                let now = Utc::now();

                Ok(MessageResult::Message(match str_to_offset(tz) {
                    Ok(zone) => format!(
                        "Current Time: {}",
                        now.with_timezone(&zone)
                            .to_rfc3339_opts(SecondsFormat::Secs, false)
                    ),
                    Err(_) => {
                        let zone = match tz.parse::<Tz>() {
                            Ok(z) => z,
                            Err(_) => {
                                return Ok(MessageResult::Error("Could not parse zone".into()))
                            }
                        };
                        format!(
                            "Current Time: {}",
                            now.with_timezone(&zone)
                                .to_rfc3339_opts(SecondsFormat::Secs, false)
                        )
                    }
                }))
            }
        })
        .about("Get the current time")
        .done()
}

fn str_to_offset(name: &str) -> Result<FixedOffset> {
    let hour = 3600;
    let minute = 60;

    let offset = match name.to_uppercase().as_str() {
        "ACDT" => FixedOffset::east(10 * hour + 30 * minute), // Australian Central Daylight Saving Time
        "ACST" => FixedOffset::east(9 * hour + 30 * minute),  // Australian Central Standard Time
        "ACT" => FixedOffset::west(5 * hour),                 // Acre Time
        "ACWST" => FixedOffset::east(8 * hour + 45 * minute), // Australian Central Western Standard Time (unofficial)
        "ADT" => FixedOffset::west(3 * hour),                 // Atlantic Daylight Time
        "AEDT" => FixedOffset::east(11 * hour), // Australian Eastern Daylight Saving Time
        "AEST" => FixedOffset::east(10 * hour), // Australian Eastern Standard Time
        "AET" => FixedOffset::east(11 * hour),  // Australian Eastern Time
        "AFT" => FixedOffset::east(4 * hour + 30 * minute), // Afghanistan Time
        "AKDT" => FixedOffset::west(8 * hour),  // Alaska Daylight Time
        "AKST" => FixedOffset::west(9 * hour),  // Alaska Standard Time
        "ALMT" => FixedOffset::east(6 * hour),  // Alma-Ata Time
        "AMST" => FixedOffset::west(3 * hour),  // Amazon Summer Time (Brazil)
        "AMT" => FixedOffset::west(4 * hour),   // Amazon Time (Brazil)
        //"AMT" => FixedOffset::east(4 * hour),  // Armenia Time
        "ANAT" => FixedOffset::east(12 * hour), // Anadyr Time
        "AQTT" => FixedOffset::east(5 * hour),  // Aqtobe Time
        "ART" => FixedOffset::west(3 * hour),   // Argentina Time
        //"AST" => FixedOffset::east(3 * hour),  // Arabia Standard Time
        "KSA" => FixedOffset::east(3 * hour), // Arabia Standard Time
        "AST" => FixedOffset::west(4 * hour), // Atlantic Standard Time
        "AWST" => FixedOffset::east(8 * hour), // Australian Western Standard Time
        "AZOST" => FixedOffset::east(0),      // Azores Summer Time
        "AZOT" => FixedOffset::west(1 * hour), // Azores Standard Time
        "AZT" => FixedOffset::east(4 * hour), // Azerbaijan Time
        "BDT" => FixedOffset::east(8 * hour), // Brunei Time
        "BIOT" => FixedOffset::east(6 * hour), // British Indian Ocean Time
        "BIT" => FixedOffset::west(12 * hour), // Baker Island Time
        "BOT" => FixedOffset::west(4 * hour), // Bolivia Time
        "BRST" => FixedOffset::west(2 * hour), // Brasília Summer Time
        "BRT" => FixedOffset::west(3 * hour), // Brasília Time
        "BST" => FixedOffset::east(6 * hour), // Bangladesh Standard Time
        //"BST" => FixedOffset::east(11 * hour), // Bougainville Standard Time
        //"BST" => FixedOffset::east(1 * hour), // British Summer Time (British Standard Time from Feb 1968 to Oct 1971)
        "BTT" => FixedOffset::east(6 * hour), // Bhutan Time
        "CAT" => FixedOffset::east(2 * hour), // Central Africa Time
        "CCT" => FixedOffset::east(6 * hour + 30 * minute), // Cocos Islands Time
        "CDT" => FixedOffset::west(5 * hour), // Central Daylight Time (North America)
        //"CDT" => FixedOffset::west(4 * hour), // Cuba Daylight Time
        "CEST" => FixedOffset::east(2 * hour), // Central European Summer Time (Cf HAEC)
        "CET" => FixedOffset::east(1 * hour),  // Central European Time
        "CHADT" => FixedOffset::east(13 * hour + 45 * minute), // Chatham Daylight Time
        "CHAST" => FixedOffset::east(12 * hour + 45 * minute), // Chatham Standard Time
        "CHOT" => FixedOffset::east(8 * hour), // Choibalsan Standard Time
        "CHOST" => FixedOffset::east(9 * hour), // Choibalsan Summer Time
        "CHST" => FixedOffset::east(10 * hour), // Chamorro Standard Time
        "CHUT" => FixedOffset::east(10 * hour), // Chuuk Time
        "CIST" => FixedOffset::west(8 * hour), // Clipperton Island Standard Time
        "CIT" => FixedOffset::east(8 * hour),  // Central Indonesia Time
        "CKT" => FixedOffset::west(10 * hour), // Cook Island Time
        "CLST" => FixedOffset::west(3 * hour), // Chile Summer Time
        "CLT" => FixedOffset::west(4 * hour),  // Chile Standard Time
        "COST" => FixedOffset::west(4 * hour), // Colombia Summer Time
        "COT" => FixedOffset::west(5 * hour),  // Colombia Time
        "CST" => FixedOffset::west(6 * hour),  // Central Standard Time (North America)
        //"CST" => FixedOffset::east(8 * hour),  // China Standard Time
        //"CST" => FixedOffset::west(5 * hour), // Cuba Standard Time
        "CT" => FixedOffset::east(8 * hour),  // China Time
        "CVT" => FixedOffset::west(1 * hour), // Cape Verde Time
        "CWST" => FixedOffset::east(8 * hour + 45 * minute), // Central Western Standard Time (Australia) unofficial
        "CXT" => FixedOffset::east(7 * hour),                // Christmas Island Time
        "DAVT" => FixedOffset::east(7 * hour),               // Davis Time
        "DDUT" => FixedOffset::east(10 * hour),              // Dumont d'Urville Time
        "DFT" => FixedOffset::east(1 * hour), // AIX-specific equivalent of Central European Time
        "EASST" => FixedOffset::west(5 * hour), // Easter Island Summer Time
        "EAST" => FixedOffset::west(6 * hour), // Easter Island Standard Time
        "EAT" => FixedOffset::east(3 * hour), // East Africa Time
        "ECT" => FixedOffset::west(4 * hour), // Eastern Caribbean Time (does not recognise DST)
        //"ECT" => FixedOffset::west(5 * hour), // Ecuador Time
        "EDT" => FixedOffset::west(4 * hour), // Eastern Daylight Time (North America)
        "EEST" => FixedOffset::east(3 * hour), // Eastern European Summer Time
        "EET" => FixedOffset::east(2 * hour), // Eastern European Time
        "EGST" => FixedOffset::east(0),       // Eastern Greenland Summer Time
        "EGT" => FixedOffset::west(1 * hour), // Eastern Greenland Time
        "EIT" => FixedOffset::east(9 * hour), // Eastern Indonesian Time
        "EST" => FixedOffset::west(5 * hour), // Eastern Standard Time (North America)
        "FET" => FixedOffset::east(3 * hour), // Further-western European Time
        "FJT" => FixedOffset::east(12 * hour), // Fiji Time
        "FKST" => FixedOffset::west(3 * hour), // Falkland Islands Summer Time
        "FKT" => FixedOffset::west(4 * hour), // Falkland Islands Time
        "FNT" => FixedOffset::west(2 * hour), // Fernando de Noronha Time
        "GALT" => FixedOffset::west(6 * hour), // Galápagos Time
        "GAMT" => FixedOffset::west(9 * hour), // Gambier Islands Time
        "GET" => FixedOffset::east(4 * hour), // Georgia Standard Time
        "GFT" => FixedOffset::west(3 * hour), // French Guiana Time
        "GILT" => FixedOffset::east(12 * hour), // Gilbert Island Time
        "GIT" => FixedOffset::west(9 * hour), // Gambier Island Time
        "GMT" => FixedOffset::east(0),        // Greenwich Mean Time
        //"GST" => FixedOffset::west(2 * hour), // South Georgia and the South Sandwich Islands Time
        "GST" => FixedOffset::east(4 * hour), // Gulf Standard Time
        "GYT" => FixedOffset::west(4 * hour), // Guyana Time
        "HDT" => FixedOffset::west(9 * hour), // Hawaii–Aleutian Daylight Time
        "HAEC" => FixedOffset::east(2 * hour), // Heure Avancée d'Europe Centrale French-language name for CEST
        "HST" => FixedOffset::west(10 * hour), // Hawaii–Aleutian Standard Time
        "HKT" => FixedOffset::east(8 * hour),  // Hong Kong Time
        "HMT" => FixedOffset::east(5 * hour),  // Heard and McDonald Islands Time
        "HOVST" => FixedOffset::east(8 * hour), // Hovd Summer Time (not used from 2017-present)
        "HOVT" => FixedOffset::east(7 * hour), // Hovd Time
        "ICT" => FixedOffset::east(7 * hour),  // Indochina Time
        "IDLW" => FixedOffset::west(12 * hour), // International Day Line West time zone
        "IDT" => FixedOffset::east(3 * hour),  // Israel Daylight Time
        "IOT" => FixedOffset::east(3 * hour),  // Indian Ocean Time
        "IRDT" => FixedOffset::east(4 * hour + 30 * minute), // Iran Daylight Time
        "IRKT" => FixedOffset::east(8 * hour), // Irkutsk Time
        "IRST" => FixedOffset::east(3 * hour + 30 * minute), // Iran Standard Time
        "IST" => FixedOffset::east(5 * hour + 30 * minute), // Indian Standard Time
        //"IST" => FixedOffset::east(1 * hour),  // Irish Standard Time
        //"IST" => FixedOffset::east(2 * hour), // Israel Standard Time
        "JST" => FixedOffset::east(9 * hour), // Japan Standard Time
        "KALT" => FixedOffset::east(2 * hour), // Kaliningrad Time
        "KGT" => FixedOffset::east(6 * hour), // Kyrgyzstan Time
        "KOST" => FixedOffset::east(11 * hour), // Kosrae Time
        "KRAT" => FixedOffset::east(7 * hour), // Krasnoyarsk Time
        "KST" => FixedOffset::east(9 * hour), // Korea Standard Time
        "LHST" => FixedOffset::east(10 * hour + 30 * minute), // Lord Howe Standard Time
        //"LHST" => FixedOffset::east(11 * hour), // Lord Howe Summer Time
        "LINT" => FixedOffset::east(14 * hour), // Line Islands Time
        "MAGT" => FixedOffset::east(12 * hour), // Magadan Time
        "MART" => FixedOffset::west(9 * hour + 30 * minute), // Marquesas Islands Time
        "MAWT" => FixedOffset::east(5 * hour),  // Mawson Station Time
        "MDT" => FixedOffset::west(6 * hour),   // Mountain Daylight Time (North America)
        "MET" => FixedOffset::east(1 * hour),   // Middle European Time Same zone as CET
        "MEST" => FixedOffset::east(2 * hour),  // Middle European Summer Time Same zone as CEST
        "MHT" => FixedOffset::east(12 * hour),  // Marshall Islands Time
        "MIST" => FixedOffset::east(11 * hour), // Macquarie Island Station Time
        "MIT" => FixedOffset::west(9 * hour + 30 * minute), // Marquesas Islands Time
        "MMT" => FixedOffset::east(6 * hour + 30 * minute), // Myanmar Standard Time
        "MSK" => FixedOffset::east(3 * hour),   // Moscow Time
        //"MST" => FixedOffset::east(8 * hour),  // Malaysia Standard Time
        "MST" => FixedOffset::west(7 * hour), // Mountain Standard Time (North America)
        "MUT" => FixedOffset::east(4 * hour), // Mauritius Time
        "MVT" => FixedOffset::east(5 * hour), // Maldives Time
        "MYT" => FixedOffset::east(8 * hour), // Malaysia Time
        "NCT" => FixedOffset::east(11 * hour), // New Caledonia Time
        "NDT" => FixedOffset::west(2 * hour + 30 * minute), // Newfoundland Daylight Time
        "NFT" => FixedOffset::east(11 * hour), // Norfolk Island Time
        "NOVT" => FixedOffset::east(7 * hour), // Novosibirsk Time
        "NPT" => FixedOffset::east(5 * hour + 45 * minute), // Nepal Time
        "NST" => FixedOffset::west(3 * hour + 30 * minute), // Newfoundland Standard Time
        "NT" => FixedOffset::west(3 * hour + 30 * minute), // Newfoundland Time
        "NUT" => FixedOffset::west(11 * hour), // Niue Time
        "NZDT" => FixedOffset::east(13 * hour), // New Zealand Daylight Time
        "NZST" => FixedOffset::east(12 * hour), // New Zealand Standard Time
        "OMST" => FixedOffset::east(6 * hour), // Omsk Time
        "ORAT" => FixedOffset::east(5 * hour), // Oral Time
        "PDT" => FixedOffset::west(7 * hour), // Pacific Daylight Time (North America)
        "PET" => FixedOffset::west(5 * hour), // Peru Time
        "PETT" => FixedOffset::east(12 * hour), // Kamchatka Time
        "PGT" => FixedOffset::east(10 * hour), // Papua New Guinea Time
        "PHOT" => FixedOffset::east(13 * hour), // Phoenix Island Time
        "PHT" => FixedOffset::east(8 * hour), // Philippine Time
        "PKT" => FixedOffset::east(5 * hour), // Pakistan Standard Time
        "PMDT" => FixedOffset::west(2 * hour), // Saint Pierre and Miquelon Daylight Time
        "PMST" => FixedOffset::west(3 * hour), // Saint Pierre and Miquelon Standard Time
        "PONT" => FixedOffset::east(11 * hour), // Pohnpei Standard Time
        "PST" => FixedOffset::west(8 * hour), // Pacific Standard Time (North America)
        //"PST" => FixedOffset::east(8 * hour),  // Philippine Standard Time
        "PYST" => FixedOffset::west(3 * hour), // Paraguay Summer Time
        "PYT" => FixedOffset::west(4 * hour),  // Paraguay Time
        "RET" => FixedOffset::east(4 * hour),  // Réunion Time
        "ROTT" => FixedOffset::west(3 * hour), // Rothera Research Station Time
        "SAKT" => FixedOffset::east(11 * hour), // Sakhalin Island Time
        "SAMT" => FixedOffset::east(4 * hour), // Samara Time
        "SAST" => FixedOffset::east(2 * hour), // South African Standard Time
        "SBT" => FixedOffset::east(11 * hour), // Solomon Islands Time
        "SCT" => FixedOffset::east(4 * hour),  // Seychelles Time
        "SDT" => FixedOffset::west(10 * hour), // Samoa Daylight Time
        "SGT" => FixedOffset::east(8 * hour),  // Singapore Time
        "SLST" => FixedOffset::east(5 * hour + 30 * minute), // Sri Lanka Standard Time
        "SRET" => FixedOffset::east(11 * hour), // Srednekolymsk Time
        "SRT" => FixedOffset::west(3 * hour),  // Suriname Time
        //"SST" => FixedOffset::west(11 * hour),  // Samoa Standard Time
        "SST" => FixedOffset::east(8 * hour), // Singapore Standard Time
        "SYOT" => FixedOffset::east(3 * hour), // Showa Station Time
        "TAHT" => FixedOffset::west(10 * hour), // Tahiti Time
        "THA" => FixedOffset::east(7 * hour), // Thailand Standard Time
        "TFT" => FixedOffset::east(5 * hour), // French Southern and Antarctic Time
        "TJT" => FixedOffset::east(5 * hour), // Tajikistan Time
        "TKT" => FixedOffset::east(13 * hour), // Tokelau Time
        "TLT" => FixedOffset::east(9 * hour), // Timor Leste Time
        "TMT" => FixedOffset::east(5 * hour), // Turkmenistan Time
        "TRT" => FixedOffset::east(3 * hour), // Turkey Time
        "TOT" => FixedOffset::east(13 * hour), // Tonga Time
        "TVT" => FixedOffset::east(12 * hour), // Tuvalu Time
        "ULAST" => FixedOffset::east(9 * hour), // Ulaanbaatar Summer Time
        "ULAT" => FixedOffset::east(8 * hour), // Ulaanbaatar Standard Time
        "UTC" => FixedOffset::east(0),        // Coordinated Universal Time
        "UYST" => FixedOffset::west(2 * hour), // Uruguay Summer Time
        "UYT" => FixedOffset::west(3 * hour), // Uruguay Standard Time
        "UZT" => FixedOffset::east(5 * hour), // Uzbekistan Time
        "VET" => FixedOffset::west(4 * hour), // Venezuelan Standard Time
        "VLAT" => FixedOffset::east(10 * hour), // Vladivostok Time
        "VOLT" => FixedOffset::east(4 * hour), // Volgograd Time
        "VOST" => FixedOffset::east(6 * hour), // Vostok Station Time
        "VUT" => FixedOffset::east(11 * hour), // Vanuatu Time
        "WAKT" => FixedOffset::east(12 * hour), // Wake Island Time
        "WAST" => FixedOffset::east(2 * hour), // West Africa Summer Time
        "WAT" => FixedOffset::east(1 * hour), // West Africa Time
        "WEST" => FixedOffset::east(1 * hour), // Western European Summer Time
        "WET" => FixedOffset::east(0),        // Western European Time
        "WIT" => FixedOffset::east(7 * hour), // Western Indonesian Time
        "WGST" => FixedOffset::west(2 * hour), // West Greenland Summer Time
        "WGT" => FixedOffset::west(3 * hour), // West Greenland Time
        "WST" => FixedOffset::east(8 * hour), // Western Standard Time
        "YAKT" => FixedOffset::east(9 * hour), // Yakutsk Time
        "YEKT" => FixedOffset::east(5 * hour), // Yekaterinburg Time
        _ => bail!(TimeError::UnknownOffset(name.into())),
    };
    Ok(offset)
}
