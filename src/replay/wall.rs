use super::{read_utils, ReplayTime, Result};
use crate::replay::{
    assert_start_of_block, BlockType, CouldLoadBlock, CouldLoadBlockSize, HasStaticBlockSize,
    LineValue, ParsedReplayBlock, ReplayFloat, ReplayInt,
};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;

#[derive(Debug, PartialEq)]
pub struct Walls(Vec<Wall>);

impl Walls {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Walls> {
        assert_start_of_block(r, BlockType::Walls)?;

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Wall>::with_capacity(count);

        for _ in 0..count {
            vec.push(Wall::load(r)?);
        }

        Ok(Walls(vec))
    }

    pub fn load_block<RS: Read + Seek>(
        r: &mut RS,
        block: &ParsedReplayBlock<Walls>,
    ) -> Result<Self> {
        r.seek(SeekFrom::Start(block.pos))?;

        Self::load(r)
    }

    pub fn get_vec(&self) -> &Vec<Wall> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl HasStaticBlockSize for Walls {
    fn get_static_size() -> usize {
        size_of::<u8>() + size_of::<ReplayInt>()
    }
}

impl CouldLoadBlock for ParsedReplayBlock<Walls> {
    type Item = Walls;

    fn load<RS: Read + Seek>(&self, r: &mut RS) -> Result<Self::Item> {
        Self::Item::load_block(r, self)
    }
}

impl CouldLoadBlockSize for Walls {
    type Item = Walls;

    fn get_total_block_size<RS: Read + Seek>(
        r: &mut RS,
        pos: u64,
    ) -> Result<ParsedReplayBlock<Walls>> {
        assert_start_of_block(r, BlockType::Walls)?;

        let count = read_utils::read_int(r)?;

        Ok(ParsedReplayBlock::<Walls> {
            pos,
            bytes: Walls::get_static_size() as u64 + Wall::get_static_size() as u64 * count as u64,
            items_count: count,
            _phantom: PhantomData,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Wall {
    pub line_idx: LineValue,
    pub obstacle_type: u8,
    pub width: u8,
    pub energy: ReplayFloat,
    pub time: ReplayTime,
    pub spawn_time: ReplayTime,
}

impl Wall {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Wall> {
        let mut wall_id = read_utils::read_int(r)?;

        let line_idx = (wall_id / 100) as LineValue;
        wall_id %= 100;

        let obstacle_type = (wall_id / 10) as u8;
        wall_id %= 10;

        let width = wall_id as u8;

        let energy = read_utils::read_float(r)?;
        let time = read_utils::read_float(r)?;
        let spawn_time = read_utils::read_float(r)?;

        Ok(Self {
            line_idx,
            obstacle_type,
            width,
            energy,
            time,
            spawn_time,
        })
    }
}

impl HasStaticBlockSize for Wall {
    fn get_static_size() -> usize {
        size_of::<ReplayInt>() + size_of::<ReplayFloat>() * 3
    }
}

impl CouldLoadBlockSize for Wall {
    type Item = Wall;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{BsorError, ReplayFloat, ReplayInt};
    use rand::random;
    use std::io::Cursor;

    pub(crate) fn generate_random_wall() -> Wall {
        Wall {
            line_idx: random::<u8>() % 4,
            obstacle_type: random::<u8>() % 10,
            width: random::<u8>() % 4,
            energy: random::<ReplayFloat>() * 100.0,
            time: random::<ReplayFloat>() * 100.0,
            spawn_time: random::<ReplayFloat>() * 100.0,
        }
    }

    fn append_wall(vec: &mut Vec<u8>, wall: &Wall) {
        let wall_id: ReplayInt = wall.line_idx as ReplayInt * 100
            + wall.obstacle_type as ReplayInt * 10
            + wall.width as ReplayInt;
        vec.append(&mut ReplayInt::to_le_bytes(wall_id).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.energy).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.time).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(wall.spawn_time).to_vec());
    }

    pub(self) fn get_walls_buffer(walls: &Vec<Wall>) -> Result<Vec<u8>> {
        let walls_id = BlockType::Walls.try_into()?;
        let mut buf: Vec<u8> = Vec::from([walls_id]);

        buf.append(&mut ReplayInt::to_le_bytes(walls.len() as ReplayInt).to_vec());
        for f in walls.iter() {
            append_wall(&mut buf, &f);
        }

        Ok(buf)
    }

    #[test]
    fn it_returns_correct_static_size_of_wall() {
        assert_eq!(Wall::get_static_size(), 16);
    }

    #[test]
    fn it_can_load_wall() {
        let wall = generate_random_wall();

        let mut buf: Vec<u8> = Vec::new();
        append_wall(&mut buf, &wall);

        let result = Wall::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, wall)
    }

    #[test]
    fn it_returns_correct_static_size_of_walls() {
        assert_eq!(Walls::get_static_size(), 5);
    }

    #[test]
    fn it_returns_invalid_bsor_error_when_walls_block_id_is_invalid() -> Result<()> {
        let walls = Vec::from([generate_random_wall(), generate_random_wall()]);

        let mut buf = get_walls_buffer(&walls)?;
        buf[0] = 255;

        let result = Walls::load(&mut Cursor::new(buf));

        assert!(matches!(result, Err(BsorError::InvalidBsor)));

        Ok(())
    }

    #[test]
    fn it_can_load_walls() -> Result<()> {
        let walls = Vec::from([generate_random_wall(), generate_random_wall()]);

        let buf = get_walls_buffer(&walls)?;

        let result = Walls::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), walls);
        assert_eq!(result.is_empty(), false);
        assert_eq!(result.len(), walls.len());

        Ok(())
    }

    #[test]
    fn it_can_load_parsed_walls_block() -> Result<()> {
        let walls = Vec::from([generate_random_wall(), generate_random_wall()]);

        let buf = get_walls_buffer(&walls)?;

        let pos = 0;
        let reader = &mut Cursor::new(buf);
        let walls_block = Walls::get_total_block_size(reader, pos)?;

        let result = walls_block.load(reader)?;

        assert_eq!(walls_block.pos(), pos);
        assert_eq!(
            walls_block.bytes(),
            Walls::get_static_size() as u64 + Wall::get_static_size() as u64 * walls.len() as u64
        );
        assert_eq!(walls_block.is_empty(), false);
        assert_eq!(walls_block.len(), walls.len() as i32);
        assert_eq!(*result.get_vec(), walls);

        Ok(())
    }
}
