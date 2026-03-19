//! A crate for generating Discord- and Twitter-style unique IDs at speed and scale.
//! 
//! [`Snowflake`]s are guaranteed to be unique across multiple workers with no synchronization,
//! as long as no two [`SnowflakeGenerator`]s have the same unique_id.
//! `Snowflake` values increase monotonically; flakes generated later with the same generator are guaranteed to be larger, 
//! and a flake generated on machine A at least a few milliseconds later than another flake generated on machine B should be larger.
//! 
//! The maximum rate of snowflakes that can be generated with one generator per millisecond is 4096.
//! This is a direct consequence of the bit layout;
//! the counter that ensures flakes generated in the same millisecond are unique only has 10 bits to work with,
//! so if the generator would go faster it has to pause briefly to wait for the next millisecond.
//! 
//! ## Basic example
//!
//! ```
//! use neoflake::SnowflakeGenerator;
//! 
//! let unique_id: u16 = 47;
//! let snow_machine: SnowflakeGenerator = SnowflakeGenerator::new(unique_id);
//! let flake1 = snow_machine.generate();
//! let flake2 = snow_machine.generate();
//! 
//! assert!(flake1 < flake2);
//! assert_eq!(flake1.unique_id(), 47);
//! assert_eq!(flake1.unique_id(), flake2.unique_id());
//! ```
//! 
//! ## Features
//! The `setup` feature enables the [`setup`] macro.
//! It is enabled by default, but can be removed;
//! the main reason you might want to do this is to remove the dependency on `lazy_static`.
//! 
//! The `serde` feature implements [`serde`](https://docs.rs/serde)'s
//! `Serialize` and `Deserialize` for `Snowflake`.
//! 
//! The `tracing` feature enables hooks for the [`tracing`](https://docs.rs/tracing) crate.

/// The epoch used by Discord's snowflake ids.
/// 
/// Value obtained from the
/// [Discord API reference](https://docs.discord.com/developers/reference#snowflakes).
pub const DISCORD_EPOCH: u64 = 1420070400000;

/// The epoch used by Twitter's snowflake ids.
/// 
/// Value obtained from the 
/// [publicly available 2010 implementation](https://github.com/twitter-archive/snowflake/blob/snowflake-2010/src/main/scala/com/twitter/service/snowflake/IdWorker.scala).
pub const TWITTER_EPOCH: u64 = 1288834974657;

/// The [Unix epoch](https://en.wikipedia.org/wiki/Unix_time).
pub const UNIX_EPOCH: u64 = 0;

mod snowflake;
mod generator;

pub use snowflake::{Snowflake, InvalidTimestampError};
pub use generator::SnowflakeGenerator;

/// A macro to automate a common use case.
/// 
/// Creates a type alias for a `Snowflake` with a specific `EPOCH` value,
/// and creates a global snowflake generator with the same `EPOCH` wrapped in a
/// [`lazy_static`](https://docs.rs/lazy_static).
/// This is useful when you want all threads in your application to share one generator
/// and not have to pass anything around.
/// `SnowflakeGenerator` uses a `Mutex` internally, so it's thread-safe.
/// 
/// ## Example
/// 
/// ```rust
/// use neoflake::setup;
/// 
/// const UNIQUE_ID: u16 = 47;
/// const MY_CUSTOM_EPOCH: u64 = 1767225600000;
/// setup!(MyFlake, SNOW_MACHINE, MY_CUSTOM_EPOCH, UNIQUE_ID);
///
/// fn main() {
///     let flake1: MyFlake = SNOW_MACHINE.generate();
///     let flake2: MyFlake = SNOW_MACHINE.generate();
///     assert_eq!(flake1.epoch(), MY_CUSTOM_EPOCH);
/// }
/// ```
/// 
/// ## Expansion
/// ```
/// # use neoflake::setup;
/// # const MY_CUSTOM_EPOCH: u64 = 1;
/// # const UNIQUE_ID: u16 = 7;
/// setup!(MySnowflake, FLAKE_GEN, MY_CUSTOM_EPOCH, UNIQUE_ID);
/// ```
/// expands to:
/// ```
/// # use neoflake::{Snowflake, SnowflakeGenerator};
/// # use lazy_static::lazy_static;
/// # const MY_CUSTOM_EPOCH: u64 = 1;
/// # const UNIQUE_ID: u16 = 7;
/// type MySnowflake = Snowflake<MY_CUSTOM_EPOCH>;
/// lazy_static! {
///     pub static ref FLAKE_GEN: SnowflakeGenerator<MY_CUSTOM_EPOCH> = SnowflakeGenerator::new(UNIQUE_ID);
/// }
/// ```
#[cfg(feature = "setup")]
#[macro_export]
macro_rules! setup {
    ($snowflake_type_name:ident, $global_generator_name:ident, $epoch:expr, $unique_id:expr) => {
        type $snowflake_type_name = neoflake::Snowflake<{$epoch}>;
        neoflake::__lazy_static! {
            pub static ref $global_generator_name: neoflake::SnowflakeGenerator<{$epoch}> = neoflake::SnowflakeGenerator::new($unique_id);
        }
    };
}

#[cfg(feature = "setup")]
#[doc(hidden)]
pub use lazy_static::lazy_static as __lazy_static;