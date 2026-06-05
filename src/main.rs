mod check;
mod context;
mod eval;
mod object;
mod parse;
mod syntax;
mod term;
mod typ;

use std::process::ExitCode;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use chumsky::{Parser, error::Rich};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{
    check::{TypeContext, check_syntax},
    eval::{ObjectContext, eval_term},
};

pub type Error<'src> = Rich<'src, char>;
pub type Result<'src, T> = std::result::Result<T, Vec<Error<'src>>>;

const HISTORY_FILE_NAME: &str = ".lamuda_history";
const REPL_ID: &str = "REPL";

fn repl_process<'src>(
    input: &'src str,
    type_context: &TypeContext,
    object_context: &ObjectContext,
) -> Result<'src, ()> {
    let term = parse::syntax().parse(input).into_result()?;
    let (term, typ) = check_syntax(&term, type_context)?;
    let object = eval_term(&term, object_context);
    println!("{object} : {typ}");
    Ok(())
}

fn report_errors<'src>(errors: &Vec<Error<'src>>, id: &str, src: &'src str) {
    for error in errors {
        Report::build(ReportKind::Error, (id, error.span().into_range()))
            .with_config(Config::new().with_index_type(IndexType::Byte))
            .with_message(error.to_string())
            .with_label(
                Label::new((id, error.span().into_range()))
                    .with_message(error.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .print((id, Source::from(src)))
            .unwrap();
    }
}

fn main() -> ExitCode {
    let mut editor = DefaultEditor::new().unwrap();

    if editor.load_history(HISTORY_FILE_NAME).is_err() {
        println!("no previous history")
    }

    let type_context = TypeContext::new();
    let object_context = ObjectContext::new();

    loop {
        match editor.readline("❯ ") {
            Ok(input) if input.is_empty() => {}
            Ok(input) => {
                editor.add_history_entry(&input).unwrap();

                if let Err(errors) = repl_process(&input, &type_context, &object_context) {
                    report_errors(&errors, REPL_ID, &input);
                }
            }
            Err(ReadlineError::Eof) => {
                editor.append_history(HISTORY_FILE_NAME).unwrap();
                break ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("Error: {error}");
                editor.append_history(HISTORY_FILE_NAME).unwrap();
                break ExitCode::FAILURE;
            }
        }
    }
}
