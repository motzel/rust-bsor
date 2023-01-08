/// Read and parse BS Open Replay (bsor) files
///
/// <https://github.com/BeatLeader/BS-Open-Replay>
///
/// # Examples
/// ```no_run
/// use bsor::replay::Replay;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let br = &mut BufReader::new(File::open("example.bsor").unwrap());
/// let replay = Replay::load(br).unwrap();
/// println!("{:#?}", replay);
/// ```
pub mod replay;
