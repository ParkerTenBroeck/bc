#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType{
    Unknown,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number{
    val: u128,
    typ: NumberType
}