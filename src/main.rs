use bsor::replay::Replay;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let br = &mut BufReader::new(File::open("test.bsor").unwrap());

    let replay = Replay::load(br).unwrap();

    println!("{:#?}", replay);
}
