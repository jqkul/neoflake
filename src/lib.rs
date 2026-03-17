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

#[cfg(feature = "setup")]
#[macro_export]
macro_rules! setup {
    ($flakename:ident, $epoch:expr, $genname:ident, $unique_id:expr) => {
        type $flakename = global_snowflake::Snowflake<{$epoch}>;
        lazy_static::lazy_static! {
            pub static ref $genname: global_snowflake::SnowflakeGenerator<{$epoch}> = global_snowflake::SnowflakeGenerator::new($unique_id);
        }
    };
}