use crate::errors::ZNSError;

pub struct Reader<'a> {
    buffer: &'a [u8],
    position: usize,
}

type Result<T> = std::result::Result<T, ZNSError>;

impl<'a> Reader<'a> {
    pub fn new(buffer: &[u8]) -> Reader {
        Reader {
            buffer,
            position: 0,
        }
    }

    pub fn unread_bytes(&self) -> usize {
        self.buffer.len() - self.position
    }

    pub fn read(&mut self, size: usize) -> Result<Vec<u8>> {
        if size > self.unread_bytes() {
            Err(ZNSError::Reader {
                message: String::from("cannot read enough bytes"),
            })
        } else {
            self.position += size;
            Ok(self.buffer[self.position - size..self.position].to_vec())
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        if self.unread_bytes() == 0 {
            Err(ZNSError::Reader {
                message: String::from("cannot read u8"),
            })
        } else {
            self.position += 1;
            Ok(self.buffer[self.position - 1])
        }
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let result =
            u16::from_be_bytes(self.read(2)?.try_into().map_err(|_| ZNSError::Reader {
                message: String::from("invalid read_u16"),
            })?);
        Ok(result)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let result =
            i32::from_be_bytes(self.read(4)?.try_into().map_err(|_| ZNSError::Reader {
                message: String::from("invalid read_u32"),
            })?);
        Ok(result)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let result =
            u32::from_be_bytes(self.read(4)?.try_into().map_err(|_| ZNSError::Reader {
                message: String::from("invalid read_u32"),
            })?);
        Ok(result)
    }

    pub fn seek(&self, position: usize) -> Result<Self> {
        if position >= self.position {
            Err(ZNSError::Reader {
                message: String::from("Seeking into the future is not allowed!!"),
            })
        } else {
            let mut reader = Reader::new(&self.buffer[0..self.position]);
            reader.position = position;
            Ok(reader)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let fake_bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut reader = Reader::new(&fake_bytes);

        assert_eq!(reader.unread_bytes(), 11);

        let u16 = reader.read_u16();
        assert!(u16.is_ok());
        assert_eq!(u16.unwrap(), 1);

        assert_eq!(reader.unread_bytes(), 9);

        let u8 = reader.read_u8();
        assert!(u8.is_ok());
        assert_eq!(u8.unwrap(), 2);
        assert_eq!(reader.unread_bytes(), 8);

        let u32 = reader.read_u32();
        assert!(u32.is_ok());
        assert_eq!(
            u32.unwrap(),
            u32::from_be_bytes(fake_bytes[3..7].try_into().unwrap())
        );
        assert_eq!(reader.unread_bytes(), 4);

        let read = reader.read(3);
        assert!(read.is_ok());
        assert_eq!(read.unwrap(), fake_bytes[7..10]);
        assert_eq!(reader.unread_bytes(), 1);

        let too_much = reader.read(2);
        assert!(too_much.is_err());
        assert_eq!(reader.unread_bytes(), 1);

        assert!(reader.read_u8().is_ok());

        assert!(reader.read_u8().is_err());
        assert!(reader.read_u16().is_err());
        assert!(reader.read_u32().is_err());
        assert!(reader.read_i32().is_err());

        let new_reader = reader.seek(1);
        assert!(new_reader.is_ok());
        assert_eq!(new_reader.unwrap().unread_bytes(), 10);

        let new_reader = reader.seek(100);
        assert!(new_reader.is_err());
    }
}
