use chrono::{DateTime, Utc};

use crate::DISCORD_EPOCH;

/// A snowflake id.
/// 
/// EPOCH is in milliseconds as an offset from the Unix epoch.
/// 
/// Snowflakes are just a `u64` internally, and can more or less be treated like an opaque number:
/// you can compare them, hash them, etc. Sorting an array of snowflakes will sort them roughly by when
/// they were generated, even if they were generated on different machines.
/// 
/// ## Bit ranges
/// 
/// | Bit range | Number of bits | Mask        | Contents                    |
/// |-----------|----------------|-------------|-----------------------------|
/// | 63-22     | 42             | `!0x3FFFFF` | epoch-relative timestamp    |
/// | 21-12     | 10             | `0x3FF000`  | unique id                   |
/// | 11-0      | 12             | `0xFFF`     | intra-millisecond increment |
/// 
/// Discord and Twitter both split the unique id into two 5-bit fields
/// (worker id and process id for Discord and datacenter id and worker id for Twitter).
/// This crate is not opinionated about server architecture,
/// and as such does not make assumptions about how you use the 10 bits of `unique_id` so long as it is unique,
/// but does provide `worker_id` and `process_id` methods consistent with Discord's terminology for convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Snowflake<const EPOCH: u64 = DISCORD_EPOCH>(
    #[cfg_attr(feature = "serde", serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string"))]
    u64
);

impl<const EPOCH: u64> Snowflake<EPOCH> {
    /// Gets the snowflake's embedded timestamp, relative to `EPOCH`.
    /// 
    /// Note that all epoch information is lost in this conversion,
    /// so make sure you're converting it yourself.
    /// [`timestamp_unix`](Snowflake::timestamp_unix) 
    /// and [`time`](Snowflake::time)
    /// provide ways of reliably extracting the absolute date and time.
    pub fn timestamp(self) -> u64 {
        self.0 >> 22
    }

    /// Gets the snowflake's embedded timestamp as a standard Unix timestamp relative to the Unix epoch, in milliseconds.
    pub fn timestamp_unix(self) -> u64 {
        self.timestamp().checked_add(EPOCH).expect("Should not happen for 100 years!")
    }

    /// Gets the snowflake's embedded timestamp as a `DateTime` from the [`chrono`](https://docs.rs/chrono) crate.
    pub fn time(self) -> Result<DateTime<Utc>, InvalidTimestampError> {
        DateTime::from_timestamp_millis(
                self.timestamp_unix()
                .try_into()
                .map_err(|_| InvalidTimestampError(self.timestamp_unix()))?
            )
            .ok_or(InvalidTimestampError(self.0))
    }

    /// Gets the unique id of the [`SnowflakeGenerator`](crate::SnowflakeGenerator) that generated this snowflake.
    pub fn unique_id(self) -> u16 {
        ((self.0 & 0x3FF000) >> 12) as u16
    }

    /// Gets the upper 5 bits of the unique id, consistent with Discord's worker id.
    /// Provided for convenience.
    pub fn worker_id(self) -> u8 {
        ((self.0 & 0x3E0000) >> 17) as u8
    }

    /// Gets the lower 5 bits of the unique id, consistent with Discord's process id.
    /// Provided for convenience.
    pub fn process_id(self) -> u8 {
        ((self.0 & 0x1F000) >> 12) as u8
    }

    /// Gets the intra-millisecond increment of this snowflake.
    /// 
    /// The only purpose of these bits is to ensure uniqueness on the same generator,
    /// so there's not much use for isolating them, but it's here for completeness.
    pub fn increment(self) -> u16 {
        (self.0 & 0xFFF) as u16
    }

    /// Returns the epoch that this snowflake's timestamp is relative to.
    /// 
    /// This information is encoded in a const generic parameter,
    /// so there aren't many scenarios where this won't be known until runtime.
    pub fn epoch(self) -> u64 {
        EPOCH
    }

    /// Gets the full inner `u64` value of this snowflake.
    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl<const EPOCH: u64> From<u64> for Snowflake<EPOCH> {
    fn from(value: u64) -> Self {
        Snowflake(value)
    }
}

impl<const EPOCH: u64> std::str::FromStr for Snowflake<EPOCH> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Snowflake(s.parse()?))
    }
}

#[cfg(feature = "serde")]
impl<const EPOCH: u64> serde::Serialize for Snowflake<EPOCH> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&format!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("invalid snowflake timestamp {0}")]
pub struct InvalidTimestampError(u64);

#[cfg(test)]
mod tests {
    use super::*;

    const RAND_TEST_ITERATIONS: usize = 1000;

    #[cfg(feature = "serde")]
    mod serde {
        use rand::Rng;

        use super::*;

        #[test]
        fn serialize_json() {
            let mut rng = rand::rng();
            for _ in 0..RAND_TEST_ITERATIONS {
                let x = rng.next_u64();
                let flake: Snowflake = Snowflake(x);
                let json = serde_json::to_string(&flake).unwrap();
                assert_eq!(
                    format!("\"{}\"", x),
                    json
                );
            }
        }

        #[test]
        fn deserialize_json() {
            let mut rng = rand::rng();
            for _ in 0..RAND_TEST_ITERATIONS {
                let x = rng.next_u64();
                let json = format!("\"{}\"", x);
                let flake: Snowflake = serde_json::from_str(&json).unwrap();
                assert_eq!(
                    x,
                    flake.into_inner()
                );
            }
        }

        #[test]
        fn roundtrip_json() {
            let mut rng = rand::rng();
            for _ in 0..RAND_TEST_ITERATIONS {
                let flake1: Snowflake = Snowflake(rng.next_u64());
                let json = serde_json::to_string(&flake1).unwrap();
                let flake2: Snowflake = serde_json::from_str(&json).unwrap();
                assert_eq!(flake1, flake2);
            }
        }
    }
}