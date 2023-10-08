use std::{iter::Peekable, str::Chars, fmt::Write};

use byteyarn::YarnBox;

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
    NumericLiteral(u128, ()),
    CharLiteral(char),
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
pub struct TokenizerErrorFull {
    pub error: TokenizerError,
    pub meta: TokenMeta,
}

impl TokenizerErrorFull {
    pub fn new(error: TokenizerError, meta: TokenMeta) -> Self {
        Self {
            error,
            meta,
        }
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
    fn start_end(start: Position, end: Position) -> Self {
        TokenMeta {
            line: start.line as u32,
            col: start.col as u32,
            offset: start.offset as u32,
            len: (end.offset - start.offset) as u32,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Position {
    offset: usize,
    line: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerError {
    InvalidChar(char),
    EmptyCharLiteral,
    UnclosedCharLiteral,
    CharLiteralTooBig,
}

type TokenizerResult<'a> = Result<TokenFull<'a>, Box<TokenizerErrorFull>>;

enum State {
    Default,

    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    Equal,
    Gt,
    GtGt,
    Lt,
    LtLt,
    Not,
    Or,
    And,
    Xor,

    Dot,
    DotDot,

    Ident,

    Eof,
    
    String,
    CharLiteral,
    CharLiteralEnd,
    CharLiteralLarge,
}

pub struct Tokenizer<'a> {
    str: &'a str,
    chars: Peekable<Chars<'a>>,
    state: State,

    start: Position,
    current: Position,

    str_builder: SSBuilder<'a>
}

pub enum SSBuilder<'a>{
    None,
    Ref(&'a str),
    Small(str_buf::StrBuf<15>),
    Alloc(String)
}

impl<'a> SSBuilder<'a> {
    fn take(&mut self) -> SSBuilder<'a> {
        let mut tmp = SSBuilder::None;
        std::mem::swap(self, &mut tmp);
        tmp
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(str: &'a str) -> Self {
        // byteyarn::Yarn::from;
        Self {
            str,
            chars: str.chars().peekable(),
            state: State::Default,
            start: Position::default(),
            current: Position::default(),
            str_builder: SSBuilder::None,
        }
    }
}

impl<'a> Iterator for  Tokenizer<'a>{
    type Item = TokenizerResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut char_lit = '\0';
        loop {
            let c = self.chars.peek().copied();
            let mut consume = true;
            let update_start_on_error = true;
            let error_meta = None;


            let mut ret = None;
            let ok_ret_state = State::Default;
            let err_ret_state = State::Default;

            let processing = if let Some(char) = c {
                let mut tmp = self.current;
                tmp.offset += char.len_utf8();
                if char == '\n' {
                    tmp.line += 1;
                    tmp.col = 0;
                } else {
                    tmp.col += 1;
                }
                tmp
            } else {
                self.current
            };

            macro_rules! eof_none {
                ($expr:expr) => {
                    if let Some(char) = $expr {
                        char
                    } else {
                        self.state = State::Eof;
                        return None;
                    }
                };
            }

            macro_rules! unconsume_ret {
                ($sel:ident, $expr:expr) => {
                    {
                        consume = false;
                        ret = Some($expr);
                    }
                };
            }

            match self.state {
                State::Default => match eof_none!(c) {
                    '|' => self.state = State::Or,
                    '^' => self.state = State::Xor,
                    '/' => self.state = State::Divide,
                    '%' => self.state = State::Mod,
                    '-' => self.state = State::Minus,
                    '+' => self.state = State::Plus,
                    '*' => self.state = State::Times,
                    '=' => self.state = State::Equal,                    
                    '.' => self.state = State::Dot,
                    '<' => self.state = State::Lt,
                    '>' => self.state = State::Gt,
                    '!' => self.state = State::Not,
                    '&' => self.state = State::And,
                    '"' => self.state = State::String,
                    '\'' => self.state = State::CharLiteral,

                    '(' => ret = Some(Ok(Token::LPar)),
                    ')' => ret = Some(Ok(Token::RPar)),
                    '{' => ret = Some(Ok(Token::LBrace)),
                    '}' => ret = Some(Ok(Token::RBrace)),
                    '[' => ret = Some(Ok(Token::LBracket)),
                    ']' => ret = Some(Ok(Token::RBracket)),
                    '~' => ret = Some(Ok(Token::BitwiseNot)),
                    ',' => ret = Some(Ok(Token::Comma)),
                    '?' => ret = Some(Ok(Token::QuestionMark)),
                    ':' => ret = Some(Ok(Token::Colon)),

                    c if c.is_whitespace() => self.start = processing,
                    c if c.is_alphabetic() => self.state = State::Ident,

                    c => ret = Some(Err(TokenizerError::InvalidChar(c)))
                },
                State::Plus => match c{
                    Some('=') => ret = Some(Ok(Token::PlusEq)),
                    _ => unconsume_ret!(self, Ok(Token::Plus)),
                },
                State::Minus => match c{
                    Some('=') => ret = Some(Ok(Token::MinusEq)),
                    _ => unconsume_ret!(self, Ok(Token::Minus)),
                },
                State::Times => match c{
                    Some('=') => ret = Some(Ok(Token::TimesEq)),
                    _ => unconsume_ret!(self, Ok(Token::Star)),
                },
                State::Divide => match c{
                    Some('=') => ret = Some(Ok(Token::DivideEq)),
                    _ => unconsume_ret!(self, Ok(Token::Slash)),
                },
                State::Mod => match c{
                    Some('=') => ret = Some(Ok(Token::ModuloEq)),
                    _ => unconsume_ret!(self, Ok(Token::Modulo)),
                },
                State::Equal => match c{
                    Some('=') => ret = Some(Ok(Token::Assignment)),
                    _ => unconsume_ret!(self, Ok(Token::Equals)),
                },
                State::Gt => match c{
                    Some('=') => ret = Some(Ok(Token::GreaterThanEq)),
                    Some('>') => self.state = State::GtGt,
                    _ => unconsume_ret!(self, Ok(Token::GreaterThan)),
                },
                State::GtGt => match c{
                    Some('=') => ret = Some(Ok(Token::ShiftRightEq)),
                    _ => unconsume_ret!(self, Ok(Token::ShiftRight)),
                },
                State::Lt => match c{
                    Some('=') => ret = Some(Ok(Token::LessThanEq)),
                    Some('<') => self.state = State::LtLt,
                    _ => unconsume_ret!(self, Ok(Token::LessThan)),
                },
                State::LtLt => match c{
                    Some('=') => ret = Some(Ok(Token::ShiftLeftEq)),
                    _ => unconsume_ret!(self, Ok(Token::ShiftLeft)),
                },
                State::Not => match c{
                    Some('=') => ret = Some(Ok(Token::NotEquals)),
                    _ => unconsume_ret!(self, Ok(Token::LogicalNot)),
                },
                State::Or => match c{
                    Some('=') => ret = Some(Ok(Token::OrEq)),
                    Some('|') => ret = Some(Ok(Token::LogicalOr)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseOr)),
                },
                State::And => match c{
                    Some('=') => ret = Some(Ok(Token::AndEq)),
                    Some('&') => ret = Some(Ok(Token::LogicalAnd)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseXor)),
                },
                State::Xor => match c{
                    Some('=') => ret = Some(Ok(Token::XorEq)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseXor)),
                },
                State::Dot => match c{
                    Some('.') => self.state = State::DotDot,
                    _ => unconsume_ret!(self, Ok(Token::Dot)),
                },
                State::DotDot => match c{
                    Some('=') => ret = Some(Ok(Token::RangeInclusive)),
                    _ => unconsume_ret!(self, Ok(Token::RangeExclusive)),
                },
                State::Ident => match c{
                    Some(c) if c.is_alphanumeric() => {}
                    _ =>  unconsume_ret!(self, Ok(Token::Ident(&self.str[self.start.offset..self.current.offset]))),
                },
                State::CharLiteral => match c{
                    Some('\'') => ret = Some(Err(TokenizerError::EmptyCharLiteral)),
                    Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    Some(c) => {
                        self.state = State::CharLiteralEnd;
                        char_lit = c;
                    },
                    None => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                },
                State::CharLiteralEnd => match c{
                    Some('\'') => ret = Some(Ok(Token::CharLiteral(char_lit))),
                    None | Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    Some(_) => self.state = State::CharLiteralLarge,
                }
                State::CharLiteralLarge => match c{
                    Some('\'') => ret = Some(Err(TokenizerError::CharLiteralTooBig)),
                    None | Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    _ => {}
                }

                State::String => match c{
                    Some('"') => {
                        let yarn = match self.str_builder.take(){
                            SSBuilder::None => byteyarn::YarnBox::new(""),
                            SSBuilder::Ref(str) => byteyarn::YarnBox::from(str),
                            SSBuilder::Small(small_buf) => byteyarn::YarnBox::new(small_buf.as_str()).immortalize(),
                            SSBuilder::Alloc(string) => byteyarn::Yarn::from_string(string),
                        };
                        ret = Some(Ok(Token::StringLiteral(yarn)))
                    },
                    Some('\\') => {
                        todo!()
                    }
                    Some(c) => match self.str_builder.take(){
                        SSBuilder::None => {
                            self.str_builder = SSBuilder::Ref(&self.str[self.start.offset+'"'.len_utf8()..processing.offset])
                        },
                        SSBuilder::Ref(_) => self.str_builder = SSBuilder::Ref(&self.str[self.start.offset+'"'.len_utf8()..processing.offset]),
                        SSBuilder::Small(mut small) => {
                            if small.write_char(c).is_ok(){
                                self.str_builder = SSBuilder::Small(small);
                            }else{
                                let mut string = small.to_string();
                                string.push(c);
                                self.str_builder = SSBuilder::Alloc(string);
                            }
                        },
                        SSBuilder::Alloc(mut string) => {
                            string.push(c);
                            self.str_builder = SSBuilder::Alloc(string);
                        },
                    },
                    None => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                },
                State::Eof => return None,
            }

            if consume {
                self.chars.next();
                self.current = processing;
            }

            if let Some(ret) = ret {
                match ret {
                    Ok(ok) => {
                        let meta = TokenMeta::start_end(self.start, self.current);
                        self.start = self.current;
                        self.state = ok_ret_state;
                        return Some(Ok(TokenFull::new(ok, meta)));
                    }
                    Err(err) => {
                        let meta = error_meta.unwrap_or(TokenMeta::start_end(self.start, self.current));
                        if update_start_on_error {
                            self.start = self.current;
                        }
                        self.state = err_ret_state;
                        return Some(Err(Box::new(TokenizerErrorFull::new(err, meta))));
                    }
                }
            }
        }
    }
}

#[test]
fn test(){
    let data = r#"
hello :) () [] {} : / 
> >> >= >>=
< << >= <<=
+ +=
- -= 
* *=
/ /=
& && &= 
| || |=
^ ^=
! !=
~
:
;
. .. ..= 
wow... that was aLot
char 'c' literal
'thats now how char literals work'
    '
    '
'
"this is a string :)"
""

"
"
    "' "#;

    let tokenizer = Tokenizer::new(data);

    for token in tokenizer{
        match token{
            Ok(ok) => {
                let repr = &data[ok.meta.offset as usize..(ok.meta.offset+ok.meta.len) as usize];
                println!("{:?} => {:?}", repr, ok)
            },
            Err(err) => {
                let repr = &data[err.meta.offset as usize..(err.meta.offset+err.meta.len) as usize];
                println!("Error {:?}: {:?}",repr,  err)
            },
        }
    }
}