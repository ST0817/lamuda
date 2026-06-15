use std::{cmp::max, rc::Rc};

use chumsky::span::SimpleSpan;

use crate::{
    Error, Result,
    context::{Context, Entry},
    env::Env,
    syntax::{Command, Syntax},
    term::{Term, normalize, shift},
};

fn check_sort<'src>(typ: &Term, span: &SimpleSpan) -> Result<'src, usize> {
    let Term::Sort { level } = typ else {
        return Err(vec![Error::custom(*span, "not a sort")]);
    };
    Ok(*level)
}

pub fn check_syntax<'src>(
    syntax: &Syntax<'src>,
    context: &Context,
    env: &Env,
) -> Result<'src, (Rc<Term>, Rc<Term>)> {
    match syntax {
        Syntax::Sort { level } => Ok((
            Rc::new(Term::Sort { level: *level }),
            Rc::new(Term::Sort { level: level + 1 }),
        )),
        Syntax::Unit => Ok((Rc::new(Term::Unit), Rc::new(Term::UnitType))),
        Syntax::UnitType => Ok((Rc::new(Term::UnitType), Rc::new(Term::Sort { level: 1 }))),
        Syntax::Nat { value } => Ok((Rc::new(Term::Nat { value: *value }), Rc::new(Term::NatType))),
        Syntax::NatType => Ok((Rc::new(Term::NatType), Rc::new(Term::Sort { level: 1 }))),
        Syntax::Fun {
            param_name,
            param_type,
            body,
        } => {
            let (param_type_term, param_type_sort) = check_syntax(param_type, context, env)?;
            check_sort(&param_type_sort, &param_type.span)?;
            let param_name = Rc::new(param_name.to_string());
            let new_context = context.extend(Entry {
                name: param_name.clone(),
                typ: param_type_term.clone(),
            });
            let new_env = env.extend(None);
            let (body_term, body_type) = check_syntax(body, &new_context, &new_env)?;
            let fun_term = Term::Fun {
                param_name: param_name.clone(),
                param_type: param_type_term.clone(),
                body: body_term,
            };
            let fun_type = Term::Prod {
                param_name: param_name.clone(),
                param_type: param_type_term,
                body_type: body_type,
            };
            Ok((Rc::new(fun_term), Rc::new(fun_type)))
        }
        Syntax::Prod {
            param_name,
            param_type,
            body_type,
        } => {
            let (param_type_term, param_type_sort) = check_syntax(param_type, context, env)?;
            let param_type_sort_level = check_sort(&param_type_sort, &param_type.span)?;
            let param_name = Rc::new(param_name.to_string());
            let new_context = context.extend(Entry {
                name: param_name.clone(),
                typ: param_type_term.clone(),
            });
            let new_env = env.extend(None);
            let (body_type_term, body_type_sort) = check_syntax(body_type, &new_context, &new_env)?;
            let body_type_sort_level = check_sort(&body_type_sort, &body_type.span)?;
            let prod_term = Term::Prod {
                param_name,
                param_type: param_type_term,
                body_type: body_type_term,
            };
            let prod_type = Term::Sort {
                level: max(param_type_sort_level, body_type_sort_level),
            };
            Ok((Rc::new(prod_term), Rc::new(prod_type)))
        }
        Syntax::Var { name } => {
            let Some((index, entry)) = context.get(name) else {
                return Err(vec![Error::custom(name.span, "unbound variable")]);
            };
            let var_term = Term::Var {
                index,
                name: entry.name.clone(),
            };
            let var_type = shift(&entry.typ, index as isize + 1, 0);
            Ok((Rc::new(var_term), var_type))
        }
        Syntax::App { callee, arg } => {
            let (callee_term, callee_type) = check_syntax(callee, context, env)?;
            let norm_callee_type = normalize(&callee_type, env);
            let Term::Prod {
                param_type,
                body_type,
                ..
            } = norm_callee_type.as_ref()
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let (arg_term, arg_type) = check_syntax(arg, context, env)?;

            if normalize(&arg_type, env) != normalize(&param_type, env) {
                return Err(vec![Error::custom(
                    arg.span,
                    format!("type mismatch: {arg_type} and {param_type}"),
                )]);
            }

            let app_term = Term::App {
                callee: callee_term,
                arg: arg_term.clone(),
            };
            let new_env = env.extend(Some(arg_term));
            let norm_body_type = shift(&normalize(body_type, &new_env), -1, 0);
            Ok((Rc::new(app_term), norm_body_type))
        }
        Syntax::Let { name, value, body } => {
            let (value_term, value_type) = check_syntax(value, context, env)?;
            let name = Rc::new(name.to_string());
            let new_context = context.extend(Entry {
                name: name.clone(),
                typ: value_type,
            });
            let new_env = env.extend(Some(value_term.clone()));
            let (body_term, body_type) = check_syntax(body, &new_context, &new_env)?;
            let let_term = Term::Let {
                name,
                value: value_term,
                body: body_term,
            };
            Ok((Rc::new(let_term), body_type))
        }
    }
}

pub fn check_command<'src>(
    command: &Command<'src>,
    context: &mut Context,
    env: &mut Env,
) -> Result<'src, ()> {
    match command {
        Command::Def { name, value } => {
            let (value_term, value_type) = check_syntax(value, context, env)?;
            context.push(Entry {
                name: Rc::new(name.to_string()),
                typ: value_type,
            });
            env.push(Some(value_term));
        }
        Command::Inductive { name, typ } => {
            let (type_term, type_sort) = check_syntax(typ, context, env)?;
            check_sort(&type_sort, &typ.span)?;
            context.push(Entry {
                name: Rc::new(name.to_string()),
                typ: type_term,
            });
            env.push(None);
        }
    }
    Ok(())
}
