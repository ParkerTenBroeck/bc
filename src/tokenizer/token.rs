use byteyarn::YarnBox;

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    LPar,
    RPar,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    ShiftLeft,
    ShiftRight,
    Percent,
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    Dot,
    Comma,
    Colon,
    Semicolon,
    QuestionMark,
    At,
    Octothorp,
    Dollar,

    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equals,
    NotEquals,

    Assignment,

    ModuloEq,
    Modulo,
    DivideEq,
    TimesEq,
    MinusEq,
    PlusEq,
    RangeInclusive,
    RangeExclusive,
    SmallRightArrow,
    BigRightArrow,
    OrEq,
    AndEq,
    XorEq,
    ShiftRightEq,
    ShiftLeftEq,

    Fn,
    Return,
    If,
    Else,
    While,
    Loop,
    Let,
    For,

    Ident(&'a str),

    StringLiteral(YarnBox<'a, str>),
    NumericLiteral(Number<'a>),
    CharLiteral(char),

    FalseLiteral,
    TrueLiteral,

    SingleLineComment(&'a str),
    MultiLineComment(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<T> {
    pub span: TokenMeta,
    pub val: T,
}

impl<T> Span<T> {
    pub fn new(val: T, span: TokenMeta) -> Self {
        Self { val, span }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TokenMeta {
    pub line: u32,
    pub col: u32,
    pub offset: u32,
    pub len: u32,
}

impl TokenMeta {
    pub(super) fn start_end(start: Position, end: Position) -> Self {
        TokenMeta {
            line: start.line as u32,
            col: start.col as u32,
            offset: start.offset as u32,
            len: (end.offset - start.offset) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerError<'a> {
    InvalidChar(char),
    EmptyCharLiteral,
    UnclosedCharLiteral,
    CharLiteralTooBig,
    UnclosedMultiLineComment,
    InvalidEscape(&'a str),
    UnfinishedEscapeSequence(&'a str),
    UnclosedStringLiteral,
    EmptyExponent,
    InvalidBase2Digit(char),
    NoNumberAfterBasePrefix,
    NumberParseError(NumberError),
}
