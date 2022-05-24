use crate::{errors::{DecodeErrorKind, DecodeError}, code::Code};

#[derive(Clone)]
pub struct Decoder<'input> {
    buf: &'input [u8],
    pub file_position: usize,
    pub code:Code
}

impl<'input> Decoder<'input> {
    pub fn new(buf: &'input [u8]) -> Decoder<'input> {
        Decoder {
            buf,
            file_position: 0,
            code: Code::new()
        }
    }

    /// The position inside the file, *not* this decoder.
    pub fn file_position(&self) -> usize {
        self.file_position
    }

    /// Advances by a specific number of bytes.
    pub fn advance(&mut self, count: usize) -> Result<(), DecodeError> {
        if count > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::NoMoreData,
                self.file_position,
            ))
        } else {
            self.buf = &self.buf[count..];
            self.file_position += count;
            Ok(())
        }
    }

    /// Reads bytes into the buffer supplied and advances.
    pub fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        if buf.len() > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::NoMoreData,
                self.file_position,
            ))
        } else {
            buf.copy_from_slice(&self.buf[..buf.len()]);
            self.buf = &self.buf[buf.len()..];
            self.file_position += buf.len();
            Ok(())
        }
    }

    pub fn read_bytes_vec(&mut self, buf: &mut Vec<u8>) -> Result<(), DecodeError> {
        if buf.len() > self.buf.len() {
            Err(DecodeError::with_info(
                DecodeErrorKind::NoMoreData,
                self.file_position,
            ))
        } else {
            let sz = buf.len();
            buf.copy_from_slice(&self.buf[..sz]);
            self.buf = &self.buf[buf.len()..];
            self.file_position += buf.len();
            Ok(())
        }
    }

    pub fn read<T: Decode<'input>>(&mut self) -> Result<T, DecodeError> {
        T::decode(self)
    }
}

pub trait Decode<'input>: Sized + 'input {
    fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError>;
}

macro_rules! impl_decode {
    ($($t:ty => $len:expr,)*) => {
        $(
            impl<'input> Decode<'input> for $t {
                fn decode(decoder: &mut Decoder<'input>) -> Result<Self, DecodeError> {
                    let mut buf = <[u8; $len]>::default();
                    decoder.read_bytes(&mut buf)?;
                    Ok(Self::from_le_bytes(buf))
                }
            }
        )*
    }
}

impl_decode! {
    u8 => 1, i8 => 1,
    u16 => 2, i16 => 2,
    u32 => 4, i32 => 4,
    u64 => 8, i64 => 8,
}

impl<'input> Decode<'input> for f64 {
    fn decode(decoder: &mut Decoder<'input>) -> Result<f64, DecodeError> {
        let bits = decoder.read()?;
        Ok(f64::from_bits(bits))
    }
}