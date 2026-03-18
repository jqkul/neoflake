# neoflake

A Rust crate for generating Twitter/Discord-style 64-bit snowflake ids quickly and scalably

### Why this crate?
There are a couple other crates
([`twitter_snowflake`](https://crates.io/crates/twitter_snowflake),
[`flake`](https://crates.io/crates/flake))
that provide similar functionality, but neither of them were quite what I wanted, so I made my own.
You should use whichever one meets your needs best.

Here's what `neoflake` offers:
- A generator that can be shared across threads or made thread-local
- A smart `Snowflake` type with accessor methods for its bit fields
- A type-level guarantee of what epoch a snowflake's timestamp is relative to, using const generics


## Getting started

Run `cargo add neoflake` to add the latest version to your project's dependencies.
Then, either instantiate a `SnowflakeGenerator` with a unique id:

```rust
use neoflake::SnowflakeGenerator;

fn main() {
    let unique_id: u16 = 47;
    let snow_machine = SnowflakeGenerator::new(unique_id);
    let flake1 = snow_machine.generate();
    let flake2 = snow_machine.generate();
}
```

Or use the `setup` macro to create a handy type alias and a static global generator to be shared across threads:

```rust
use neoflake::setup;
const UNIQUE_ID: u16 = 47;
const MY_CUSTOM_EPOCH: u64 = 1767225600000;
setup!(MyFlake, SNOW_MACHINE, MY_CUSTOM_EPOCH, UNIQUE_ID);

fn main() {
    let flake1: MyFlake = SNOW_MACHINE.generate();
    let flake2: MyFlake = SNOW_MACHINE.generate();
}
```