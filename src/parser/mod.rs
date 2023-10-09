use crate::tokenizer::Tokenizer;

pub mod ast;



struct Parser<'a>{
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a>{
    pub fn new(data: &'a str) -> Self{
        Self { 
            tokenizer: Tokenizer::new(data), 
        }
    }

    pub fn parse(&mut self) -> Result<ast::TokenTree, ()>{
        todo!()
    }
}

#[test]
fn test(){
    let program = r#"\
        fn test(val: u32, val2: bool){
            let test: u32 = val;
            println(test);
            println(val2);
        }
    "#;
    let mut parser = Parser::new(program);
    match parser.parse(){
        Ok(o) => {
            println!("{:#?}", o)
        },
        Err(_err) => {
            todo!()
        },
    }
}
