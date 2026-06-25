use std::{cmp::max, rc::Rc, vec};

use chumsky::span::{SimpleSpan, Spanned};

use crate::{
    Error, Result,
    context::{Const, ConstValue, GlobalContext, LocalContext, RecRule, Var},
    env::Env,
    syntax::{Command, Ctor, Syntax},
    term::Term,
};

fn shift(term: &Rc<Term>, value: isize, cutoff: usize) -> Rc<Term> {
    match term.as_ref() {
        Term::Var { index, name } if *index >= cutoff => Rc::new(Term::Var {
            index: (*index as isize + value) as usize,
            name: name.clone(),
        }),
        Term::Fun {
            param_name,
            param_type,
            body,
        } => Rc::new(Term::Fun {
            param_name: param_name.clone(),
            param_type: shift(param_type, value, cutoff),
            body: shift(body, value, cutoff + 1),
        }),
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => Rc::new(Term::Prod {
            param_name: param_name.clone(),
            param_type: shift(param_type, value, cutoff),
            body_type: shift(body_type, value, cutoff + 1),
        }),
        Term::App { callee, arg } => Rc::new(Term::App {
            callee: shift(callee, value, cutoff),
            arg: shift(arg, value, cutoff),
        }),
        _ => term.clone(),
    }
}

fn split_app(term: &Rc<Term>) -> (Rc<Term>, Vec<Rc<Term>>) {
    let mut head = term.clone();
    let mut args = Vec::new();
    while let Term::App { callee, arg } = head.as_ref() {
        args.push(arg.clone());
        head = callee.clone();
    }
    (head, args)
}

fn try_rec(term: &Rc<Term>, global_context: &GlobalContext, env: &Env) -> Option<Rc<Term>> {
    let (callee, args) = split_app(term);
    let Term::Const { name } = callee.as_ref() else {
        return None;
    };
    let Const {
        value: ConstValue::Rec { rule },
        ..
    } = global_context.get(name)?
    else {
        return None;
    };
    let ctors_count = rule.ctors.len();
    if args.len() < ctors_count + 2 {
        return None;
    }
    let major_promise = normalize(&args[ctors_count + 1], global_context, env);
    let Term::Const { name: ctor_name } = major_promise.as_ref() else {
        return None;
    };
    let ctor_index = rule.ctors.iter().position(|name| name == ctor_name)?;
    Some(args[ctor_index + 1].clone())
}

pub fn normalize(term: &Rc<Term>, global_context: &GlobalContext, env: &Env) -> Rc<Term> {
    match term.as_ref() {
        Term::Var { index, .. } if let Some(value) = &env[*index] => {
            shift(value, *index as isize + 1, 0)
        }
        Term::Fun {
            param_name,
            param_type,
            body,
        } => {
            let norm_param_type = normalize(param_type, global_context, env);
            let new_env = env.extend(None);
            Rc::new(Term::Fun {
                param_name: param_name.clone(),
                param_type: norm_param_type,
                body: normalize(body, global_context, &new_env),
            })
        }
        Term::Prod {
            param_name,
            param_type,
            body_type,
        } => {
            let norm_param_type = normalize(param_type, global_context, env);
            let new_env = env.extend(None);
            Rc::new(Term::Prod {
                param_name: param_name.clone(),
                param_type: norm_param_type,
                body_type: normalize(body_type, global_context, &new_env),
            })
        }
        Term::App { callee, arg } => {
            let norm_callee = normalize(callee, global_context, env);
            let norm_arg = normalize(arg, global_context, env);
            match norm_callee.as_ref() {
                Term::Fun { body, .. } => {
                    let new_env = env.extend(Some(norm_arg));
                    shift(&normalize(body, global_context, &new_env), -1, 0)
                }
                _ => {
                    let app_term = Rc::new(Term::App {
                        callee: norm_callee.clone(),
                        arg: norm_arg,
                    });
                    if let Some(reduced) = try_rec(&app_term, global_context, env) {
                        normalize(&reduced, global_context, env)
                    } else {
                        app_term
                    }
                }
            }
        }
        Term::Let { value, body, .. } => {
            let norm_value = normalize(value, global_context, env);
            let new_env = env.extend(Some(norm_value));
            normalize(body, global_context, &new_env)
        }
        _ => term.clone(),
    }
}

fn get_var<'src>(
    name: &'src str,
    span: &SimpleSpan,
    global_context: &GlobalContext,
    local_context: &LocalContext,
) -> Result<'src, (Rc<Term>, Rc<Term>)> {
    if let Some((index, entry)) = local_context.get(name) {
        let var_term = Term::Var {
            index,
            name: entry.name.clone(),
        };
        let var_type = shift(&entry.typ, index as isize + 1, 0);
        Ok((Rc::new(var_term), var_type))
    } else if let Some(cnst) = global_context.get(name) {
        let const_term = match &cnst.value {
            ConstValue::Def { value } => value.clone(),
            _ => Rc::new(Term::Const {
                name: name.to_string(),
            }),
        };
        Ok((const_term, cnst.typ.clone()))
    } else {
        Err(vec![Error::custom(*span, "unbound variable")])
    }
}

fn check_sort<'src>(typ: &Term, span: &SimpleSpan) -> Result<'src, usize> {
    let Term::Sort { level } = typ else {
        return Err(vec![Error::custom(*span, "not a sort")]);
    };
    Ok(*level)
}

pub fn check_syntax<'src>(
    syntax: &Syntax<'src>,
    global_context: &GlobalContext,
    local_context: &LocalContext,
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
            let (param_type_term, param_type_sort) =
                check_syntax(param_type, global_context, local_context, env)?;
            check_sort(&param_type_sort, &param_type.span)?;
            let new_context = local_context.extend(Var {
                name: param_name.to_string(),
                typ: param_type_term.clone(),
            });
            let new_env = env.extend(None);
            let (body_term, body_type) =
                check_syntax(body, global_context, &new_context, &new_env)?;
            let fun_term = Term::Fun {
                param_name: param_name.to_string(),
                param_type: param_type_term.clone(),
                body: body_term,
            };
            let fun_type = Term::Prod {
                param_name: param_name.to_string(),
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
            let (param_type_term, param_type_sort) =
                check_syntax(param_type, global_context, local_context, env)?;
            let param_type_sort_level = check_sort(&param_type_sort, &param_type.span)?;
            let new_context = local_context.extend(Var {
                name: param_name.to_string(),
                typ: param_type_term.clone(),
            });
            let new_env = env.extend(None);
            let (body_type_term, body_type_sort) =
                check_syntax(body_type, global_context, &new_context, &new_env)?;
            let body_type_sort_level = check_sort(&body_type_sort, &body_type.span)?;
            let prod_term = Term::Prod {
                param_name: param_name.to_string(),
                param_type: param_type_term,
                body_type: body_type_term,
            };
            let prod_type = Term::Sort {
                level: max(param_type_sort_level, body_type_sort_level),
            };
            Ok((Rc::new(prod_term), Rc::new(prod_type)))
        }
        Syntax::Var { name } => get_var(name, &name.span, global_context, local_context),
        Syntax::App { callee, arg } => {
            let (callee_term, callee_type) =
                check_syntax(callee, global_context, local_context, env)?;
            let norm_callee_type = normalize(&callee_type, global_context, env);
            let Term::Prod {
                param_type,
                body_type,
                ..
            } = norm_callee_type.as_ref()
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let (arg_term, arg_type) = check_syntax(arg, global_context, local_context, env)?;

            if normalize(&arg_type, global_context, env)
                != normalize(&param_type, global_context, env)
            {
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
            let norm_body_type = shift(&normalize(body_type, global_context, &new_env), -1, 0);
            Ok((Rc::new(app_term), norm_body_type))
        }
        Syntax::Let { name, value, body } => {
            let (value_term, value_type) = check_syntax(value, global_context, local_context, env)?;
            let new_context = local_context.extend(Var {
                name: name.to_string(),
                typ: value_type,
            });
            let new_env = env.extend(Some(value_term.clone()));
            let (body_term, body_type) =
                check_syntax(body, global_context, &new_context, &new_env)?;
            let let_term = Term::Let {
                name: name.to_string(),
                value: value_term,
                body: body_term,
            };
            Ok((Rc::new(let_term), body_type))
        }
    }
}

fn define_const<'src>(
    name: String,
    cnst: Const,
    span: &SimpleSpan,
    global_context: &mut GlobalContext,
) -> Result<'src, ()> {
    let None = global_context.get(&name) else {
        return Err(vec![Error::custom(
            *span,
            format!("{name} is already defined"),
        )]);
    };
    global_context.insert(name, cnst);
    Ok(())
}

fn check_def_command<'src>(
    name: &Spanned<&'src str>,
    value: &Syntax<'src>,
    global_context: &mut GlobalContext,
    local_context: &LocalContext,
    env: &Env,
) -> Result<'src, ()> {
    let (value_term, value_type) = check_syntax(value, &global_context, local_context, env)?;
    let def_const = Const {
        typ: value_type,
        value: ConstValue::Def { value: value_term },
    };
    define_const(name.to_string(), def_const, &name.span, global_context)?;
    Ok(())
}

fn split_ctor_type(ctor_type: &Rc<Term>) -> (Vec<Rc<Term>>, Rc<Term>) {
    let mut field_types = Vec::new();
    let mut result_type = ctor_type.clone();
    while let Term::Prod {
        param_type,
        body_type,
        ..
    } = result_type.as_ref()
    {
        field_types.push(param_type.clone());
        result_type = body_type.clone();
    }
    (field_types, result_type)
}

fn check_field_positivity<'src>(
    ind_type: &Rc<Term>,
    field_type: &Rc<Term>,
    span: &SimpleSpan,
) -> Result<'src, ()> {
    if let Term::Prod { param_type, .. } = field_type.as_ref() {
        check_field_positivity(ind_type, param_type, span)?;
    } else if field_type == ind_type {
        return Err(vec![Error::custom(
            *span,
            "found invalid occurrence of inductive type",
        )]);
    }
    Ok(())
}

fn check_field<'src>(
    ind_type: &Rc<Term>,
    field_type: &Rc<Term>,
    span: &SimpleSpan,
) -> Result<'src, ()> {
    if let Term::Prod { param_type, .. } = field_type.as_ref() {
        check_field_positivity(ind_type, param_type, span)
    } else {
        Ok(())
    }
}

fn check_ctor_type<'src>(
    ind_type: &Rc<Term>,
    ctor_type: &Rc<Term>,
    span: &SimpleSpan,
) -> Result<'src, ()> {
    let (field_types, result_type) = split_ctor_type(ctor_type);

    for field_type in field_types {
        check_field(ind_type, &field_type, span)?;
    }

    let (head_type, _) = split_app(&result_type);
    if head_type != *ind_type {
        return Err(vec![Error::custom(
            *span,
            "unecpected resulting type of constructor",
        )]);
    }

    Ok(())
}

fn check_ctor<'src>(
    ctor: &Ctor<'src>,
    ind_type: &Rc<Term>,
    global_context: &GlobalContext,
    local_context: &LocalContext,
    env: &Env,
) -> Result<'src, Const> {
    let (ctor_type_term, ctor_type_sort) =
        check_syntax(&ctor.typ, global_context, local_context, env)?;
    check_sort(&ctor_type_sort, &ctor.typ.span)?;
    check_ctor_type(ind_type, &ctor_type_term, &ctor.typ.span)?;
    let ctor_const = Const {
        typ: ctor_type_term,
        value: ConstValue::Ctor,
    };
    Ok(ctor_const)
}

fn create_rec_type(ind_type: &Rc<Term>, rec_rule: &RecRule) -> Term {
    let motive_type = Term::Prod {
        param_name: "".to_string(),
        param_type: ind_type.clone(),
        body_type: Rc::new(Term::Sort { level: 1 }),
    };
    let mut rec_type = Rc::new(Term::Prod {
        param_name: "t".to_string(),
        param_type: ind_type.clone(),
        body_type: Rc::new(Term::App {
            callee: Rc::new(Term::Var {
                index: rec_rule.ctors.len() + 1,
                name: "motive".to_string(),
            }),
            arg: Rc::new(Term::Var {
                index: 0,
                name: "t".to_string(),
            }),
        }),
    });
    for (motive_index, ctor) in rec_rule.ctors.iter().enumerate().rev() {
        rec_type = Rc::new(Term::Prod {
            param_name: "".to_string(),
            param_type: Rc::new(Term::App {
                callee: Rc::new(Term::Var {
                    index: motive_index,
                    name: "motive".to_string(),
                }),
                arg: Rc::new(Term::Const {
                    name: ctor.to_string(),
                }),
            }),
            body_type: rec_type,
        });
    }
    Term::Prod {
        param_name: "motive".to_string(),
        param_type: Rc::new(motive_type),
        body_type: rec_type,
    }
}

fn check_rec<'src>(ind_type: &Rc<Term>, ctors: &Vec<Ctor>) -> Result<'src, Const> {
    let rec_rule = RecRule {
        ctors: ctors.iter().map(|ctor| ctor.name.to_string()).collect(),
    };
    let rec_type = create_rec_type(ind_type, &rec_rule);
    Ok(Const {
        typ: Rc::new(rec_type),
        value: ConstValue::Rec { rule: rec_rule },
    })
}

fn check_inductive_command<'src>(
    name: &Spanned<&'src str>,
    typ: &Spanned<Syntax<'src>>,
    ctors: &Vec<Ctor<'src>>,
    global_context: &mut GlobalContext,
    local_context: &LocalContext,
    env: &Env,
) -> Result<'src, ()> {
    let (type_term, type_sort) = check_syntax(typ, &global_context, local_context, env)?;
    check_sort(&type_sort, &typ.span)?;
    let cnst = Const {
        typ: type_term,
        value: ConstValue::Ind,
    };
    define_const(name.to_string(), cnst, &name.span, global_context)?;

    let ind_type = Rc::new(Term::Const {
        name: name.to_string(),
    });
    for ctor in ctors {
        let ctor_const = check_ctor(ctor, &ind_type, &global_context, local_context, env)?;
        define_const(
            ctor.name.to_string(),
            ctor_const,
            &ctor.name.span,
            global_context,
        )?;
    }

    let rec_const = check_rec(&ind_type, ctors)?;
    define_const(
        format!("rec{}", name.inner),
        rec_const,
        &name.span,
        global_context,
    )?;

    Ok(())
}

pub fn check_command<'src>(
    command: &Command<'src>,
    global_context: &GlobalContext,
    local_context: &mut LocalContext,
    env: &mut Env,
) -> Result<'src, GlobalContext> {
    let mut new_global_cnontext = global_context.clone();
    match command {
        Command::Def { name, value } => {
            check_def_command(name, value, &mut new_global_cnontext, local_context, env)?
        }
        Command::Inductive { name, typ, ctors } => check_inductive_command(
            name,
            typ,
            ctors,
            &mut new_global_cnontext,
            local_context,
            env,
        )?,
    }
    Ok(new_global_cnontext)
}
