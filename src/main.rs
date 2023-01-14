use bsor::prelude::*;
use std::fs::File;
use std::io::BufReader;

fn main() {
    {
        let br = &mut BufReader::new(File::open("example.bsor").unwrap());
        let replay = Replay::load(br).unwrap();
        println!("{:#?}", replay.info);
    }

    {
        let br = &mut BufReader::new(File::open("example.bsor").unwrap());
        let parsed_replay = ParsedReplay::parse(br).unwrap();
        let notes = parsed_replay.notes.load(br).unwrap();

        if !notes.is_empty() {
            let notes_count = notes.len();
            let idx = notes_count / 2;
            println!(
                "Info: {:#?}\nNote[{}] = {:#?}",
                parsed_replay.info,
                idx,
                notes.get_vec()[idx]
            );
        } else {
            println!("Replay contains no notes 🤔");
        }
    }
}
