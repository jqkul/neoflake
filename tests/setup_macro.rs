use chrono::{DateTime, TimeDelta, Utc};

const MY_EPOCH: u64 = 1767225600000;

global_snowflake::setup!(Snowflake, MY_EPOCH, SNOWFLAKE_GEN, 0);

fn main() {
    let epoch_dt = DateTime::from_timestamp_millis(MY_EPOCH.try_into().unwrap()).unwrap();
    let expected_difference = Utc::now() - epoch_dt;
    for _ in 0..10 {
        let flake = SNOWFLAKE_GEN.generate();
        let ts = flake.timestamp_millis_unix();
        let dt = DateTime::from_timestamp_millis(ts.try_into().unwrap()).unwrap();

        assert!((dt - epoch_dt) - expected_difference < TimeDelta::seconds(5));
    }
}