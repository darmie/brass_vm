use crate::{
    code::Code,
    errors::{DecodeError, DecodeErrorKind},
};

#[derive(Clone)]
pub struct Decoder<'input> {
    pub buf: &'input [u8],
    pub file_position: usize,
    pub code: Code,
}

impl<'input> Decoder<'input> {
    pub fn new(buf: &'input [u8]) -> Decoder<'input> {
        Decoder {
            buf,
            file_position: 0,
            code: Code::new(),
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

    pub fn read_index(&mut self, buf: &mut [u8]) -> Result<i32, DecodeError> {
        let b = u8::decode(self)?;
        if (b & 0x80) == 0 {
            return Ok((b & 0x7F).into());
        }
        if (b & 0x40) == 0 {
            let v: u32 = (u8::decode(self)? | (b & 31)).into();
            return Ok(v.try_into().unwrap());
        }
        {
            let c = u8::decode(self)?;
            let d = u8::decode(self)?;
            let e = u8::decode(self)?;

            // let v =  ((b & 31) << 24)  | (c << 16) | (d << 8) | e;
            let v = vec![b, c, d, e];
            let mut u = <[u8; 4]>::default();
            u.copy_from_slice(&v[..]);
            let i = i32::from_be_bytes(u); // << Doesn't feel right!
            Ok(i)
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
    u8 => 1,
    i8 => 1,
    u16 => 2,
    i16 => 2,
    u32 => 4,
    i32 => 4,
    u64 => 8,
    i64 => 8,
}

impl<'input> Decode<'input> for f64 {
    fn decode(decoder: &mut Decoder<'input>) -> Result<f64, DecodeError> {
        let bits = decoder.read()?;
        Ok(f64::from_bits(bits))
    }
}

