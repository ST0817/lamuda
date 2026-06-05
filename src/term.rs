#[derive(Clone)]
pub enum Term {
    Unit,
    Int { value: i32 },
    Fun { param_name: String, body: Box<Self> },
    Var { name: String },
    App { callee: Box<Self>, arg: Box<Self> },
}
