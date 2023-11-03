use crate::tokenizer::{Span, Token, Tokenizer, TokenizerError};

pub mod ast;

struct TokenizerIdk<'a> {
    tokenizer: Tokenizer<'a>,
    errors: Vec<Span<TokenizerError<'a>>>,
}

impl<'a> TokenizerIdk<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer,
            errors: Default::default(),
        }
    }
}

impl<'a> Iterator for TokenizerIdk<'a> {
    type Item = Span<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.tokenizer.next()? {
                Ok(ok) => return Some(ok),
                Err(err) => self.errors.push(*err),
            }
        }
    }
}

struct Parser<'a> {
    tokenizer: TokenizerIdk<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            tokenizer: TokenizerIdk::new(Tokenizer::new(data)),
        }
    }

    pub fn parse(&mut self) -> Result<ast::TokenTree<'a>, ()> {
        todo!()
    }
}

#[test]
fn test() {
    let program = r#"\
        fn test(val: u32, val2: bool){
            let test: u32 = val;
            println(test);
            println(val2);
        }

        fn main(){
            test(12, false);
        }
    "#;
    let mut parser = Parser::new(program);
    match parser.parse() {
        Ok(o) => {
            println!("{:#?}", o)
        }
        Err(_err) => {
            todo!()
        }
    }
}
