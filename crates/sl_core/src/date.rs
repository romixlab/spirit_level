//! Calendar dates without a date crate, and the expiry of an override: a date or a release version.

use serde::Serialize;
use std::fmt;

/// A calendar date (proleptic Gregorian), ordered
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Date {
    /// Year
    pub year: i32,
    /// 1-12
    pub month: u8,
    /// 1-31
    pub day: u8,
}

impl Date {
    /// Parses `YYYY-MM-DD`, checking the day exists
    pub fn parse(s: &str) -> Option<Date> {
        let mut it = s.split('-');
        let (y, m, d) = (it.next()?, it.next()?, it.next()?);
        if it.next().is_some() || y.len() != 4 || m.len() != 2 || d.len() != 2 {
            return None;
        }
        let date = Date {
            year: y.parse().ok()?,
            month: m.parse().ok()?,
            day: d.parse().ok()?,
        };
        ((1..=12).contains(&date.month)
            && date.day >= 1
            && date.day <= days_in_month(date.year, date.month))
        .then_some(date)
    }

    /// The date of a count of days since 1970-01-01 (Howard Hinnant's civil-from-days)
    pub fn from_days(days: i64) -> Date {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
    }

    /// Today in UTC from the system clock. Callers that must be deterministic (tests, the engine) take the date
    /// as a parameter instead and call this only at the edge (the CLI)
    pub fn today_utc() -> Date {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        Date::from_days(secs.div_euclid(86_400))
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        _ => 28,
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A release version `MAJOR.MINOR.PATCH`, ordered (pre-release and build parts are not supported in expiries)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    /// Parses `1.2.3` or `v1.2.3`
    pub fn parse(s: &str) -> Option<Version> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let mut it = s.split('.').map(|p| p.parse::<u64>().ok());
        let v = Version(it.next()??, it.next()??, it.next()??);
        it.next().is_none().then_some(v)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// When an override or a waiver stops holding
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Expiry {
    /// On this date (expired from that day on)
    Date(Date),
    /// At this release of the project (expired once the project's version reaches it)
    Version(Version),
}

impl Expiry {
    /// Parses `2027-01-31`, `v0.5.0` or `0.5.0`
    pub fn parse(s: &str) -> Option<Expiry> {
        Date::parse(s)
            .map(Expiry::Date)
            .or_else(|| Version::parse(s).map(Expiry::Version))
    }

    /// Whether it has expired on `today` for a project at `version` (a version expiry without a project
    /// version never expires, and the configuration check says so)
    pub fn expired(&self, today: Date, version: Option<Version>) -> bool {
        match self {
            Expiry::Date(d) => today >= *d,
            Expiry::Version(v) => version.is_some_and(|p| p >= *v),
        }
    }
}

impl fmt::Display for Expiry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expiry::Date(d) => d.fmt(f),
            Expiry::Version(v) => write!(f, "v{v}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dates() {
        assert_eq!(
            Date::parse("2026-11-13"),
            Some(Date {
                year: 2026,
                month: 11,
                day: 13
            })
        );
        assert_eq!(Date::parse("2026-02-29"), None);
        assert!(Date::parse("2028-02-29").is_some());
        assert_eq!(Date::parse("2026-13-01"), None);
        assert_eq!(Date::parse("26-01-01"), None);
    }

    #[test]
    fn days() {
        assert_eq!(Date::from_days(0).to_string(), "1970-01-01");
        assert_eq!(Date::from_days(20_405).to_string(), "2025-11-13");
    }

    #[test]
    fn expiry() {
        let today = Date::parse("2026-11-13").unwrap();
        assert!(Expiry::parse("2026-11-13").unwrap().expired(today, None));
        assert!(!Expiry::parse("2026-11-14").unwrap().expired(today, None));
        let e = Expiry::parse("v0.5.0").unwrap();
        assert!(!e.expired(today, Some(Version(0, 4, 9))));
        assert!(e.expired(today, Some(Version(0, 5, 0))));
        assert!(!e.expired(today, None));
    }
}
