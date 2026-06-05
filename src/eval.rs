use std::rc::Rc;

use crate::{context::Context, object::Object, term::Term};

pub type ObjectContext = Context<Object>;

pub fn eval_term(term: &Term, object_context: &ObjectContext) -> Object {
    match term {
        Term::Unit => Object::Unit,
        Term::Int { value } => Object::Int { value: *value },
        Term::Fun { param_name, body } => {
            let object_context = object_context.clone();
            let param_name = param_name.clone();
            let body = body.clone();
            Object::Fun {
                fun: Rc::new(move |param_object| {
                    let new_object_context = object_context.extend(&param_name, param_object);
                    eval_term(&body, &new_object_context)
                }),
            }
        }
        Term::Var { name } => object_context[name].clone(),
        Term::App { callee, arg } => {
            let Object::Fun { fun } = eval_term(callee, object_context) else {
                panic!()
            };
            let arg_object = eval_term(arg, object_context);
            fun(arg_object)
        }
    }
}
