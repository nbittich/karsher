use anyhow::Context;
use rustyline::completion::FilenameCompleter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{
    Cmd, CompletionType, Config, EditMode, Editor, KeyEvent, Movement,
};
use rustyline_derive::*;
use std::path::Path;

fn get_default_history_path() -> Option<Box<Path>> {
    let mut home_dir = dirs::home_dir()?;
    home_dir.push(".adana.history.txt");
    Some(home_dir.into_boxed_path())
}

pub fn save_history(
    rl: &mut Editor<CustomHelper>,
    history_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let history_path = history_path
        .map(|p| p.as_ref().into())
        .or_else(get_default_history_path)
        .context("history path not found")?;
    rl.save_history(history_path.as_ref())
        .map_err(|e| anyhow::Error::msg(format!("{e}")))
}

pub fn read_line(
    rl: &mut Editor<CustomHelper>,
    curr_cache: &str,
) -> Result<String, rustyline::error::ReadlineError> {
    let p = format!("[{curr_cache}] >> ");
    rl.readline(&format!("\x1b[1;32m{}\x1b[0m", p))
}

pub fn build_editor(
    history_path: Option<impl AsRef<Path>>,
) -> Editor<CustomHelper> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();
    let h = CustomHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        validator: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::with_config(config)
        .expect("could not build editor with given config");
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyEvent::ctrl('d'), Cmd::Interrupt);
    rl.bind_sequence(KeyEvent::ctrl('c'), Cmd::Kill(Movement::WholeBuffer));
    rl.bind_sequence(KeyEvent::ctrl('l'), Cmd::ClearScreen);
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    if rl
        .load_history(
            history_path
                .map(|p| p.as_ref().into())
                .or_else(get_default_history_path)
                .context("history path missing")
                .unwrap()
                .as_ref(),
        )
        .is_err()
    {
        println!("No previous history.");
    }
    rl
}

#[derive(Helper, Completer, Highlighter, Validator, Hinter)]
pub struct CustomHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}
