use crate::replay::{read_utils, BsorError, ReplayFloat};
use std::io::Read;

#[derive(PartialEq, Debug)]
pub struct Vector3 {
    pub x: ReplayFloat,
    pub y: ReplayFloat,
    pub z: ReplayFloat,
}

impl Vector3 {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Vector3, BsorError> {
        let vec = read_utils::read_float_multi(r, 3)?;

        Ok(Self {
            x: vec[0],
            y: vec[1],
            z: vec[2],
        })
    }
}

impl From<Vector4> for Vector3 {
    fn from(v: Vector4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Vector4 {
    pub x: ReplayFloat,
    pub y: ReplayFloat,
    pub z: ReplayFloat,
    pub w: ReplayFloat,
}

impl Vector4 {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Vector4, BsorError> {
        let vec = read_utils::read_float_multi(r, 4)?;

        Ok(Self {
            x: vec[0],
            y: vec[1],
            z: vec[2],
            w: vec[3],
        })
    }
}

impl From<Vector3> for Vector4 {
    fn from(v: Vector3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_can_load_vector3() {
        let floats = [1.0, 1.5, 2.0];
        let mut u8_vec: Vec<u8> =
            Vec::with_capacity(floats.len() * std::mem::size_of::<ReplayFloat>());

        for i in 0..floats.len() {
            u8_vec.extend_from_slice(&ReplayFloat::to_le_bytes(floats[i]));
        }

        let result = Vector3::load(&mut Cursor::new(&u8_vec[..])).unwrap();

        assert_eq!(floats[0], result.x);
        assert_eq!(floats[1], result.y);
        assert_eq!(floats[2], result.z);
    }

    #[test]
    fn it_can_load_vector4() {
        let floats = [1.0, 1.5, 2.0, 2.5];
        let mut u8_vec: Vec<u8> =
            Vec::with_capacity(floats.len() * std::mem::size_of::<ReplayFloat>());

        for i in 0..floats.len() {
            u8_vec.extend_from_slice(&ReplayFloat::to_le_bytes(floats[i]));
        }

        let result = Vector4::load(&mut Cursor::new(&u8_vec[..])).unwrap();

        assert_eq!(floats[0], result.x);
        assert_eq!(floats[1], result.y);
        assert_eq!(floats[2], result.z);
        assert_eq!(floats[3], result.w);
    }
}
