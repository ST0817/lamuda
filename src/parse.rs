use chumsky::{
    IterParser, Parser,
    extra::Err,
    prelude::{Recursive, any, choice, just},
    span::{SimpleSpan, SpanWrap, Spanned},
    text::{self, ascii::keyword},
};

use crate::{
    Error,
    repl_cmd::ReplCmd,
    syntax::{Command, Syntax},
};

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

struct Param<'src> {
    names: Vec<&'src str>,
    typ: Spanned<Box<Syntax<'src>>>,
}

impl<'src> Param<'src> {
    fn no_name(typ: Spanned<Box<Syntax<'src>>>) -> Self {
        Self {
            names: vec![""],
            typ,
        }
    }
}

fn param<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Param<'src>, Err<Error<'src>>> + Clone {
    let param_inner = name()
        .padded()
        .repeated()
        .at_least(1)
        .collect()
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(syntax.map(Box::new).spanned())
        .map(|(names, typ)| Param { names, typ });
    parens(param_inner)
}

fn params<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
    at_least: usize,
) -> impl Parser<'src, &'src str, Vec<Param<'src>>, Err<Error<'src>>> + Clone {
    param(syntax).repeated().at_least(at_least).collect()
}

fn desugar_fun_params<'src>(params: Vec<Param<'src>>, body: Syntax<'src>) -> Syntax<'src> {
    params.iter().rfold(body, |body, param| {
        param.names.iter().rfold(body, |body, name| Syntax::Fun {
            param_name: name,
            param_type: param.typ.clone(),
            body: Box::new(body),
        })
    })
}

fn desugar_prod_param<'src>(
    param: Spanned<Param<'src>>,
    body_type: Spanned<Syntax<'src>>,
) -> Spanned<Syntax<'src>> {
    param.names.iter().rfold(body_type, |body_type, name| {
        let span: SimpleSpan = (param.span.start..body_type.span.end).into();
        Syntax::Prod {
            param_name: name,
            param_type: param.typ.clone(),
            body_type: Box::new(body_type.inner).with_span(body_type.span),
        }
        .with_span(span)
    })
}

fn fun_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    keyword("fun")
        .padded()
        .ignore_then(params(syntax.clone(), 1))
        .padded()
        .then_ignore(just("=>"))
        .padded()
        .then(syntax)
        .map(|(params, body)| desugar_fun_params(params, body))
}

fn prod_syntax<'src>(
    syntax: impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Syntax<'src>, Err<Error<'src>>> + Clone {
    let no_name_param = syntax.clone().map(Box::new).spanned().map(Param::no_name);
    let param = choice((param(syntax.clone()), no_name_param));
    param
        .spanned()
        .padded()
        .then_ignore(just("->"))
        .padded()
        .repeated()
        .foldr(syntax.spanned(), desugar_prod_param)
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
        .then(params(syntax.clone(), 0))
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(syntax.clone())
        .padded()
        .then_ignore(just(';'))
        .padded()
        .then(syntax.map(Box::new))
        .map(|(((name, params), value), body)| Syntax::Let {
            name,
            value: Box::new(desugar_fun_params(params, value)),
            body,
        })
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

fn def_command<'src>() -> impl Parser<'src, &'src str, Command<'src>, Err<Error<'src>>> + Clone {
    keyword("def")
        .padded()
        .ignore_then(name().spanned())
        .padded()
        .then(params(syntax(), 0))
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(syntax())
        .map(|((name, params), value)| Command::Def {
            name,
            value: desugar_fun_params(params, value),
        })
}

fn inductive_command<'src>() -> impl Parser<'src, &'src str, Command<'src>, Err<Error<'src>>> + Clone
{
    let params = param(syntax())
        .spanned()
        .padded()
        .repeated()
        .collect::<Vec<_>>();
    let ctors = just('|')
        .padded()
        .ignore_then(name().spanned())
        .padded()
        .repeated()
        .collect();
    keyword("inductive")
        .padded()
        .ignore_then(name().spanned())
        .padded()
        .then(params)
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(syntax().spanned())
        .padded()
        .then(ctors)
        .map(|(((name, params), typ), ctors)| Command::Inductive {
            name,
            typ: params
                .into_iter()
                .rfold(typ, |typ, param| desugar_prod_param(param, typ)),
            ctors,
        })
}

fn command<'src>() -> impl Parser<'src, &'src str, Command<'src>, Err<Error<'src>>> + Clone {
    choice((def_command(), inductive_command()))
}

fn command_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone
{
    command().map(|command| ReplCmd::Command { command })
}

fn syntax_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone
{
    syntax().map(|syntax| ReplCmd::Syntax { syntax })
}

pub fn repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> + Clone {
    choice((command_repl_cmd(), syntax_repl_cmd()))
}
