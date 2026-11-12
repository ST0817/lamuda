use chumsky::{
    Parser,
    error::Rich,
    extra::Err,
    input::{Input, ValueInput},
    select,
    span::{SimpleSpan, Spanned},
    text::TextExpected,
};

use crate::{diag::Diag, syntax::Expr, token::Token};

fn parser<'tokens, I>() -> impl Parser<'tokens, I, Expr, Err<Rich<'tokens, Token>>>
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! { Token::Int(value) => Expr::Int(value) }.labelled(TextExpected::<()>::Int)
}

pub fn parse(tokens: &[Spanned<Token>], src: &str) -> Result<Expr, Vec<Diag>> {
    let input = tokens.split_spanned((src.len()..src.len()).into());
    parser().parse(input).into_result().map_err(|errors| {
        errors
            .iter()
            .map(|error| Diag::new(error.to_string(), *error.span()))
            .collect()
    })
}
