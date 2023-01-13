# rust-bsor

[BS Open Replay](https://github.com/BeatLeader/BS-Open-Replay) Rust parser

Disclaimer: This is my Rust learning project, so expect bugs and non-idomatic code

## Known limitations

Version 0.1.1 does not support replays saved out of specification by a very old version of the Beat Leader mod (incorrect utf8 string encoding).

## Install

Run following Cargo command in your project directory

```sh
cargo add bsor
```

Or add the following line to your ``[dependencies]`` section of the ``Cargo.toml``:

```toml
bsor = "0.2.0"
```

## Usage

Loading the entire replay into memory:

```rust
use bsor::replay::Replay;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut br = BufReader::new(File::open("example.bsor").unwrap());

    let replay = Replay::load(&mut br).unwrap();

    println!("{:#?}", replay);
}
```

Since you may rarely need the full replay structure (especially Frames) and at the same time would like to keep memory usage low, there is also the option of loading only selected blocks (keep in mind that Header and Info blocks are always loaded).

```rust
use bsor::replay::{ParsedReplay, CouldLoadBlock};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut br = BufReader::new(File::open("example.bsor").unwrap());

    let parsed_replay = ParsedReplay::parse(br).unwrap();

    let notes = parsed_replay.notes.load(br).unwrap();
    println!(
        "Info: {:#?}\nNotes count: {:#?}",
        parsed_replay.info,
        notes.len()
    );
    if !notes.is_empty() {
        println!("{:#?}", notes.get_vec()[notes.len() / 2]);
    }
}

```

The memory savings can be significant, for example, for an average replay of 1375kB:

| Block         | Memory usage |
|---------------|--------------|
| Whole replay  | 1383kB       |
| Header + Info | 9kB          |
| Frames        | 1255kB       |
| Notes         | 137kB        |
