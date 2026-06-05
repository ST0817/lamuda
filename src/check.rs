use crate::{Error, Result, context::Context, syntax::Syntax, term::Term, typ::Type};

pub type TypeContext = Context<Type>;

pub fn check_syntax<'src>(
    syntax: &Syntax<'src>,
    type_context: &TypeContext,
) -> Result<'src, (Term, Type)> {
    match syntax {
        Syntax::Unit => Ok((Term::Unit, Type::Unit)),
        Syntax::Int { value } => Ok((Term::Int { value: *value }, Type::Int)),
        Syntax::Fun {
            param_name,
            param_type,
            body,
        } => {
            let new_type_context = type_context.extend(param_name, param_type.clone());
            let (body_term, body_type) = check_syntax(body, &new_type_context)?;
            let fun_term = Term::Fun {
                param_name: param_name.to_string(),
                body: Box::new(body_term),
            };
            let fun_type = Type::Fun {
                param_type: Box::new(param_type.clone()),
                body_type: Box::new(body_type),
            };
            Ok((fun_term, fun_type))
        }
        Syntax::Var { name } => type_context
            .get(name.inner)
            .map(|typ| {
                (
                    Term::Var {
                        name: name.to_string(),
                    },
                    typ.clone(),
                )
            })
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),
        Syntax::App { callee, arg } => {
            let (callee_term, callee_type) = check_syntax(callee, type_context)?;
            let Type::Fun {
                param_type,
                body_type,
            } = callee_type
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let (arg_term, arg_type) = check_syntax(arg, type_context)?;

            if arg_type != *param_type {
                return Err(vec![Error::custom(arg.span, "type mismatch")]);
            }

            let app_term = Term::App {
                callee: Box::new(callee_term),
                arg: Box::new(arg_term),
            };
            Ok((app_term, *body_type))
        }
    }
}
