use bsor::replay::Replay;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let br = &mut BufReader::new(File::open("test.bsor").unwrap());

    let replay = Replay::load(br).unwrap();

    let frames = replay.frames.get_vec();
    if frames.len() > 0 {
        println!("Count: {}, frames[0]={:#?}", frames.len(), frames[0]);
    } else {
        println!("No frames!");
    }
}
