use chumsky::{
    IterParser, Parser,
    extra::Err,
    prelude::{Recursive, any, choice, just},
    span::{SimpleSpan, SpanWrap},
    text::{self, ascii::keyword},
};

use crate::{Error, syntax::Syntax, typ::Type};

fn parens<'src, T>(
    parser: impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone {
    parser.delimited_by(just('(').padded(), just(')'))
}

fn int<'src>() -> impl Parser<'src, &'src str, i32, Err<Error<'src>>> + Clone {
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

// Tyoe

fn int_type<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    keyword("Int").map(|_| Type::Int)
}

fn unit_type<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    keyword("Unit").map(|_| Type::Unit)
}

fn fun_type<'src>(
    typ: impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    typ.clone()
        .padded()
        .then_ignore(just("->"))
        .padded()
        .repeated()
        .foldr(typ, |param_type, body_type| Type::Fun {
            param_type: Box::new(param_type),
            body_type: Box::new(body_type),
        })
}

fn typ<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    let mut typ = Recursive::declare();
    typ.define({
        let atom = choice((int_type(), parens(typ.clone()), unit_type()));
        fun_type(atom)
    });
    typ
}

// Syntax

fn int_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    int().map(|value| Syntax::Int { value })
}

fn unit_syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("unit").map(|_| Syntax::Unit)
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
        .then(typ())
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

pub fn syntax<'src>() -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    let mut syntax = Recursive::declare();
    syntax.define({
        let atom = choice((
            int_syntax(),
            parens(syntax.clone()),
            unit_syntax(),
            fun_syntax(syntax.clone()),
            var_syntax(),
        ));
        app_syntax(atom)
    });
    syntax
}
