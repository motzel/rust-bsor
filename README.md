# rust-bsor

[BS Open Replay](https://github.com/BeatLeader/BeatSaber-Web-Replays) Rust parser

Disclaimer: This is my Rust learning project, so expect bugs and non-idomatic code

## Install

Run following Cargo command in your project directory

```sh
cargo add bsor
```

Or add the following line to your ``[dependencies]`` section of the ``Cargo.toml``:

```toml
bsor = "0.1.0"
```

## Usage

```rust
use bsor::replay::Replay;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut br = BufReader::new(File::open("test.bsor").unwrap());

    let replay = Replay::load(&mut br).unwrap();

    println!("{:#?}", replay);
}

```