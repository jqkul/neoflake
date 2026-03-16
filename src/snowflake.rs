use chrono::{DateTime, Utc};

use crate::DISCORD_EPOCH;

/// A Discord-style snowflake ID.
/// 
/// EPOCH is in milliseconds, relative to the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snowflake<const EPOCH: u64 = DISCORD_EPOCH>(u64);

impl<const EPOCH: u64> Snowflake<EPOCH> {
    pub fn timestamp_millis_custom(self) -> u64 {
        self.0 >> 22
    }

    pub fn timestamp_millis_unix(self) -> u64 {
        self.timestamp_millis_custom().checked_add(EPOCH).expect("Should not happen for 100 years!")
    }

    pub fn time(self) -> Result<DateTime<Utc>, MalformedSnowflakeError> {
        DateTime::from_timestamp_millis(self.timestamp_millis_unix() as i64)
            .ok_or(MalformedSnowflakeError(self.0))
    }

    pub fn worker_id(self) -> u8 {
        ((self.0 & 0x3E0000) >> 17) as u8
    }

    pub fn process_id(self) -> u8 {
        ((self.0 & 0x1F000) >> 12) as u8
    }

    pub fn unique_id(self) -> u16 {
        ((self.0 & 0x3FF000) >> 12) as u16
    }

    pub fn increment(self) -> u16 {
        (self.0 & 0xFFF) as u16
    }
}

impl From<u64> for Snowflake {
    fn from(value: u64) -> Self {
        Snowflake(value)
    }
}

impl std::str::FromStr for Snowflake {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Snowflake(s.parse()?))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MalformedSnowflakeError(u64);

impl std::fmt::Display for MalformedSnowflakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Malformed snowflake: {}", self.0)
    }
}

impl std::error::Error for MalformedSnowflakeError {}