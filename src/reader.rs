use std::array::TryFromSliceError;

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
        self.position += 1;
        Ok(self.buffer[self.position - 1])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let result = u16::from_be_bytes(
            self.buffer[self.position..self.position + 2]
                .try_into()
                .map_err(|e: TryFromSliceError| ZNSError::Reader {
                    message: e.to_string(),
                })?,
        );
        self.position += 2;
        Ok(result)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let result = i32::from_be_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into()
                .map_err(|e: TryFromSliceError| ZNSError::Reader {
                    message: e.to_string(),
                })?,
        );
        self.position += 4;
        Ok(result)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let result = u32::from_be_bytes(
            self.buffer[self.position..self.position + 4]
                .try_into()
                .map_err(|e: TryFromSliceError| ZNSError::Reader {
                    message: e.to_string(),
                })?,
        );
        self.position += 4;
        Ok(result)
    }

    pub fn seek(&self, position: usize) -> Result<Self> {
        if position >= self.position {
            Err(ZNSError::Reader {
                message: String::from("Seeking into the future is not allowed!!"),
            })
        } else {
            Ok(Reader::new(&self.buffer[position..self.position]))
        }
    }
}
