use chumsky::{
    IterParser, Parser,
    extra::Err,
    prelude::{Recursive, any, choice, just},
    span::{SimpleSpan, SpanWrap},
    text::{self, ascii::keyword},
};

use crate::{Error, repl_cmd::ReplCmd, syntax::Syntax};

fn parens<'src, T>(
    parser: impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone {
    parser.delimited_by(just('(').padded(), just(')'))
}

fn nat<'src>() -> impl Parser<'src, &'src str, usize, Err<Error<'src>>> + Clone {
    text::int(10).from_str().unwrapped()
}

fn alpha<'src>() -> impl Parser<'src, &'src str, char, Err<Error<'src>>> + Clone {
    any().filter(char::is_ascii_alphabetic)
}

fn alphanum<'src>() -> impl Parser<'src, &'src str, char, Err<Error<'src>>> + Clone {
    any().filter(char::is_ascii_alphanumeric)
}

fn name<'src>() -> impl Parser<'src, &'src str, &'src str, Err<Error<'src>>> + Clone {
    alpha()
        .repeated()
        .at_least(1)
        .ignore_then(alphanum().repeated())
        .to_slice()
}

// Syntax

fn nat_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    nat().map(|value| Syntax::Nat { value })
}

fn sort_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("Sort")
        .padded()
        .ignore_then(nat())
        .map(|level| Syntax::Sort { level })
}

fn prop_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("Prop").map(|_| Syntax::Sort { level: 0 })
}

fn type_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("Type")
        .padded()
        .ignore_then(nat().or_not())
        .map(|level| Syntax::Sort {
            level: level.unwrap_or_default() + 1,
        })
}

fn unit_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("unit").map(|_| Syntax::Unit)
}

fn unit_type_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone
{
    keyword("Unit").map(|_| Syntax::UnitType)
}

fn nat_type_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("Nat").map(|_| Syntax::NatType)
}

fn fun_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("fun")
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(syntax.clone().map(Box::new).spanned())
        .padded()
        .then_ignore(just("=>"))
        .padded()
        .then(syntax.map(Box::new))
        .map(|((param_name, param_type), body)| Syntax::Fun {
            param_name,
            param_type,
            body,
        })
}

fn prod_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    let named_param = parens(
        name()
            .padded()
            .then_ignore(just(':'))
            .padded()
            .then(syntax.clone().spanned()),
    );
    let no_name_param = syntax.clone().spanned().map(|param_type| ("", param_type));
    let param = choice((named_param, no_name_param));
    param
        .spanned()
        .padded()
        .then_ignore(just("->"))
        .padded()
        .repeated()
        .foldr(syntax.spanned(), |param, body_type| {
            let (param_name, param_type) = param.inner;
            let span: SimpleSpan = (param.span.start..body_type.span.end).into();
            Syntax::Prod {
                param_name,
                param_type: Box::new(param_type.inner).with_span(param_type.span),
                body_type: Box::new(body_type.inner).with_span(body_type.span),
            }
            .with_span(span)
        })
        .map(|spanned| spanned.inner)
}

fn var_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    name().spanned().map(|name| Syntax::Var { name })
}

fn app_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    syntax
        .clone()
        .spanned()
        .padded()
        .foldl(syntax.spanned().padded().repeated(), |callee, arg| {
            let span: SimpleSpan = (callee.span.start..arg.span.end).into();
            Syntax::App {
                callee: Box::new(callee.inner).with_span(callee.span),
                arg: Box::new(arg.inner).with_span(arg.span),
            }
            .with_span(span)
        })
        .map(|spanned| spanned.inner)
}

fn let_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("let")
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(syntax.clone().map(Box::new))
        .padded()
        .then_ignore(just(';'))
        .padded()
        .then(syntax.map(Box::new))
        .map(|((name, value), body)| Syntax::Let { name, value, body })
}

fn syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    let mut syntax = Recursive::declare();
    syntax.define({
        let atom = choice((
            nat_syntax(),
            parens(syntax.clone()),
            prop_syntax(),
            sort_syntax(),
            type_syntax(),
            unit_syntax(),
            unit_type_syntax(),
            nat_type_syntax(),
            fun_syntax(syntax.clone()),
            let_syntax(syntax.clone()),
            var_syntax(),
        ));
        let app = app_syntax(atom);
        prod_syntax(app)
    });
    syntax
}

fn def_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone {
    just(':')
        .padded()
        .ignore_then(keyword("def"))
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(syntax())
        .map(|(name, syntax)| ReplCmd::Def { name, syntax })
}

fn syntax_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone
{
    syntax().map(|syntax| ReplCmd::Syntax { syntax })
}

pub fn repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone {
    choice((def_repl_cmd(), syntax_repl_cmd()))
}
