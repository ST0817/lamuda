use chumsky::{Parser, error::Simple, extra::Err, span::Spanned, text::int};

use crate::{diag::Diag, token::Token};

fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Spanned<Token>>, Err<Simple<'src, char>>> {
    int(10)
        .from_str()
        .unwrapped()
        .map(Token::Int)
        .spanned()
        .map(|token| vec![token])
}

pub fn lex(src: &str) -> Result<Vec<Spanned<Token>>, Vec<Diag>> {
    lexer().parse(src).into_result().map_err(|errors| {
        errors
            .iter()
            .map(|error| Diag::new("invalid token".to_string(), *error.span()))
            .collect()
    })
}
