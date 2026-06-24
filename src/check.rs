use std::{cmp::max, rc::Rc, vec};

use chumsky::span::SimpleSpan;

use crate::{
    Error, Result,
    context::{Const, ConstValue, GlobalContext, LocalContext, RecRule, Var},
    env::Env,
    syntax::{Command, Syntax},
    term::{Term, shift, spine},
};

pub struct Checker {
    global_context: GlobalContext,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            global_context: GlobalContext::new(),
        }
    }

    fn try_rec(&self, term: &Rc<Term>, env: &Env) -> Option<Rc<Term>> {
        let (callee, args) = spine(term);
        let Term::Const { name } = callee.as_ref() else {
            return None;
        };
        let Const {
            value: ConstValue::Rec { rule },
            ..
        } = self.global_context.get(name)?
        else {
            return None;
        };
        let ctors_count = rule.ctors.len();
        if args.len() < ctors_count + 2 {
            return None;
        }
        let major_promise = self.normalize(&args[ctors_count + 1], env);
        let Term::Const { name: ctor_name } = major_promise.as_ref() else {
            return None;
        };
        let ctor_index = rule.ctors.iter().position(|name| name == ctor_name)?;
        Some(args[ctor_index + 1].clone())
    }

    pub fn normalize(&self, term: &Rc<Term>, env: &Env) -> Rc<Term> {
        match term.as_ref() {
            Term::Var { index, .. } if let Some(value) = &env[*index] => {
                shift(value, *index as isize + 1, 0)
            }
            Term::Fun {
                param_name,
                param_type,
                body,
            } => {
                let norm_param_type = self.normalize(param_type, env);
                let new_env = env.extend(None);
                Rc::new(Term::Fun {
                    param_name: param_name.clone(),
                    param_type: norm_param_type,
                    body: self.normalize(body, &new_env),
                })
            }
            Term::Prod {
                param_name,
                param_type,
                body_type,
            } => {
                let norm_param_type = self.normalize(param_type, env);
                let new_env = env.extend(None);
                Rc::new(Term::Prod {
                    param_name: param_name.clone(),
                    param_type: norm_param_type,
                    body_type: self.normalize(body_type, &new_env),
                })
            }
            Term::App { callee, arg } => {
                let norm_callee = self.normalize(callee, env);
                let norm_arg = self.normalize(arg, env);
                match norm_callee.as_ref() {
                    Term::Fun { body, .. } => {
                        let new_env = env.extend(Some(norm_arg));
                        shift(&self.normalize(body, &new_env), -1, 0)
                    }
                    _ => {
                        let app_term = Rc::new(Term::App {
                            callee: norm_callee.clone(),
                            arg: arg.clone(),
                        });
                        if let Some(reduced) = self.try_rec(&app_term, env) {
                            self.normalize(&reduced, env)
                        } else {
                            app_term
                        }
                    }
                }
            }
            Term::Let { value, body, .. } => {
                let norm_value = self.normalize(value, env);
                let new_env = env.extend(Some(norm_value));
                self.normalize(body, &new_env)
            }
            _ => term.clone(),
        }
    }

    fn get_var<'src>(
        &self,
        name: &'src str,
        span: &SimpleSpan,
        local_context: &LocalContext,
    ) -> Result<'src, (Rc<Term>, Rc<Term>)> {
        if let Some((index, entry)) = local_context.get(name) {
            let var_term = Term::Var {
                index,
                name: entry.name.clone(),
            };
            let var_type = shift(&entry.typ, index as isize + 1, 0);
            Ok((Rc::new(var_term), var_type))
        } else if let Some(cnst) = self.global_context.get(name) {
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

    fn check_sort<'src>(&self, typ: &Term, span: &SimpleSpan) -> Result<'src, usize> {
        let Term::Sort { level } = typ else {
            return Err(vec![Error::custom(*span, "not a sort")]);
        };
        Ok(*level)
    }

    pub fn check_syntax<'src>(
        &self,
        syntax: &Syntax<'src>,
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
            Syntax::Nat { value } => {
                Ok((Rc::new(Term::Nat { value: *value }), Rc::new(Term::NatType)))
            }
            Syntax::NatType => Ok((Rc::new(Term::NatType), Rc::new(Term::Sort { level: 1 }))),
            Syntax::Fun {
                param_name,
                param_type,
                body,
            } => {
                let (param_type_term, param_type_sort) =
                    self.check_syntax(param_type, local_context, env)?;
                self.check_sort(&param_type_sort, &param_type.span)?;
                let new_context = local_context.extend(Var {
                    name: param_name.to_string(),
                    typ: param_type_term.clone(),
                });
                let new_env = env.extend(None);
                let (body_term, body_type) = self.check_syntax(body, &new_context, &new_env)?;
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
                    self.check_syntax(param_type, local_context, env)?;
                let param_type_sort_level = self.check_sort(&param_type_sort, &param_type.span)?;
                let new_context = local_context.extend(Var {
                    name: param_name.to_string(),
                    typ: param_type_term.clone(),
                });
                let new_env = env.extend(None);
                let (body_type_term, body_type_sort) =
                    self.check_syntax(body_type, &new_context, &new_env)?;
                let body_type_sort_level = self.check_sort(&body_type_sort, &body_type.span)?;
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
            Syntax::Var { name } => self.get_var(name, &name.span, local_context),
            Syntax::App { callee, arg } => {
                let (callee_term, callee_type) = self.check_syntax(callee, local_context, env)?;
                let norm_callee_type = self.normalize(&callee_type, env);
                let Term::Prod {
                    param_type,
                    body_type,
                    ..
                } = norm_callee_type.as_ref()
                else {
                    return Err(vec![Error::custom(callee.span, "not a function")]);
                };
                let (arg_term, arg_type) = self.check_syntax(arg, local_context, env)?;

                if self.normalize(&arg_type, env) != self.normalize(&param_type, env) {
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
                let norm_body_type = shift(&self.normalize(body_type, &new_env), -1, 0);
                Ok((Rc::new(app_term), norm_body_type))
            }
            Syntax::Let { name, value, body } => {
                let (value_term, value_type) = self.check_syntax(value, local_context, env)?;
                let new_context = local_context.extend(Var {
                    name: name.to_string(),
                    typ: value_type,
                });
                let new_env = env.extend(Some(value_term.clone()));
                let (body_term, body_type) = self.check_syntax(body, &new_context, &new_env)?;
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
        &mut self,
        name: String,
        cnst: Const,
        span: &SimpleSpan,
    ) -> Result<'src, ()> {
        let None = self.global_context.get(&name) else {
            return Err(vec![Error::custom(*span, "already defined")]);
        };
        self.global_context.insert(name, cnst);
        Ok(())
    }

    fn create_rec_type(&self, ind_type: &Rc<Term>, rec_rule: &RecRule) -> Term {
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

    pub fn check_command<'src>(
        &mut self,
        command: &Command<'src>,
        local_context: &mut LocalContext,
        env: &mut Env,
    ) -> Result<'src, ()> {
        match command {
            Command::Def { name, value } => {
                let (value_term, value_type) = self.check_syntax(value, local_context, env)?;
                let def_const = Const {
                    typ: value_type,
                    value: ConstValue::Def { value: value_term },
                };
                self.define_const(name.to_string(), def_const, &name.span)
            }
            Command::Inductive { name, typ, ctors } => {
                let (type_term, type_sort) = self.check_syntax(typ, local_context, env)?;
                self.check_sort(&type_sort, &typ.span)?;
                if ctors.len() > 0 {
                    let Term::Sort { .. } = type_term.as_ref() else {
                        return Err(vec![Error::custom(
                            name.span,
                            "unexpected resulting type for constructor",
                        )]);
                    };
                }
                let cnst = Const {
                    typ: type_term,
                    value: ConstValue::Ind,
                };
                self.define_const(name.to_string(), cnst, &name.span)?;

                let ind_type = Rc::new(Term::Const {
                    name: name.to_string(),
                });
                for ctor in ctors {
                    let ctor_const = Const {
                        typ: ind_type.clone(),
                        value: ConstValue::Ctor,
                    };
                    self.define_const(ctor.to_string(), ctor_const, &ctor.span)?;
                }

                let rec_rule = RecRule {
                    ctors: ctors.iter().map(|ctor| ctor.to_string()).collect(),
                };
                let rec_type = self.create_rec_type(&ind_type, &rec_rule);
                self.define_const(
                    format!("rec{}", name.inner),
                    Const {
                        typ: Rc::new(rec_type),
                        value: ConstValue::Rec { rule: rec_rule },
                    },
                    &name.span,
                )?;

                Ok(())
            }
        }
    }
}
