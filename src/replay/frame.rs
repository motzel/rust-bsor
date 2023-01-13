use super::{read_utils, vector, ReplayInt, ReplayTime, Result};
use crate::replay::{
    assert_start_of_block, BlockType, CouldLoadBlock, CouldLoadBlockSize, HasStaticBlockSize,
    ParsedReplayBlock,
};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;

#[derive(Debug, PartialEq)]
pub struct Frames(Vec<Frame>);

impl Frames {
    #[cfg(test)]
    pub(crate) fn new(vec: Vec<Frame>) -> Frames {
        Frames(vec)
    }

    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Frames> {
        assert_start_of_block(r, BlockType::Frames)?;

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Frame>::with_capacity(count);

        for _ in 0..count {
            vec.push(Frame::load(r)?);
        }

        Ok(Frames(vec))
    }

    pub fn load_block<RS: Read + Seek>(
        r: &mut RS,
        block: &ParsedReplayBlock<Frames>,
    ) -> Result<Self> {
        r.seek(SeekFrom::Start(block.pos))?;

        Self::load(r)
    }

    pub fn get_vec(&self) -> &Vec<Frame> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl HasStaticBlockSize for Frames {
    fn get_static_size() -> usize {
        size_of::<u8>() + size_of::<ReplayInt>()
    }
}

impl CouldLoadBlock for ParsedReplayBlock<Frames> {
    type Item = Frames;

    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item> {
        Self::Item::load_block(r, self)
    }
}

impl CouldLoadBlockSize for Frames {
    type Item = Frames;

    fn get_total_block_size<RS: Read + Seek>(
        r: &mut RS,
        pos: u64,
    ) -> Result<ParsedReplayBlock<Frames>> {
        assert_start_of_block(r, BlockType::Frames)?;

        let count = read_utils::read_int(r)?;

        Ok(ParsedReplayBlock::<Frames> {
            pos,
            bytes: Frames::get_static_size() as u64
                + Frame::get_static_size() as u64 * count as u64,
            items_count: count,
            _phantom: PhantomData,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Frame {
    pub time: ReplayTime,
    pub fps: ReplayInt,
    pub head: PositionAndRotation,
    pub left_hand: PositionAndRotation,
    pub right_hand: PositionAndRotation,
}

impl Frame {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Frame> {
        let time = read_utils::read_float(r)?;
        let fps = read_utils::read_int(r)?;
        let head = PositionAndRotation::load(r)?;
        let left_hand = PositionAndRotation::load(r)?;
        let right_hand = PositionAndRotation::load(r)?;

        Ok(Self {
            time,
            fps,
            head,
            left_hand,
            right_hand,
        })
    }
}

impl HasStaticBlockSize for Frame {
    fn get_static_size() -> usize {
        size_of::<ReplayTime>()
            + size_of::<ReplayInt>()
            + PositionAndRotation::get_static_size() * 3
    }
}

#[derive(PartialEq, Debug)]
pub struct PositionAndRotation {
    pub position: vector::Vector3,
    pub rotation: vector::Vector4,
}

impl PositionAndRotation {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<PositionAndRotation> {
        let position = vector::Vector3::load(r)?;
        let rotation = vector::Vector4::load(r)?;

        Ok(Self { position, rotation })
    }
}

impl HasStaticBlockSize for PositionAndRotation {
    fn get_static_size() -> usize {
        vector::Vector3::get_static_size() + vector::Vector4::get_static_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::BsorError;
    use crate::tests_util::{append_frame, generate_random_frame, get_frames_buffer};
    use std::io::Cursor;

    #[test]
    fn it_returns_correct_static_size_of_frame() {
        assert_eq!(Frame::get_static_size(), 92);
    }

    #[test]
    fn it_can_load_frame() {
        let frame = generate_random_frame();

        let mut buf: Vec<u8> = Vec::new();
        append_frame(&mut buf, &frame);

        let result = Frame::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, frame)
    }

    #[test]
    fn it_returns_invalid_bsor_error_when_frames_block_id_is_invalid() -> Result<()> {
        let frames = Vec::from([generate_random_frame(), generate_random_frame()]);

        let mut buf = get_frames_buffer(&frames)?;
        buf[0] = 255;

        let result = Frames::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_frames() -> Result<()> {
        let frames = Vec::from([generate_random_frame(), generate_random_frame()]);

        let buf = get_frames_buffer(&frames)?;

        let result = Frames::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), frames);
        assert_eq!(result.is_empty(), false);
        assert_eq!(result.len(), frames.len());

        Ok(())
    }

    #[test]
    fn it_returns_correct_static_size_of_frames() {
        assert_eq!(Frames::get_static_size(), 5);
    }

    #[test]
    fn it_can_load_parsed_frames_block() -> Result<()> {
        let frames = Vec::from([generate_random_frame(), generate_random_frame()]);

        let buf = get_frames_buffer(&frames)?;

        let pos = 0;
        let reader = &mut Cursor::new(buf);
        let frames_block = Frames::get_total_block_size(reader, pos)?;

        let result = frames_block.load(reader)?;

        assert_eq!(frames_block.pos(), pos);
        assert_eq!(
            frames_block.bytes(),
            Frames::get_static_size() as u64 + Frame::get_static_size() as u64 * 2
        );
        assert_eq!(frames_block.is_empty(), false);
        assert_eq!(frames_block.len(), frames.len() as i32);
        assert_eq!(*result.get_vec(), frames);

        Ok(())
    }
}
