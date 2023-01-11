use super::error::BsorError;
use crate::replay::{ReplayFloat, ReplayInt, ReplayLong};
use std::io::Read;

type Result<T> = std::result::Result<T, BsorError>;

pub(crate) fn read_byte<R: Read>(r: &mut R) -> Result<u8> {
    let mut buffer = [0; std::mem::size_of::<u8>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(buffer[0])
}

pub(crate) fn read_bool<R: Read>(r: &mut R) -> Result<bool> {
    let b = read_byte(r)?;

    Ok(b == 1)
}

pub(crate) fn read_int<R: Read>(r: &mut R) -> Result<ReplayInt> {
    let mut buffer = [0; std::mem::size_of::<ReplayInt>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(ReplayInt::from_le_bytes(buffer))
}

pub(crate) fn read_long<R: Read>(r: &mut R) -> Result<ReplayLong> {
    let mut buffer = [0; std::mem::size_of::<ReplayLong>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(ReplayLong::from_le_bytes(buffer))
}

pub(crate) fn read_float<R: Read>(r: &mut R) -> Result<ReplayFloat> {
    let mut buffer = [0; std::mem::size_of::<ReplayFloat>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(ReplayFloat::from_le_bytes(buffer))
}

pub(crate) fn read_float_multi<R: Read>(r: &mut R, count: usize) -> Result<Vec<ReplayFloat>> {
    let mut buffer = vec![0; count * std::mem::size_of::<ReplayFloat>()];

    read_into_buffer(r, &mut buffer)?;

    Ok(into_replay_float_vec(&buffer)?)
}

pub(crate) fn read_string<R: Read>(r: &mut R) -> Result<String> {
    let len = read_int(r)?;
    let mut buffer = vec![0; len as usize];

    read_into_buffer(r, &mut buffer)?;

    match std::str::from_utf8(&buffer) {
        Ok(v) => Ok(v.to_owned()),
        Err(e) => Err(BsorError::DecodingError(Box::new(e))),
    }
}

pub(crate) fn read_into_buffer<'a, R: Read>(r: &'a mut R, buffer: &'a mut [u8]) -> Result<()> {
    let result = r.read_exact(buffer);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(BsorError::Io(e)),
    }
}

fn into_replay_float_vec(buf: &[u8]) -> Result<Vec<ReplayFloat>> {
    let count = buf.len() / std::mem::size_of::<ReplayFloat>();

    let mut vec = Vec::with_capacity(count);

    for i in 0..count {
        vec.push(ReplayFloat::from_le_bytes(
            buf[i * std::mem::size_of::<ReplayFloat>()
                ..(i + 1) * std::mem::size_of::<ReplayFloat>()]
                .try_into()?,
        ));
    }

    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_can_read_into_buffer() {
        let test_values = [0x1, 0x2, 0x3, 0x4];
        let mut buffer = [0u8; 4];

        read_into_buffer(&mut Cursor::new(test_values), &mut buffer).unwrap();

        assert_eq!(buffer, test_values);
    }

    #[test]
    fn it_can_read_int() {
        let test_replay_int_buf = [1, 2, 3, 4];

        let value = read_int(&mut Cursor::new(test_replay_int_buf)).unwrap();

        assert_eq!(value, ReplayInt::from_le_bytes(test_replay_int_buf));
    }

    #[test]
    fn it_can_read_long() {
        let test_replay_long_buf = [1, 2, 3, 4, 5, 6, 7, 8];

        let value = read_long(&mut Cursor::new(test_replay_long_buf)).unwrap();

        assert_eq!(value, ReplayLong::from_le_bytes(test_replay_long_buf));
    }

    #[test]
    fn it_can_read_float() {
        let f = 3.14;
        let test_replay_float_buf = ReplayFloat::to_le_bytes(f);

        let value = read_float(&mut Cursor::new(test_replay_float_buf)).unwrap();

        assert_eq!(f, value);
    }

    #[test]
    fn it_can_read_byte() {
        let test_u8_buf = [1];

        let value = read_byte(&mut Cursor::new(test_u8_buf)).unwrap();

        assert_eq!(value, test_u8_buf[0]);
    }

    #[test]
    fn it_can_read_bool() {
        let b = true;
        let test_bool_buf = [1];

        let value = read_bool(&mut Cursor::new(test_bool_buf)).unwrap();

        assert_eq!(b, value);
    }

    #[test]
    fn it_can_read_string() {
        let test_string = "test_str";

        let len = test_string.len() as ReplayInt;
        let mut test_string_buf = ReplayInt::to_le_bytes(len).to_vec();
        test_string_buf.append(&mut test_string.as_bytes().to_vec());

        let value = read_string(&mut Cursor::new(test_string_buf)).unwrap();

        assert_eq!(value, test_string);
    }

    #[test]
    fn it_can_read_multi_float() {
        let floats = vec![1.0, 1.5, 2.0, 2.5, 3.0];
        let mut u8_vec: Vec<u8> =
            Vec::with_capacity(floats.len() * std::mem::size_of::<ReplayFloat>());

        for i in 0..floats.len() {
            u8_vec.extend_from_slice(&ReplayFloat::to_le_bytes(floats[i]));
        }

        let result = read_float_multi(&mut Cursor::new(&u8_vec[..]), floats.len()).unwrap();

        assert_eq!(floats, result);
    }
}
