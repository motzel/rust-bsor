use super::{read_utils, vector, BsorError, ReplayInt, ReplayTime, Result};
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct Frames(Vec<Frame>);

impl Frames {
    pub(crate) fn load<R: Read>(r: &mut R) -> Result<Frames> {
        match read_utils::read_byte(r) {
            Ok(v) => {
                if v != 1 {
                    return Err(BsorError::InvalidBsor);
                }
            }
            Err(e) => return Err(e),
        }

        let count = read_utils::read_int(r)? as usize;
        let mut vec = Vec::<Frame>::with_capacity(count);

        for _ in 0..count {
            vec.push(Frame::load(r)?);
        }

        Ok(Frames(vec))
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

#[derive(PartialEq, Debug)]
pub struct Frame {
    pub time: ReplayTime,
    pub fps: ReplayInt,
    pub head: PositionAndRotation,
    pub left_hand: PositionAndRotation,
    pub right_hand: PositionAndRotation,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::ReplayFloat;
    use crate::tests_util::{
        append_vector3, append_vector4, generate_random_vec3, generate_random_vec4,
    };
    use rand::random;
    use std::io::Cursor;

    pub(crate) fn generate_random_position_and_rotation() -> PositionAndRotation {
        PositionAndRotation {
            position: generate_random_vec3(),
            rotation: generate_random_vec4(),
        }
    }

    pub(crate) fn generate_random_frame() -> Frame {
        Frame {
            time: random::<ReplayFloat>() * 100.0,
            fps: random::<u8>() as ReplayInt,
            head: generate_random_position_and_rotation(),
            left_hand: generate_random_position_and_rotation(),
            right_hand: generate_random_position_and_rotation(),
        }
    }

    pub(crate) fn append_position_and_rotation(vec: &mut Vec<u8>, pr: &PositionAndRotation) {
        append_vector3(vec, &pr.position);
        append_vector4(vec, &pr.rotation);
    }

    fn append_frame(vec: &mut Vec<u8>, frame: &Frame) {
        vec.append(&mut ReplayFloat::to_le_bytes(frame.time).to_vec());
        vec.append(&mut ReplayInt::to_le_bytes(frame.fps).to_vec());
        append_position_and_rotation(vec, &frame.head);
        append_position_and_rotation(vec, &frame.left_hand);
        append_position_and_rotation(vec, &frame.right_hand);
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
    fn it_can_load_frames() {
        let frames = Vec::from([generate_random_frame(), generate_random_frame()]);

        let mut buf: Vec<u8> = Vec::from([1u8]);
        buf.append(&mut ReplayInt::to_le_bytes(frames.len() as ReplayInt).to_vec());
        for f in frames.iter() {
            append_frame(&mut buf, &f);
        }

        let result = Frames::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), frames)
    }
}
