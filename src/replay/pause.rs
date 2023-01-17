//! structs storing the Pauses block data
use super::{read_utils, BsorError, ReplayTime, Result};
use crate::replay::{
    assert_start_of_block, BlockIndex, BlockType, GetStaticBlockSize, LoadBlock, LoadRealBlockSize,
    ReplayFloat, ReplayInt, ReplayLong,
};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

/// Struct implements [std::ops::Deref] trait so it could be treated as Vec<[Pause]>
#[derive(Debug, PartialEq)]
pub struct Pauses(Vec<Pause>);

impl Pauses {
    #[cfg(test)]
    pub(crate) fn new(vec: Vec<Pause>) -> Pauses {
        Pauses(vec)
    }

    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Pauses> {
        match read_utils::read_byte(r) {
            Ok(v) => {
                if v != 5 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Pause>::with_capacity(count);

        for _ in 0..count {
            vec.push(Pause::load(r)?);
        }

        Ok(Pauses(vec))
    }

    pub(crate) fn load_block<RS: Read + Seek>(
        r: &mut RS,
        block: &BlockIndex<Pauses>,
    ) -> Result<Self> {
        r.seek(SeekFrom::Start(block.pos))?;

        Self::load(r)
    }
}

impl Deref for Pauses {
    type Target = Vec<Pause>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GetStaticBlockSize for Pauses {
    fn get_static_size() -> usize {
        size_of::<u8>() + size_of::<ReplayInt>()
    }
}

impl LoadBlock for BlockIndex<Pauses> {
    type Item = Pauses;

    /// Loads Frames block from ReplayIndex
    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item> {
        Self::Item::load_block(r, self)
    }
}

impl LoadRealBlockSize for Pauses {
    type Item = Pauses;

    fn load_real_block_size<RS: Read + Seek>(r: &mut RS, pos: u64) -> Result<BlockIndex<Pauses>> {
        assert_start_of_block(r, BlockType::Pauses)?;

        let count = read_utils::read_int(r)?;

        Ok(BlockIndex::<Pauses> {
            pos,
            bytes: Pauses::get_static_size() as u64
                + Pause::get_static_size() as u64 * count as u64,
            items_count: count,
            _phantom: PhantomData,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Pause {
    pub duration: ReplayLong,
    pub time: ReplayTime,
}

impl Pause {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Pause> {
        let duration = read_utils::read_long(r)? as ReplayLong;
        let time = read_utils::read_float(r)?;

        Ok(Self { duration, time })
    }
}

impl GetStaticBlockSize for Pause {
    fn get_static_size() -> usize {
        size_of::<ReplayLong>() + size_of::<ReplayFloat>()
    }
}

impl LoadRealBlockSize for Pause {
    type Item = Pause;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_util::{append_pause, generate_random_pause, get_pauses_buffer};
    use std::io::Cursor;

    #[test]
    fn it_returns_correct_static_size_of_pause() {
        assert_eq!(Pause::get_static_size(), 12);
    }

    #[test]
    fn it_can_load_pause() {
        let pause = generate_random_pause();

        let mut buf: Vec<u8> = Vec::new();
        append_pause(&mut buf, &pause);

        let result = Pause::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, pause)
    }

    #[test]
    fn it_returns_correct_static_size_of_pauses() {
        assert_eq!(Pauses::get_static_size(), 5);
    }

    #[test]
    fn it_returns_invalid_bsor_error_when_pauses_block_id_is_invalid() -> Result<()> {
        let pauses = Vec::from([generate_random_pause(), generate_random_pause()]);

        let mut buf = get_pauses_buffer(&pauses)?;
        buf[0] = 255;

        let result = Pauses::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_pauses() -> Result<()> {
        let pauses = Vec::from([generate_random_pause(), generate_random_pause()]);

        let buf = get_pauses_buffer(&pauses)?;

        let result = Pauses::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result, pauses);
        assert_eq!(result.len(), pauses.len());

        Ok(())
    }

    #[test]
    fn it_can_load_pauses_block_index() -> Result<()> {
        let pauses = Vec::from([generate_random_pause(), generate_random_pause()]);

        let buf = get_pauses_buffer(&pauses)?;

        let pos = 0;
        let reader = &mut Cursor::new(buf);
        let walls_block = Pauses::load_real_block_size(reader, pos)?;

        let result = walls_block.load(reader)?;

        assert_eq!(walls_block.pos(), pos);
        assert_eq!(
            walls_block.bytes(),
            Pauses::get_static_size() as u64
                + Pause::get_static_size() as u64 * pauses.len() as u64
        );
        assert_eq!(walls_block.is_empty(), false);
        assert_eq!(walls_block.len(), pauses.len() as i32);
        assert_eq!(*result, pauses);

        Ok(())
    }
}
