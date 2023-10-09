use byteyarn::YarnBox;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone)]
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

    Ident(&'a str),


    StringLiteral(YarnBox<'a, str>),
    NumericLiteral(Number),
    CharLiteral(char),

    SingleLineComment(&'a str),
    MultiLineComment(&'a str),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenFull<'a> {
    pub token: Token<'a>,
    pub meta: TokenMeta,
}

impl<'a> TokenFull<'a> {
    pub fn new(token: Token<'a>, meta: TokenMeta) -> Self {
        Self { token, meta }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenizerErrorFull<'a> {
    pub error: TokenizerError<'a>,
    pub meta: TokenMeta,
}

impl<'a> TokenizerErrorFull<'a> {
    pub fn new(error: TokenizerError<'a>, meta: TokenMeta) -> Self {
        Self { error, meta }
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
}
