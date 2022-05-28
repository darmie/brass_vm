#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeErrorKind {
    InvalidBytecodeHeader,
    UnsupportedBytecodeVersion,
    NoMoreData,
    InvalidType,
    InvalidTypeIndex,
    InvalidOpcode,
    CouldNotProcessOpcode,
    CouldNotReadIndex,
    CouldNotReadStringAtIndex,
    InvalidStringIndex,
    InvalidString,
    NegativeIndex,
    InvalidDebugFile,
    OutsideRange
}

#[derive(Debug, Clone, PartialEq, Copy)]
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