use std::{fmt::Write, iter::Peekable, str::Chars};

use str_buf::StrBuf;

pub use number::*;
pub use token::*;

pub mod number;
pub mod token;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Position {
    offset: usize,
    line: usize,
    col: usize,
}

type TokenizerResult<'a> = Result<TokenFull<'a>, Box<TokenizerErrorFull<'a>>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EscapeReturn {
    String,
    Char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    SingleLine,
    MultiLine(u16),
    MultiLineOpen1(u16),
    MultiLineClose1(u16),

    CharLiteral,
    CharLiteralEnd,
    CharLiteralLarge,

    String,

    EscapeStart(EscapeReturn),

    Eof,
}

pub struct Tokenizer<'a> {
    str: &'a str,
    chars: Peekable<Chars<'a>>,
    state: State,

    start: Position,
    current: Position,
    escape_start: Position,

    str_builder: SSBuilder<'a>,

    include_comments: bool,
}

pub enum SSBuilder<'a> {
    None,
    Ref(&'a str),
    Small(str_buf::StrBuf<15>),
    Alloc(String),
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
        Self {
            str,
            chars: str.chars().peekable(),
            state: State::Default,
            start: Position::default(),
            current: Position::default(),
            escape_start: Position::default(),
            str_builder: SSBuilder::None,
            include_comments: false,
        }
    }

    pub fn include_comments(mut self) -> Self {
        self.include_comments = true;
        self
    }

    fn escape_char_finish(&mut self, char: char, er: EscapeReturn, c_lit_char: &mut char) {
        match er {
            EscapeReturn::String => {
                match self.str_builder.take() {
                    SSBuilder::None => {
                        let mut buf = StrBuf::new();
                        // should never fail
                        buf.write_char(char).unwrap();
                        self.str_builder = SSBuilder::Small(buf)
                    }
                    SSBuilder::Ref(str) => {
                        if str.len() + char.len_utf8() <= 15 {
                            let mut buf = StrBuf::new();
                            // should never fail
                            buf.write_str(str).unwrap();
                            buf.write_char(char).unwrap();
                            self.str_builder = SSBuilder::Small(buf)
                        } else {
                            let mut buf = str.to_string();
                            buf.push(char);
                            self.str_builder = SSBuilder::Alloc(buf);
                        }
                    }
                    SSBuilder::Small(mut buf) => {
                        if buf.write_char(char).is_ok() {
                            self.str_builder = SSBuilder::Small(buf);
                        } else {
                            let mut buf = buf.to_string();
                            buf.push(char);
                            self.str_builder = SSBuilder::Alloc(buf);
                        }
                    }
                    SSBuilder::Alloc(mut buf) => {
                        buf.push(char);
                        self.str_builder = SSBuilder::Alloc(buf);
                    }
                }
                self.state = State::String;
            }
            EscapeReturn::Char => {
                self.state = State::CharLiteralEnd;
                *c_lit_char = char;
            }
        }
    }

    fn unfinished_escape_sequence(
        &mut self,
        ret_state: EscapeReturn,
        processing: &Position,
    ) -> (
        Option<Result<Token<'a>, TokenizerError<'a>>>,
        Option<TokenMeta>,
        State,
    ) {
        let meta = TokenMeta {
            line: self.escape_start.line as u32,
            col: self.escape_start.col as u32,
            offset: self.escape_start.offset as u32,
            len: (processing.offset - self.escape_start.offset) as u32,
        };
        let state = match ret_state {
            EscapeReturn::String => State::String,
            EscapeReturn::Char => State::CharLiteralEnd,
        };
        let ret = Some(Err(TokenizerError::UnfinishedEscapeSequence(
            &self.str[self.escape_start.offset..processing.offset],
        )));
        let error_meta = Some(meta);
        (ret, error_meta, state)
    }

    fn invalid_escape_sequence(
        &mut self,
        ret_state: EscapeReturn,
        processing: &Position,
    ) -> (
        Option<Result<Token<'a>, TokenizerError<'a>>>,
        Option<TokenMeta>,
        State,
    ) {
        let meta = TokenMeta {
            line: self.escape_start.line as u32,
            col: self.escape_start.col as u32,
            offset: self.escape_start.offset as u32,
            len: (processing.offset - self.escape_start.offset) as u32,
        };
        let state = match ret_state {
            EscapeReturn::String => State::String,
            EscapeReturn::Char => State::CharLiteralEnd,
        };
        let ret = Some(Err(TokenizerError::InvalidEscape(
            &self.str[self.escape_start.offset..processing.offset],
        )));
        let error_meta = Some(meta);
        (ret, error_meta, state)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenizerResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut char_lit = '\0';
        let update_start_on_error = true;
        let mut error_meta = None;

        let mut ret = None;
        let ok_ret_state = State::Default;
        let mut err_ret_state = State::Default;
        loop {
            let c = self.chars.peek().copied();
            let mut consume = true;

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
                ($sel:ident, $expr:expr) => {{
                    consume = false;
                    ret = Some($expr);
                }};
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

                    c => ret = Some(Err(TokenizerError::InvalidChar(c))),
                },
                State::Plus => match c {
                    Some('=') => ret = Some(Ok(Token::PlusEq)),
                    _ => unconsume_ret!(self, Ok(Token::Plus)),
                },
                State::Minus => match c {
                    Some('=') => ret = Some(Ok(Token::MinusEq)),
                    _ => unconsume_ret!(self, Ok(Token::Minus)),
                },
                State::Times => match c {
                    Some('=') => ret = Some(Ok(Token::TimesEq)),
                    _ => unconsume_ret!(self, Ok(Token::Star)),
                },
                State::Divide => match c {
                    Some('=') => ret = Some(Ok(Token::DivideEq)),
                    Some('/') => self.state = State::SingleLine,
                    Some('*') => self.state = State::MultiLine(0),
                    _ => unconsume_ret!(self, Ok(Token::Slash)),
                },
                State::Mod => match c {
                    Some('=') => ret = Some(Ok(Token::ModuloEq)),
                    _ => unconsume_ret!(self, Ok(Token::Modulo)),
                },
                State::Equal => match c {
                    Some('=') => ret = Some(Ok(Token::Assignment)),
                    _ => unconsume_ret!(self, Ok(Token::Equals)),
                },
                State::Gt => match c {
                    Some('=') => ret = Some(Ok(Token::GreaterThanEq)),
                    Some('>') => self.state = State::GtGt,
                    _ => unconsume_ret!(self, Ok(Token::GreaterThan)),
                },
                State::GtGt => match c {
                    Some('=') => ret = Some(Ok(Token::ShiftRightEq)),
                    _ => unconsume_ret!(self, Ok(Token::ShiftRight)),
                },
                State::Lt => match c {
                    Some('=') => ret = Some(Ok(Token::LessThanEq)),
                    Some('<') => self.state = State::LtLt,
                    _ => unconsume_ret!(self, Ok(Token::LessThan)),
                },
                State::LtLt => match c {
                    Some('=') => ret = Some(Ok(Token::ShiftLeftEq)),
                    _ => unconsume_ret!(self, Ok(Token::ShiftLeft)),
                },
                State::Not => match c {
                    Some('=') => ret = Some(Ok(Token::NotEquals)),
                    _ => unconsume_ret!(self, Ok(Token::LogicalNot)),
                },
                State::Or => match c {
                    Some('=') => ret = Some(Ok(Token::OrEq)),
                    Some('|') => ret = Some(Ok(Token::LogicalOr)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseOr)),
                },
                State::And => match c {
                    Some('=') => ret = Some(Ok(Token::AndEq)),
                    Some('&') => ret = Some(Ok(Token::LogicalAnd)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseXor)),
                },
                State::Xor => match c {
                    Some('=') => ret = Some(Ok(Token::XorEq)),
                    _ => unconsume_ret!(self, Ok(Token::BitwiseXor)),
                },
                State::Dot => match c {
                    Some('.') => self.state = State::DotDot,
                    _ => unconsume_ret!(self, Ok(Token::Dot)),
                },
                State::DotDot => match c {
                    Some('=') => ret = Some(Ok(Token::RangeInclusive)),
                    _ => unconsume_ret!(self, Ok(Token::RangeExclusive)),
                },
                State::Ident => match c {
                    Some(c) if c.is_alphanumeric() => {}
                    _ => unconsume_ret!(
                        self,
                        Ok(Token::Ident(
                            &self.str[self.start.offset..self.current.offset]
                        ))
                    ),
                },
                State::CharLiteral => match c {
                    Some('\'') => ret = Some(Err(TokenizerError::EmptyCharLiteral)),
                    Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    Some('\\') => {
                        self.escape_start = self.current;
                        self.state = State::EscapeStart(EscapeReturn::Char);
                    },
                    Some(c) => {
                        self.state = State::CharLiteralEnd;
                        char_lit = c;
                    }
                    None => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                },
                State::CharLiteralEnd => match c {
                    Some('\'') => ret = Some(Ok(Token::CharLiteral(char_lit))),
                    None | Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    Some(_) => self.state = State::CharLiteralLarge,
                },
                State::CharLiteralLarge => match c {
                    Some('\'') => ret = Some(Err(TokenizerError::CharLiteralTooBig)),
                    None | Some('\n') => ret = Some(Err(TokenizerError::UnclosedCharLiteral)),
                    _ => {}
                },

                State::String => match c {
                    Some('"') => {
                        let yarn = match self.str_builder.take() {
                            SSBuilder::None => byteyarn::YarnBox::new(""),
                            SSBuilder::Ref(str) => byteyarn::YarnBox::from(str),
                            SSBuilder::Small(small_buf) => {
                                byteyarn::YarnBox::new(small_buf.as_str()).immortalize()
                            }
                            SSBuilder::Alloc(string) => byteyarn::Yarn::from_string(string),
                        };
                        ret = Some(Ok(Token::StringLiteral(yarn)))
                    }
                    Some('\\') => { 
                        self.escape_start = self.current;
                        self.state = State::EscapeStart(EscapeReturn::String);
                    },
                    Some(c) => match self.str_builder.take() {
                        SSBuilder::None => {
                            self.str_builder = SSBuilder::Ref(
                                &self.str[self.start.offset + '"'.len_utf8()..processing.offset],
                            )
                        }
                        SSBuilder::Ref(_) => {
                            self.str_builder = SSBuilder::Ref(
                                &self.str[self.start.offset + '"'.len_utf8()..processing.offset],
                            )
                        }
                        SSBuilder::Small(mut small) => {
                            if small.write_char(c).is_ok() {
                                self.str_builder = SSBuilder::Small(small);
                            } else {
                                let mut string = small.to_string();
                                string.push(c);
                                self.str_builder = SSBuilder::Alloc(string);
                            }
                        }
                        SSBuilder::Alloc(mut string) => {
                            string.push(c);
                            self.str_builder = SSBuilder::Alloc(string);
                        }
                    },
                    None => ret = Some(Err(TokenizerError::UnclosedStringLiteral)),
                },
                State::EscapeStart(ret_state) => {
                    match c {
                        Some('0') => self.escape_char_finish('\0', ret_state, &mut char_lit),
                        Some('n') => self.escape_char_finish('\n', ret_state, &mut char_lit),
                        Some('r') => self.escape_char_finish('\r', ret_state, &mut char_lit),
                        Some('t') => self.escape_char_finish('\t', ret_state, &mut char_lit),
                        Some('\\') => self.escape_char_finish('\\', ret_state, &mut char_lit),
                        Some('\'') => self.escape_char_finish('\'', ret_state, &mut char_lit),
                        Some('"') => self.escape_char_finish('"', ret_state, &mut char_lit),
                        Some(_) => {
                            (ret, error_meta, err_ret_state) =
                                self.invalid_escape_sequence(ret_state, &processing)
                        }

                        None => {
                            (ret, error_meta, err_ret_state) =
                                self.unfinished_escape_sequence(ret_state, &processing)
                        }
                    }
                }
                State::SingleLine => match c {
                    Some('\n') => {
                        ret = Some(Ok(Token::SingleLineComment(
                            &self.str[self.start.offset + 2 * '/'.len_utf8()..self.current.offset],
                        )))
                    }
                    Some(_) => {}
                    None => {
                        ret = Some(Ok(Token::SingleLineComment(
                            &self.str[self.start.offset + 2 * '/'.len_utf8()..self.current.offset],
                        )))
                    }
                },
                State::MultiLine(indent) => match c {
                    Some('/') => self.state = State::MultiLineOpen1(indent),
                    Some('*') => self.state = State::MultiLineClose1(indent),
                    Some(_) => {}
                    None => ret = Some(Err(TokenizerError::UnclosedMultiLineComment)),
                },
                State::MultiLineOpen1(indent) => match c {
                    Some('*') => self.state = State::MultiLine(indent + 1),
                    Some(_) => {}
                    None => ret = Some(Err(TokenizerError::UnclosedMultiLineComment)),
                },
                State::MultiLineClose1(indent) => match (indent, c) {
                    (0, Some('/')) => {
                        ret = Some(Ok(Token::MultiLineComment(
                            &self.str[self.start.offset + ('*'.len_utf8() + '/'.len_utf8())
                                ..processing.offset - ('*'.len_utf8() + '/'.len_utf8())],
                        )))
                    }
                    (indent, Some('/')) => self.state = State::MultiLine(indent - 1),
                    (_, None) => ret = Some(Err(TokenizerError::UnclosedMultiLineComment)),
                    _ => {}
                },
                State::Eof => return None,
            }

            if consume {
                self.chars.next();
                self.current = processing;
            }

            if let Some(ret_res) = ret {
                match ret_res {
                    Ok(token) => {
                        let meta = TokenMeta::start_end(self.start, self.current);
                        self.start = self.current;
                        self.state = ok_ret_state;
                        if matches!(
                            token,
                            Token::MultiLineComment(_) | Token::SingleLineComment(_)
                        ) && !self.include_comments
                        {
                            ret = None;
                            continue;
                        }
                        return Some(Ok(TokenFull::new(token, meta)));
                    }
                    Err(err) => {
                        let meta =
                            error_meta.unwrap_or(TokenMeta::start_end(self.start, self.current));
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
fn test() {
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
char 'c' literal 'w' 'o' 'w'
"this is a string :)"
empty -> ""

'\n' '\0' '\t' '\'' '\"'
"\n \0 \t \' \" \r"

/* milti line 
// comment !?!
    /* even supports internal comments */
*/
/**/
// comment () !!! :)
//comment "#;

    let tokenizer = Tokenizer::new(data).include_comments();
    
    for token in tokenizer {
        match token {
            Ok(ok) => {
                let repr = &data[ok.meta.offset as usize..(ok.meta.offset + ok.meta.len) as usize];
                println!("{:?} => {:?}", repr, ok)
            }
            Err(err) => {
                let repr =
                    &data[err.meta.offset as usize..(err.meta.offset + err.meta.len) as usize];
                println!("Error {:?}: {:?}", repr, err)
            }
        }
    }
}

#[test]
fn test_errors() {
    let data = [
        r#"""#, r#""  "#, r#"'"#, r#"' "#, r#"/*"#, r#"/* *"#, r#"/* "#, r#"/* *"#, r#"'\'"#,
        r#""\"#, r#""\"#,
        r#""\9"#,
    ];

    for data in data {
        println!("{:?}", data);
        let tokenizer = Tokenizer::new(data);

        for token in tokenizer {
            match token {
                Ok(ok) => {
                    let repr =
                        &data[ok.meta.offset as usize..(ok.meta.offset + ok.meta.len) as usize];
                    println!("{:?} => {:?}", repr, ok)
                }
                Err(err) => {
                    let repr =
                        &data[err.meta.offset as usize..(err.meta.offset + err.meta.len) as usize];
                    println!("Error {:?}: {:?}", repr, err)
                }
            }
        }
        println!();
    }
}
