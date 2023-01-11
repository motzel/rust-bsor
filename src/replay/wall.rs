use super::{read_utils, BsorError, ReplayTime, Result};
use crate::replay::{LineValue, ReplayFloat};
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct Walls(Vec<Wall>);

impl Walls {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Walls> {
        match read_utils::read_byte(r) {
            Ok(v) => {
                if v != 3 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Wall>::with_capacity(count);

        for _ in 0..count {
            vec.push(Wall::load(r)?);
        }

        Ok(Walls(vec))
    }

    pub fn get_vec(&self) -> &Vec<Wall> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
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
        wall_id = wall_id % 100;

        let obstacle_type = (wall_id / 10) as u8;
        wall_id = wall_id % 10;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{ReplayFloat, ReplayInt};
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

    #[test]
    fn it_can_load_wall() {
        let wall = generate_random_wall();

        let mut buf: Vec<u8> = Vec::new();
        append_wall(&mut buf, &wall);

        let result = Wall::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, wall)
    }

    #[test]
    fn it_can_load_walls() {
        let walls = Vec::from([generate_random_wall(), generate_random_wall()]);

        let mut buf: Vec<u8> = Vec::from([3u8]);
        buf.append(&mut ReplayInt::to_le_bytes(walls.len() as ReplayInt).to_vec());
        for f in walls.iter() {
            append_wall(&mut buf, &f);
        }

        let result = Walls::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), walls)
    }
}
