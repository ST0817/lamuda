use std::process::ExitCode;

use rustyline::{DefaultEditor, error::ReadlineError};

const HISTORY_FILE_NAME: &str = ".lamuda_history";

fn main() -> ExitCode {
    let mut editor = DefaultEditor::new().unwrap();

    if editor.load_history(HISTORY_FILE_NAME).is_err() {
        println!("no previous history")
    }

    loop {
        match editor.readline("❯ ") {
            Ok(input) if input.is_empty() => {}
            Ok(input) => {
                editor.add_history_entry(&input).unwrap();
                println!("input: {input}")
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
