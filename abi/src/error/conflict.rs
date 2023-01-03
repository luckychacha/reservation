use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // s.parse() -> impl FromStr for ReservationConflict
        if let Ok(conflict) = s.parse() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // from_str: impl FromStr for ParsedInfo，将字符串转为 ParsedInfo，即两个 HashMap
        // try_into: impl TryFrom<ParsedInfo> for ReservationConflict，将 ParsedInfo 尝试转换为 ReservationConflict
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    // value.new.try_into()?: impl TryFrom<HashMap<String, String>> for ReservationWindow， 将 HashMap 转为 ReservationWindow
    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    // Key (resource_id, timespan)=(ocean-view-room-666, [\"2022-12-25 07:00:00+00\",\"2022-12-28 03:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-666, [\"2022-12-25 07:00:00+00\",\"2022-12-28 03:00:00+00\"))."
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression
        let re = Regex::new(
            r#"\((?P<k1>[a-zA-Z0-9-_]+),\s*(?P<k2>[a-zA-Z0-9-_]+)\)=\((?P<v1>[a-zA-Z0-9-_]+),\s*\[(?P<v2>[^\)\]]+)"#,
        ).unwrap();
        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map));
        }

        if maps.len() != 2 {
            return Err(());
        }

        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-666, [\"2022-12-25 07:00:00+00\",\"2022-12-28 03:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-666, [\"2022-12-25 07:00:00+00\",\"2022-12-28 03:00:00+00\")).";

    #[test]
    fn parsed_datetime_should_work() {
        let dt = parse_datetime("2022-12-26 22:00:00+00").unwrap();
        println!("dt: {}", dt);
        assert_eq!(dt.to_rfc3339(), "2022-12-26T22:00:00+00:00");
    }

    #[test]
    fn parsed_info_should_work() {
        let info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(info.new["resource_id"], "ocean-view-room-666");
    }

    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-666".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2022-12-25 07:00:00+00\",\"2022-12-28 03:00:00+00\"".to_string(),
        );
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-666");
        assert_eq!(window.start.to_rfc3339(), "2022-12-25T07:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2022-12-28T03:00:00+00:00");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let info: ReservationConflictInfo = ERR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-666");
                assert_eq!(conflict.old.rid, "ocean-view-room-666");
                assert_eq!(conflict.old.start.to_rfc3339(), "2022-12-25T07:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("should be parsed"),
        }
    }
}
