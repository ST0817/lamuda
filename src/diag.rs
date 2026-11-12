use chumsky::span::SimpleSpan;

pub struct Diag {
    message: String,
    span: SimpleSpan,
}

impl Diag {
    pub fn new(message: String, span: SimpleSpan) -> Self {
        Self { message, span }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> &SimpleSpan {
        &self.span
    }
}
