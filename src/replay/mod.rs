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

pub use error::BsorError;
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

pub type ReplayInt = i32;
pub type ReplayLong = u64;
pub type ReplayFloat = f32;
pub type ReplayTime = ReplayFloat;
pub type LineValue = u8;

pub type Result<T> = std::result::Result<T, BsorError>;

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

pub struct ParsedReplay {
    pub version: u8,
    pub info: Info,
    pub frames: ParsedReplayBlock<Frames>,
    pub notes: ParsedReplayBlock<Notes>,
    pub walls: ParsedReplayBlock<Walls>,
    pub heights: ParsedReplayBlock<Heights>,
    pub pauses: ParsedReplayBlock<Pauses>,
}

impl ParsedReplay {
    pub fn parse<RS: Read + Seek>(r: &mut RS) -> Result<ParsedReplay> {
        let header = Header::load(r)?;
        let info = Info::load(r)?;

        let frames_pos = r.stream_position()?;
        let frames = Frames::get_total_block_size(r, frames_pos)?;

        let notes_pos = frames_pos + frames.bytes;

        r.seek(SeekFrom::Start(notes_pos))?;
        let notes = Notes::get_total_block_size(r, notes_pos)?;

        let walls_pos = notes_pos + notes.bytes;
        r.seek(SeekFrom::Start(walls_pos))?;
        let walls = Walls::get_total_block_size(r, walls_pos)?;

        let heights_pos = walls_pos + walls.bytes;
        r.seek(SeekFrom::Start(heights_pos))?;
        let heights = Heights::get_total_block_size(r, heights_pos)?;

        let pauses_pos = heights_pos + heights.bytes;
        r.seek(SeekFrom::Start(pauses_pos))?;
        let pauses = Pauses::get_total_block_size(r, pauses_pos)?;

        Ok(ParsedReplay {
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

#[derive(Debug)]
pub struct ParsedReplayBlock<T> {
    ///! position in stream
    pos: u64,
    ///! block length in bytes
    bytes: u64,
    ///! sub items count
    items_count: i32,
    _phantom: PhantomData<T>,
}

impl<T> ParsedReplayBlock<T> {
    pub fn pos(&self) -> u64 {
        self.pos
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn len(&self) -> i32 {
        self.items_count
    }

    pub fn is_empty(&self) -> bool {
        self.items_count == 0
    }
}

trait HasStaticBlockSize {
    /// Static block size in bytes (if determinable without reading the replay)
    fn get_static_size() -> usize;
}

trait CouldLoadBlockSize {
    type Item: HasStaticBlockSize;

    /// Total block size (includes static size)
    fn get_total_block_size<RS: Read + Seek>(
        _r: &mut RS,
        pos: u64,
    ) -> Result<ParsedReplayBlock<Self::Item>> {
        Ok(ParsedReplayBlock::<Self::Item> {
            pos,
            bytes: Self::Item::get_static_size() as u64,
            items_count: 0,
            _phantom: PhantomData,
        })
    }
}

pub trait CouldLoadBlock {
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
