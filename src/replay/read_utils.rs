use super::error::BsorError;
use crate::replay::{ReplayFloat, ReplayInt, ReplayLong, Result};
use std::io::Read;

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

    into_replay_float_vec(&buffer)
}

pub(crate) fn read_string<R: Read>(r: &mut R) -> Result<String> {
    let len = read_int(r)?;
    let mut buffer = vec![0; len as usize];

    read_into_buffer(r, &mut buffer)?;

    Ok(std::str::from_utf8(&buffer)?.to_owned())
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
    fn it_returns_io_error() {
        let test_values = [0x1];
        let mut buffer = [0u8; 4];

        let result = read_into_buffer(&mut Cursor::new(test_values), &mut buffer);

        assert!(result.is_err());

        let io_err_kind = match result {
            Ok(_) => panic!("error is expected!"),
            Err(BsorError::Io(e)) => e.kind(),
            _ => panic!("invalid error type!"),
        };

        assert_eq!(std::io::ErrorKind::UnexpectedEof, io_err_kind);
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
    #[ignore]
    fn it_can_read_incorrectly_encoded_string() {
        let buf = [
            26u8, 0, 0, 0, 85, 110, 105, 113, 117, 101, 32, 65, 98, 105, 108, 105, 116, 121, 32,
            47, 32, 227, 131, 166, 227, 131, 139, 227, 131, 188, 227, 130, 175, 227, 130, 162, 227,
            131, 147, 227, 131, 170, 227, 131, 134, 227, 130, 163, 11, 0, 0, 0, 110, 97,
        ];

        let result = read_string(&mut Cursor::new(buf)).unwrap();

        assert_eq!(result, "Unique Ability / ユニークアビリティ");
    }

    #[test]
    fn it_returns_decoding_error_if_string_is_invalid() {
        let invalid_string_buf = [0xffu8, 0xff];

        let result = read_string(&mut Cursor::new(invalid_string_buf));

        assert!(result.is_err());
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
