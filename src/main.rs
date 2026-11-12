use std::fs::File;

use anyhow::Context;
use ariadne::{Report, ReportKind};
use dirs::home_dir;
use rustyline::error::ReadlineError;

use crate::{diag::Diag, elab::elab, lex::lex, normalize::normalize, parse::parse};

mod diag;
mod elab;
mod expr;
mod lex;
mod normalize;
mod parse;
mod syntax;
mod token;

const HISTORY_NAME: &str = ".lamuda_history";
const REPL_ID: &str = "REPL";

fn run(src: &str) -> Result<expr::Expr, Vec<Diag>> {
    let tokens = lex(src)?;
    let stx = parse(&tokens, src)?;
    let expr = elab(stx);
    Ok(normalize(expr))
}

fn report_diags(diags: &[Diag], id: &str, src: &str) {
    for diag in diags {
        Report::build(ReportKind::Error, (id, diag.span().into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(diag.message())
            .with_label(
                ariadne::Label::new((id, diag.span().into_range()))
                    .with_message(diag.message())
                    .with_color(ariadne::Color::Red),
            )
            .finish()
            .print((id, ariadne::Source::from(src)))
            .unwrap();
    }
}

fn main() -> anyhow::Result<()> {
    let mut editor = rustyline::DefaultEditor::new()?;
    let mut history_path = home_dir().context("Failed to get home directory")?;
    history_path.push(HISTORY_NAME);
    if editor.load_history(&history_path).is_err() {
        File::create(&history_path).with_context(|| {
            format!(
                "Failed to create history file: {}",
                history_path.to_string_lossy()
            )
        })?;
    }

    loop {
        match editor.readline("❯ ") {
            Ok(input) if input.is_empty() => {}
            Ok(input) => {
                match run(&input) {
                    Ok(expr) => println!("{expr}"),
                    Err(diags) => report_diags(&diags, REPL_ID, &input),
                }
                editor.add_history_entry(input)?;
            }
            Err(ReadlineError::Eof) => break,
            Err(error) => Err(error)?,
        }
    }

    editor.save_history(&history_path)?;
    Ok(())
}
