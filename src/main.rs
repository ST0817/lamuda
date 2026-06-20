mod check;
mod context;
mod env;
mod parse;
mod repl_cmd;
mod syntax;
mod term;

use std::process::ExitCode;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use chumsky::{Parser, error::Rich};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{check::Checker, context::LocalContext, env::Env, repl_cmd::ReplCmd, term::normalize};

pub type Error<'src> = Rich<'src, char>;
pub type Result<'src, T> = std::result::Result<T, Vec<Error<'src>>>;

const HISTORY_FILE_NAME: &str = ".lamuda_history";
const REPL_ID: &str = "REPL";

fn repl_process<'src>(
    input: &'src str,
    checker: &mut Checker,
    local_context: &mut LocalContext,
    env: &mut Env,
) -> Result<'src, ()> {
    match parse::repl_cmd().parse(input).into_result()? {
        ReplCmd::Command { command } => checker.check_command(&command, local_context, env)?,
        ReplCmd::Syntax { syntax } => {
            let (term, typ) = checker.check_syntax(&syntax, local_context, env)?;
            let norm_term = normalize(&term, env);
            println!("{norm_term}");
            println!(": {typ}");
        }
    }
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

    let mut checker = Checker::new();
    let mut local_context = LocalContext::new();
    let mut env = Env::new();

    loop {
        match editor.readline("❯ ") {
            Ok(input) if input.is_empty() => {}
            Ok(input) => {
                editor.add_history_entry(&input).unwrap();

                if let Err(errors) =
                    repl_process(&input, &mut checker, &mut local_context, &mut env)
                {
                    report_errors(&errors, REPL_ID, &input);
                }

                editor.append_history(HISTORY_FILE_NAME).unwrap();
            }
            Err(ReadlineError::Eof) => break ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error}");
                break ExitCode::FAILURE;
            }
        }
    }
}
