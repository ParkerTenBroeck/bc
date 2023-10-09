use std::{ptr::NonNull, num::NonZeroU16, marker::PhantomData};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Unknown(u128),
    UnknownFloat(f64),
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeHint{
    Float,
    Hex,
    Bin,
    Int,
}

#[derive(Clone, Copy)]
pub struct Number2<'a>{
    ptr: NonNull<u8>,
    len: NonZeroU16,
    ext_back_off: u8,
    hint: TypeHint,
    _phan: PhantomData<&'a str>
}

impl std::fmt::Debug for Number2<'_>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Number2").field("ptr", &self.ptr).field("len", &self.len).field("ext_back_off", &self.ext_back_off).field("hint", &self.hint).field("_phan", &self._phan).finish()
    }
}

fn test(){
    // std::str::from
}