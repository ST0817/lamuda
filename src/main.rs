use std::fs::File;

use anyhow::Context;
use dirs::home_dir;
use rustyline::error::ReadlineError;

const HISTORY_NAME: &str = ".lamuda_history";

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
                println!("input: {input}");
                editor.add_history_entry(input)?;
            }
            Err(ReadlineError::Eof) => break,
            Err(error) => Err(error)?,
        }
    }

    editor.save_history(&history_path)?;
    Ok(())
}
