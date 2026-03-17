use std::sync::Mutex;
use std::time::Duration;
use std::thread::sleep;

use chrono::Utc;

use crate::{Snowflake, DISCORD_EPOCH};

#[cfg(feature = "tracing")]
use tracing::{instrument, event, Level};

/// A generator for Discord-style snowflakes.
/// 
#[derive(Debug)]
pub struct SnowflakeGenerator<const EPOCH: u64 = DISCORD_EPOCH> {
    // 10 bits, represents the worker and process id of Discord's snowflakes
    unique_id: u64,
    state_mutex: Mutex<CounterState>
}

impl<const EPOCH: u64> SnowflakeGenerator<EPOCH> {
    pub fn new(unique_id: u16) -> SnowflakeGenerator<EPOCH> {
        SnowflakeGenerator {
            unique_id: (unique_id & 0x3FF) as u64,
            state_mutex: Mutex::new(CounterState {
                counter: 0,
                last_timestamp: 0
            })
        }
    }

    pub fn from_worker_and_process_ids(worker_id: u8, process_id: u8) -> SnowflakeGenerator<EPOCH> {
        SnowflakeGenerator::new(((worker_id & 0b11111) as u16) << 5 | ((process_id & 0b11111) as u16))
    }

    #[cfg_attr(feature = "tracing", instrument(name = "SnowflakeGenerator::generate", level="trace"))]
    pub fn generate(&self) -> Snowflake<EPOCH> {
        let (timestamp, counter) = {
            let mut state = match self.state_mutex.lock() {
                Ok(state) => state,
                Err(err) => {
                    #[cfg(feature = "tracing")]
                    event!(Level::WARN,
                        "Snowflake counter mutex was poisoned. \
                        This should not happen, so if you see this report it. \
                        Snowflake generation will still function, but this might cause slowdowns."
                    );

                    let mut poisoned = err.into_inner();
                    poisoned.reset();
                    self.state_mutex.clear_poison();
                    poisoned
                }
            };

            let mut timestamp = epoch_timestamp_millis(EPOCH);

            if timestamp < state.last_timestamp {
                #[cfg(feature = "tracing")]
                event!(Level::WARN,
                    "Unix timestamp has gone backwards.\
                    This is likely a bug in either this crate or chrono.\
                    Snowflake generation will still function."
                );

                // time has moved backwards, somehow
                // if this happens it's probably an issue with chrono,
                // but regardless the best way to handle it is a bit of latency
                timestamp = wait_for_next_ms(timestamp, state.last_timestamp - 1, EPOCH);
            }
            
            if timestamp == state.last_timestamp {
                // counter wraps at 4096
                state.counter = (state.counter + 1) & 0xFFF;
                // if it does, we need to wait until the next ms to continue generating ids
                if state.counter == 0 {
                    #[cfg(feature = "tracing")]
                    event!(Level::DEBUG,
                        "Snowflake counter has rolled over, introducing a slight delay"
                    );

                    timestamp = wait_for_next_ms(timestamp, state.last_timestamp, EPOCH);
                }
            } else {
                // we've rolled over to a new ms
                state.counter = 0;
            }

            state.last_timestamp = timestamp;

            (timestamp, state.counter)
        };

        let flake = Snowflake::from(
            (timestamp << 22)
            | (self.unique_id << 12)
            | (counter & 0xFFF)
        );

        #[cfg(feature = "tracing")]
        event!(Level::TRACE, timestamp, unique_id = self.unique_id, counter, epoch = EPOCH, flake = flake.into_inner(), "Generated snowflake");

        flake
    }
}

#[derive(Debug, Clone, Copy)]
struct CounterState {
    counter: u64,
    last_timestamp: u64
}

impl CounterState {
    fn reset(&mut self) {
        // The only way this would end up being called is if Utc::now() panics,
        // so it's fine if it's a little unoptimal
        sleep(Duration::from_millis(1));
        self.counter = 0;
        self.last_timestamp = 0;
    }
}

fn epoch_timestamp_millis(epoch: u64) -> u64 {
    Utc::now().timestamp_millis() as u64 - epoch
}

fn wait_for_next_ms(mut current: u64, target: u64, epoch: u64) -> u64 {
    while current <= target {
        // Wait in 100-us increments, should hopefully strike a decent balance
        // between spinning too much and waiting longer than necessary
        sleep(Duration::from_micros(100));
        current = epoch_timestamp_millis(epoch);
    }
    current
}