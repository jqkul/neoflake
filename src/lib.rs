/// The epoch used by Discord's snowflake ids.
pub const DISCORD_EPOCH: u64 = 1420070400000;

/// The epoch used by Twitter's snowflake ids.
/// 
/// Based on the
/// [publicly available](https://github.com/twitter-archive/snowflake/blob/snowflake-2010/src/main/scala/com/twitter/service/snowflake/IdWorker.scala)
/// 2010 implementation.
pub const TWITTER_EPOCH: u64 = 1288834974657;

mod snowflake;
mod generator;

pub use snowflake::Snowflake;
pub use generator::SnowflakeGenerator;