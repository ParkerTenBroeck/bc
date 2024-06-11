use std::num::NonZeroUsize;

use crate::parser::ast::{Expression, FloatType, IntSize, Path};

use super::{ConstantId, Context, TypeMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int(IntSize, bool),
    Float(FloatType),
    Bool,
    Char,
    Void,
    Str,
    FnPointer(Vec<Type>, Option<Box<Type>>),
    Nammed(Path),
    Ptr(Box<Type>),
    Ref(Box<Type>),
    Array(Box<Type>),
    ArrayStatic(Box<Type>, usize),
}

impl Type {

    pub fn layout(&self, context: &mut Context) -> Layout {
        match self {
            Type::Int(size, _) => match size {
                IntSize::U8 => Layout::new(1, 1).unwrap(),
                IntSize::U16 => Layout::new(2, 2).unwrap(),
                IntSize::U32 => Layout::new(4, 4).unwrap(),
                IntSize::U64 => Layout::new(8, 8).unwrap(),

                IntSize::Usize => Layout::new(8, 8).unwrap(),
            },
            Type::Float(size) => match size {
                FloatType::F32 => Layout::new(4, 4).unwrap(),
                FloatType::F64 => Layout::new(8, 8).unwrap(),
            },
            Type::Bool => Layout::new(1, 1).unwrap(),
            Type::Char => Layout::new(1, 1).unwrap(),
            Type::Void => Layout::ZERO_SIZE,
            Type::FnPointer(_, _) => Layout::new(8, 8).unwrap(),

            Type::Str => Layout::ZERO_SIZE_UNSIZED,

            Type::Ptr(inner) => {
                if inner.layout(context).is_sized() {
                    Layout::new(8, 8).unwrap()
                } else {
                    Layout::new(16, 8).unwrap()
                }
            }

            Type::Ref(inner) => {
                if inner.layout(context).is_sized() {
                    Layout::new(8, 8).unwrap()
                } else {
                    Layout::new(16, 8).unwrap()
                }
            }

            Type::Array(inner) => {
                let mut inner = inner.layout(context);
                assert!(inner.is_sized());
                inner.sized = false;
                inner.size = 0;
                inner
            }

            Type::ArrayStatic(item, length) => {
                let mut layout = item.layout(context);
                // layout.size *= *length;
                layout
            }

            Type::Nammed(user) => context.layout(user),
        }
    }
}

const VAL: usize = 34-2;
struct Test{
    vals: [u8; VAL],
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    size: usize,
    align: NonZeroUsize,
    sized: bool,
}

impl Layout {
    pub const ZERO_SIZE: Self = Layout {
        size: 0,
        align: NonZeroUsize::MIN,
        sized: true,
    };

    pub const ZERO_SIZE_UNSIZED: Self = Layout {
        size: 0,
        align: NonZeroUsize::MIN,
        sized: false,
    };

    pub fn new(size: usize, align: usize) -> Option<Self> {
        if align.count_ones() != 1 {
            return None;
        }
        Some(Self {
            size,
            align: NonZeroUsize::new(align)?,
            sized: true,
        })
    }

    pub fn new_nonzero(size: usize, align: NonZeroUsize) -> Option<Self> {
        if align.get().count_ones() != 1 {
            return None;
        }
        Some(Self {
            size,
            align,
            sized: true,
        })
    }

    pub fn new_unsized(size: usize, align: usize) -> Option<Self> {
        if align.count_ones() != 1 {
            return None;
        }
        Some(Self {
            size,
            align: NonZeroUsize::new(align)?,
            sized: false,
        })
    }

    pub fn new_nonzero_unsized(size: usize, align: NonZeroUsize) -> Option<Self> {
        if align.get().count_ones() != 1 {
            return None;
        }
        Some(Self {
            size,
            align,
            sized: false,
        })
    }

    fn align_size(mut self) -> Self {
        self.size += self.align.get() - 1;
        self.size &= !(self.align.get() - 1);
        self
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            size: self.size.max(other.size),
            align: self.align.max(other.align),
            sized: self.sized && other.sized,
        }
        .align_size()
    }

    pub fn size_bytes(&self) -> usize {
        self.size
    }

    pub fn align(&self) -> NonZeroUsize {
        self.align
    }

    pub fn is_sized(&self) -> bool {
        self.sized
    }

    pub fn unsize(mut self) -> Layout {
        self.sized = false;
        self.size = 0;
        self
    }
}
