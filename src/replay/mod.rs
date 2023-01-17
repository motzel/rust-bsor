//! A module for loading a bsor file in whole or in parts
//!
//! # Examples
//! Loading the entire replay file into memory:
//! ```no_run
//! use bsor::prelude::*;
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let br = &mut BufReader::new(File::open("example.bsor").unwrap());
//! let replay = Replay::load(br).unwrap();
//! println!("{:#?}", replay);
//! ```
//!
//! Since you may rarely need the full replay structure (especially the largest Frames block, if you do not want to display the replay) and at the same time would like to keep memory usage low, there is also the option of loading only selected blocks (keep in mind that Header and Info blocks are always loaded)
//!
//! In this case, the process is a two-step one. In the first step, the replay must be indexed, and then the individual blocks can be loaded into memory.
//!
//! Note: Unlike [Replay::load()](replay/struct.Replay.html#method.load), which requires any [std::io::Read] reader as an argument, [ReplayIndex::index()](replay/struct.ReplayIndex.html#method.index) requires [std::io::Read] + [std::io::Seek] reader
//!
//! ```no_run
//! use bsor::prelude::*;
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let mut br = &mut BufReader::new(File::open("example.bsor").unwrap());
//!
//! let replay_index = ReplayIndex::index(br).unwrap();
//!
//! let notes = replay_index.notes.load(br).unwrap();
//! println!(
//!     "Info: {:#?}\nNotes count: {:#?}",
//!     replay_index.info,
//!     notes.len()
//! );
//! if !notes.is_empty() {
//!     println!("{:#?}", notes[notes.len() / 2]);
//! }
//! ```
//!
//! The memory savings can be significant, for example, for an **average replay of 1375kB**:
//!
//! | Block         | Memory usage |
//! |---------------|--------------|
//! | Whole replay  | 1383kB       |
//! | Header + Info | 9kB          |
//! | Frames        | 1255kB       |
//! | Notes         | 137kB        |
//!
pub mod error;
pub mod frame;
mod header;
pub mod height;
pub mod info;
pub mod note;
pub mod pause;
mod read_utils;
pub mod vector;
pub mod wall;

use error::BsorError;
use frame::Frames;
use header::Header;
use height::Heights;
use info::Info;
use note::Notes;
use pause::Pauses;
use std::io::Seek;
use std::io::{Read, SeekFrom};
use std::marker::PhantomData;
use wall::Walls;

pub(crate) const BSOR_MAGIC: i32 = 0x442d3d69;

/// int type used in replay file
pub type ReplayInt = i32;
/// long type used in replay file
pub type ReplayLong = u64;
/// float type used in replay file
pub type ReplayFloat = f32;
/// time type used in replay file
pub type ReplayTime = ReplayFloat;
/// type used to store note line index
pub type LineIdx = u8;
/// type used to store note line layer
pub type LineLayer = u8;

/// This type is broadly used across the crate for any operation which may produce an error
pub type Result<T> = std::result::Result<T, BsorError>;

/// Basic crate struct corresponding to the structure of the bsor file
#[derive(Debug)]
pub struct Replay {
    pub version: u8,
    pub info: Info,
    pub frames: Frames,
    pub notes: Notes,
    pub walls: Walls,
    pub heights: Heights,
    pub pauses: Pauses,
}

impl Replay {
    /// Load replay into memory
    pub fn load<R: Read>(r: &mut R) -> Result<Replay> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;
        let frames = Frames::load(r)?;
        let notes = Notes::load(r)?;
        let walls = Walls::load(r)?;
        let heights = Heights::load(r)?;
        let pauses = Pauses::load(r)?;

        Ok(Replay {
            version: header.version,
            info,
            frames,
            notes,
            walls,
            heights,
            pauses,
        })
    }
}

/// Replay index needed to load individual blocks
pub struct ReplayIndex {
    pub version: u8,
    pub info: Info,
    pub frames: BlockIndex<Frames>,
    pub notes: BlockIndex<Notes>,
    pub walls: BlockIndex<Walls>,
    pub heights: BlockIndex<Heights>,
    pub pauses: BlockIndex<Pauses>,
}

impl ReplayIndex {
    /// Indexes replay, so you can easily load each block individually
    pub fn index<RS: Read + Seek>(r: &mut RS) -> Result<ReplayIndex> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;

        let frames_pos = r.stream_position()?;
        let frames = Frames::load_real_block_size(r, frames_pos)?;

        let notes_pos = frames_pos + frames.bytes;

        r.seek(SeekFrom::Start(notes_pos))?;
        let notes = Notes::load_real_block_size(r, notes_pos)?;

        let walls_pos = notes_pos + notes.bytes;
        r.seek(SeekFrom::Start(walls_pos))?;
        let walls = Walls::load_real_block_size(r, walls_pos)?;

        let heights_pos = walls_pos + walls.bytes;
        r.seek(SeekFrom::Start(heights_pos))?;
        let heights = Heights::load_real_block_size(r, heights_pos)?;

        let pauses_pos = heights_pos + heights.bytes;
        r.seek(SeekFrom::Start(pauses_pos))?;
        let pauses = Pauses::load_real_block_size(r, pauses_pos)?;

        Ok(ReplayIndex {
            version: header.version,
            info,
            frames,
            notes,
            walls,
            heights,
            pauses,
        })
    }
}

/// Struct storing index data about each block
#[derive(Debug)]
pub struct BlockIndex<T> {
    ///! position in stream
    pos: u64,
    ///! block length in bytes
    bytes: u64,
    ///! sub items count
    items_count: i32,
    _phantom: PhantomData<T>,
}

impl<T> BlockIndex<T> {
    /// Returns block start position in the stream
    pub fn pos(&self) -> u64 {
        self.pos
    }

    /// Returns block size in bytes
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Returns underlying items count
    pub fn len(&self) -> i32 {
        self.items_count
    }

    /// Returns whether there are any underlying items
    pub fn is_empty(&self) -> bool {
        self.items_count == 0
    }
}

trait GetStaticBlockSize {
    /// Static block size in bytes (if determinable without reading the replay)
    fn get_static_size() -> usize;
}

trait LoadRealBlockSize {
    type Item: GetStaticBlockSize;

    /// Real block size (includes static size)
    fn load_real_block_size<RS: Read + Seek>(
        _r: &mut RS,
        pos: u64,
    ) -> Result<BlockIndex<Self::Item>> {
        Ok(BlockIndex::<Self::Item> {
            pos,
            bytes: Self::Item::get_static_size() as u64,
            items_count: 0,
            _phantom: PhantomData,
        })
    }
}

/// Trait to load individual blocks into memory based on indexed data
pub trait LoadBlock {
    type Item;

    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item>;
}

pub(crate) enum BlockType {
    Info = 0,
    Frames,
    Notes,
    Walls,
    Heights,
    Pauses,
}

impl TryInto<u8> for BlockType {
    type Error = BsorError;

    fn try_into(self) -> std::result::Result<u8, Self::Error> {
        Ok(self as u8)
    }
}

fn assert_start_of_block<R: Read>(r: &mut R, bt: BlockType) -> Result<()> {
    match read_utils::read_byte(r) {
        Ok(v) => {
            if v != bt.try_into()? {
                Err(BsorError::InvalidBsor)
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_util::{generate_random_replay, get_replay_buffer};
    use std::io::Cursor;

    #[test]
    fn it_can_load_replay() -> Result<()> {
        let replay = generate_random_replay();

        let buf = get_replay_buffer(&replay)?;

        let result = Replay::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result.version, replay.version);
        assert_eq!(result.info, replay.info);
        assert_eq!(result.frames, replay.frames);
        assert_eq!(result.notes, replay.notes);
        assert_eq!(result.walls, replay.walls);
        assert_eq!(result.heights, replay.heights);
        assert_eq!(result.pauses, replay.pauses);

        Ok(())
    }

    #[test]
    fn it_can_index_replay() -> Result<()> {
        let replay = generate_random_replay();

        let buf = get_replay_buffer(&replay)?;

        let reader = &mut Cursor::new(buf);
        let result = ReplayIndex::index(reader)?;

        assert_eq!(result.version, replay.version);
        assert_eq!(result.info, replay.info);
        assert_eq!(result.frames.len(), replay.frames.len() as i32);
        assert_eq!(result.notes.len(), replay.notes.len() as i32);
        assert_eq!(result.walls.len(), replay.walls.len() as i32);
        assert_eq!(result.heights.len(), replay.heights.len() as i32);
        assert_eq!(result.pauses.len(), replay.pauses.len() as i32);

        Ok(())
    }
}
