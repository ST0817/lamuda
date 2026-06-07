use std::{cmp::max, rc::Rc};

use chumsky::span::SimpleSpan;

use crate::{
    Error, Result,
    context::Context,
    syntax::Syntax,
    term::{Term, normalize, shift, subst, whnf},
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
            let (param_type_term, param_type_sort) = check_syntax(param_type, context)?;
            check_sort(&param_type_sort, &param_type.span)?;
            let new_type_context = context.extend(param_name, param_type_term.clone());
            let (body_term, body_type) = check_syntax(body, &new_type_context)?;
            let fun_term = Term::Fun {
                param_name: Rc::new(param_name.to_string()),
                param_type: param_type_term.clone(),
                body: body_term,
            };
            let fun_type = Term::Prod {
                param_name: Rc::new(param_name.to_string()),
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
            let (param_type_term, param_type_sort) = check_syntax(param_type, context)?;
            let param_type_sort_level = check_sort(&param_type_sort, &param_type.span)?;
            let new_type_context = context.extend(param_name, param_type_term.clone());
            let (body_type_term, body_type_sort) = check_syntax(body_type, &new_type_context)?;
            let body_type_sort_level = check_sort(&body_type_sort, &body_type.span)?;
            let prod_term = Term::Prod {
                param_name: Rc::new(param_name.to_string()),
                param_type: param_type_term,
                body_type: body_type_term,
            };
            let prod_type = Term::Sort {
                level: max(param_type_sort_level, body_type_sort_level),
            };
            Ok((Rc::new(prod_term), Rc::new(prod_type)))
        }
        Syntax::Var { name } => context
            .get(name.inner)
            .map(|(index, typ)| {
                let var_term = Term::Var {
                    index,
                    name: Rc::new(name.to_string()),
                };
                let var_type = shift(typ, index as isize + 1, 0);
                (Rc::new(var_term), var_type)
            })
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),
        Syntax::App { callee, arg } => {
            let (callee_term, callee_type) = check_syntax(callee, context)?;
            let whnf_callee_type = whnf(&callee_type);
            let Term::Prod {
                param_type,
                body_type,
                ..
            } = whnf_callee_type.as_ref()
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let (arg_term, arg_type) = check_syntax(arg, context)?;

            if normalize(&arg_type) != normalize(&param_type) {
                return Err(vec![Error::custom(
                    arg.span,
                    format!("type mismatch: {arg_type} and {param_type}"),
                )]);
            }

            let app_term = Term::App {
                callee: callee_term,
                arg: arg_term.clone(),
            };
            Ok((Rc::new(app_term), subst(body_type, &arg_term)))
        }
    }
}
