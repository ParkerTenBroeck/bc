use crate::tokenizer::{Number, Span};

#[derive(Debug, Clone)]
pub enum TokenTree<'a> {
    Literal(Span<LiteralType<'a>>),
    Loop(Box<Span<TokenTree<'a>>>),
    While {},
    FunctionCall {
        name: Span<LiteralType<'a>>,
        arguments: Vec<TokenTree<'a>>,
    },
}

#[derive(Debug, Clone)]
pub enum LiteralType<'a> {
    Boolean(bool),
    String(byteyarn::YarnBox<'a, str>),
    Char(char),
    Number(Number<'a>),
}
