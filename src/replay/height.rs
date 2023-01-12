use super::{read_utils, BsorError, ReplayTime, Result};
use crate::replay::{
    assert_start_of_block, BlockType, CouldLoadBlock, CouldLoadBlockSize, HasStaticBlockSize,
    ParsedReplayBlock, ReplayFloat, ReplayInt,
};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;

#[derive(Debug, PartialEq)]
pub struct Heights(Vec<Height>);

impl Heights {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Heights> {
        match read_utils::read_byte(r) {
            Ok(v) => {
                if v != 4 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Height>::with_capacity(count);

        for _ in 0..count {
            vec.push(Height::load(r)?);
        }

        Ok(Heights(vec))
    }

    pub fn load_block<RS: Read + Seek>(
        r: &mut RS,
        block: &ParsedReplayBlock<Heights>,
    ) -> Result<Self> {
        r.seek(SeekFrom::Start(block.pos))?;

        Self::load(r)
    }

    pub fn get_vec(&self) -> &Vec<Height> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl HasStaticBlockSize for Heights {
    fn get_static_size() -> usize {
        size_of::<u8>() + size_of::<ReplayInt>()
    }
}

impl CouldLoadBlock for ParsedReplayBlock<Heights> {
    type Item = Heights;

    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item> {
        Self::Item::load_block(r, self)
    }
}

impl CouldLoadBlockSize for Heights {
    type Item = Heights;

    fn get_total_block_size<RS: Read + Seek>(
        r: &mut RS,
        pos: u64,
    ) -> Result<ParsedReplayBlock<Heights>> {
        assert_start_of_block(r, BlockType::Heights)?;

        let count = read_utils::read_int(r)?;

        Ok(ParsedReplayBlock::<Heights> {
            pos,
            bytes: Heights::get_static_size() as u64
                + Height::get_static_size() as u64 * count as u64,
            items_count: count,
            _phantom: PhantomData,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Height {
    pub height: ReplayFloat,
    pub time: ReplayTime,
}

impl Height {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Height> {
        let height = read_utils::read_float(r)?;
        let time = read_utils::read_float(r)?;

        Ok(Self { height, time })
    }
}

impl HasStaticBlockSize for Height {
    fn get_static_size() -> usize {
        size_of::<ReplayFloat>() * 2
    }
}

impl CouldLoadBlockSize for Height {
    type Item = Height;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{ReplayFloat, ReplayInt};
    use rand::random;
    use std::io::Cursor;

    pub(crate) fn generate_random_height() -> Height {
        Height {
            height: random::<ReplayFloat>() * 2.0,
            time: random::<ReplayFloat>() * 100.0,
        }
    }

    fn append_height(vec: &mut Vec<u8>, height: &Height) {
        vec.append(&mut ReplayFloat::to_le_bytes(height.height).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(height.time).to_vec());
    }

    pub(self) fn get_heights_buffer(heights: &Vec<Height>) -> Result<Vec<u8>> {
        let heights_id = BlockType::Heights.try_into()?;
        let mut buf: Vec<u8> = Vec::from([heights_id]);

        buf.append(&mut ReplayInt::to_le_bytes(heights.len() as ReplayInt).to_vec());
        for f in heights.iter() {
            append_height(&mut buf, &f);
        }

        Ok(buf)
    }

    #[test]
    fn it_returns_correct_static_size_of_height() {
        assert_eq!(Height::get_static_size(), 8);
    }

    #[test]
    fn it_can_load_height() {
        let wall = generate_random_height();

        let mut buf: Vec<u8> = Vec::new();
        append_height(&mut buf, &wall);

        let result = Height::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, wall)
    }

    #[test]
    fn it_returns_correct_static_size_of_heights() {
        assert_eq!(Heights::get_static_size(), 5);
    }

    #[test]
    fn it_returns_invalid_bsor_error_when_heights_block_id_is_invalid() -> Result<()> {
        let heights = Vec::from([generate_random_height(), generate_random_height()]);

        let mut buf = get_heights_buffer(&heights)?;
        buf[0] = 255;

        let result = Heights::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_heights() -> Result<()> {
        let heights = Vec::from([generate_random_height(), generate_random_height()]);

        let buf = get_heights_buffer(&heights)?;

        let result = Heights::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), heights);
        assert_eq!(result.is_empty(), false);
        assert_eq!(result.len(), heights.len());

        Ok(())
    }

    #[test]
    fn it_can_load_parsed_heights_block() -> Result<()> {
        let heights = Vec::from([generate_random_height(), generate_random_height()]);

        let buf = get_heights_buffer(&heights)?;

        let pos = 0;
        let reader = &mut Cursor::new(buf);
        let walls_block = Heights::get_total_block_size(reader, pos)?;

        let result = walls_block.load(reader)?;

        assert_eq!(walls_block.pos(), pos);
        assert_eq!(
            walls_block.bytes(),
            Heights::get_static_size() as u64
                + Height::get_static_size() as u64 * heights.len() as u64
        );
        assert_eq!(walls_block.is_empty(), false);
        assert_eq!(walls_block.len(), heights.len() as i32);
        assert_eq!(*result.get_vec(), heights);

        Ok(())
    }
}
