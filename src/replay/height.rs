use super::{read_utils, BsorError, ReplayTime, Result};
use crate::replay::ReplayFloat;
use std::io::Read;

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

    #[test]
    fn it_can_load_height() {
        let wall = generate_random_height();

        let mut buf: Vec<u8> = Vec::new();
        append_height(&mut buf, &wall);

        let result = Height::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, wall)
    }

    #[test]
    fn it_can_load_heights() {
        let walls = Vec::from([generate_random_height(), generate_random_height()]);

        let mut buf: Vec<u8> = Vec::from([4u8]);
        buf.append(&mut ReplayInt::to_le_bytes(walls.len() as ReplayInt).to_vec());
        for f in walls.iter() {
            append_height(&mut buf, &f);
        }

        let result = Heights::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), walls)
    }
}
