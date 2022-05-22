#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    InvalidBytecodeHeader,
    UnsupportedBytecodeVersion,
    NoMoreData,
    InvalidTypeIndex,
    CouldNotReadIndex,
    CouldNotReadStringAtIndex,
    InvalidStringIndex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    position: Option<usize>,
}

impl DecodeError {
    pub(crate) fn with_info(kind: DecodeErrorKind, position: usize) -> DecodeError {
        DecodeError {
            kind,
            position: Some(position),
        }
    }
}