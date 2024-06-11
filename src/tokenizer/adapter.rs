pub struct Lexer<'input>(super::Tokenizer<'input>);

impl<'input> Lexer<'input> {
    pub fn new(str: &'input str) -> Self {
        Self(super::Tokenizer::new(str))
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

impl<'input> Iterator for Lexer<'input> {
    type Item =
        Spanned<super::Token<'input>, usize, Box<super::Span<super::TokenizerError<'input>>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next()?;
        match next {
            Ok(ok) => {
                return Some(Ok((
                    ok.span.offset as usize,
                    ok.val,
                    ok.span.offset as usize + ok.span.len as usize,
                )));
            }
            Err(err) => return Some(Err(err)),
        }
    }
}
