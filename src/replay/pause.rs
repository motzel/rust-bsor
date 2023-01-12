use super::{read_utils, BsorError, ReplayTime, Result};
use crate::replay::ReplayLong;
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct Pauses(Vec<Pause>);

impl Pauses {
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

    pub fn get_vec(&self) -> &Vec<Pause> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{ReplayFloat, ReplayInt};
    use rand::random;
    use std::io::Cursor;

    pub(crate) fn generate_random_pause() -> Pause {
        Pause {
            duration: random::<ReplayLong>() % 30,
            time: random::<ReplayFloat>() * 100.0,
        }
    }

    fn append_pause(vec: &mut Vec<u8>, pause: &Pause) {
        vec.append(&mut ReplayLong::to_le_bytes(pause.duration).to_vec());
        vec.append(&mut ReplayFloat::to_le_bytes(pause.time).to_vec());
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
    fn it_can_load_pauses() {
        let pauses = Vec::from([generate_random_pause(), generate_random_pause()]);

        let mut buf: Vec<u8> = Vec::from([5u8]);
        buf.append(&mut ReplayInt::to_le_bytes(pauses.len() as ReplayInt).to_vec());
        for f in pauses.iter() {
            append_pause(&mut buf, &f);
        }

        let result = Pauses::load(&mut Cursor::new(buf)).unwrap();

        assert_eq!(*result.get_vec(), pauses)
    }
}
