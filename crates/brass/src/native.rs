use crate::types::ValueType;

#[derive(Clone)]
#[derive(Debug)]
pub struct Native {
    pub lib:String,
    pub name:String,
    pub t:ValueType,
    pub findex:usize
}