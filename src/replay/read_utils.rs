use super::error::BsorError;
use std::io::Read;

pub(crate) fn read_byte<R: Read>(r: &mut R) -> Result<u8, BsorError> {
    let mut buffer = [0; std::mem::size_of::<u8>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(buffer[0])
}

pub(crate) fn read_int<R: Read>(r: &mut R) -> Result<i32, BsorError> {
    let mut buffer = [0; std::mem::size_of::<i32>()];
    read_into_buffer(r, &mut buffer)?;

    Ok(i32::from_le_bytes(buffer))
}

pub(crate) fn read_into_buffer<'a, R: Read>(
    r: &'a mut R,
    buffer: &'a mut [u8],
) -> Result<(), BsorError> {
    let result = r.read_exact(buffer);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(BsorError::Io(e)),
    }
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
    fn it_can_read_i32() {
        let test_i32_buf = [1, 2, 3, 4];

        let value = read_int(&mut Cursor::new(test_i32_buf)).unwrap();

        assert_eq!(value, i32::from_le_bytes(test_i32_buf));
    }

    #[test]
    fn it_can_read_u8() {
        let test_u8_buf = [1];

        let value = read_byte(&mut Cursor::new(test_u8_buf)).unwrap();

        assert_eq!(value, test_u8_buf[0]);
    }
}
