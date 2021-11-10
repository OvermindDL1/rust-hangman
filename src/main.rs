mod words;

use anyhow::Context;
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::*;
use crossterm::{event, execute, queue};
use rand::prelude::*;
use std::io::{stdout, Write};

const HANGMAN: &[&str] = &[
	"
    +---+
    |   |
        |
        |
        |
        |
    =========",
	r"
    +---+
    |   |
    O   |
        |
        |
        |
    =========",
	r"
    +---+
    |   |
    O   |
    |   |
        |
        |
    =========",
	r"
    +---+
    |   |
    O   |
   /|   |
        |
        |
    =========",
	r"
    +---+
    |   |
    O   |
   /|\  |
        |
        |
    =========",
	r"
    +---+
    |   |
    O   |
   /|\  |
   /    |
        |
    =========",
	r"
    +---+
    |   |
    O   |
   /|\  |
   / \  |
        |
    =========",
];

// Hangman game!
fn main() -> anyhow::Result<()> {
	crossterm::terminal::enable_raw_mode()?;
	execute!(stdout(), EnterAlternateScreen,)?;

	let result = run_game();
	execute!(stdout(), LeaveAlternateScreen).unwrap();
	crossterm::terminal::disable_raw_mode()?;
	match result {
		Ok(msg) => {
			println!("{}\n", msg);
			Ok(())
		}
		Err(err) => Err(err),
	}
}

fn run_game() -> anyhow::Result<String> {
	// Default word list, maybe add ability to load external file sometime?
	let words = words::DEFAULT_WORD_LIST;
	let mut stdout = stdout();

	loop {
		// Lowercase is hidden, uppercase is not
		let mut word = words
			.choose(&mut thread_rng())
			.context("wordlist is empty")?
			.to_string()
			.to_ascii_lowercase()
			.as_bytes()
			.to_vec();
		let mut guessed_letters = "abcdefghijklmnopqrstuvwxyz".as_bytes().to_vec();
		let mut remaining = 0;
		let mut msg = String::new();
		let mut won = false;

		loop {
			queue!(stdout, Clear(ClearType::All), MoveTo(0, 0), Print(&msg))?;
			msg.clear();
			if remaining == HANGMAN.len() - 1 {
				queue!(stdout, SetForegroundColor(Color::Red))?;
			} else {
				queue!(stdout, ResetColor)?;
			}
			for (l, line) in HANGMAN[remaining].split('\n').enumerate() {
				queue!(stdout, MoveTo(0, 1 + l as u16), Print(line))?;
			}
			queue!(
				stdout,
				ResetColor,
				MoveTo(20, HANGMAN.len() as u16 / 2 + 1),
				Print("Word: "),
				Print(
					word.iter()
						.copied()
						.map(char::from)
						.map(underscore_lowercase)
						.collect::<String>()
				),
				MoveTo(0, 3 + HANGMAN.len() as u16),
				Print("Guessed Characters: "),
				Print(
					guessed_letters
						.iter()
						.copied()
						.map(char::from)
						.map(underscore_lowercase)
						.collect::<String>()
				),
				MoveTo(0, 4 + HANGMAN.len() as u16),
				if won || remaining == HANGMAN.len() - 1 {
					Print("Press Esc to exit or Enter to start a new game")
				} else {
					Print("Press character to guess or Esc to give up")
				},
			)?;
			stdout.flush()?;

			match event::read()? {
				Event::Key(KeyEvent {
					code: KeyCode::Esc, ..
				}) => {
					return Ok(format!(
						"Quit, word was: {}",
						word.iter()
							.copied()
							.map(|c| char::from(c).to_ascii_uppercase())
							.collect::<String>()
					));
				}
				Event::Key(KeyEvent {
					code: KeyCode::Enter,
					..
				}) if won || remaining == HANGMAN.len() - 1 => {
					break;
				}
				Event::Key(KeyEvent {
					code: KeyCode::Char(c),
					..
				}) if c >= 'a' && c <= 'z' && remaining < HANGMAN.len() - 1 && !won => {
					let index: usize = c as usize - 'a' as usize;
					if c != guessed_letters[index] as char {
						msg = format!("\rAlready guessed: {:?}", c);
						continue;
					}
					guessed_letters[index] = c.to_ascii_uppercase() as u8;
					let mut found = false;
					word.iter_mut().filter(|cc| **cc == c as u8).for_each(|c| {
						*c = c.to_ascii_uppercase();
						found = true;
					});
					if found {
						if word
							.iter()
							.copied()
							.map(char::from)
							.all(|c| c.is_ascii_uppercase())
						{
							won = true;
							msg = format!(
								"You win!  Word was: {}",
								word.iter()
									.copied()
									.map(|c| char::from(c).to_ascii_uppercase())
									.collect::<String>()
							);
						} else {
							msg = format!("Found: {:?}", c);
						}
					} else {
						remaining += 1;
						if remaining == HANGMAN.len() - 1 {
							msg = format!(
								"\rFAILED, word was: {}",
								word.iter()
									.copied()
									.map(|c| char::from(c).to_ascii_uppercase())
									.collect::<String>()
							);
						} else {
							msg = format!("Not found: {:?}", c);
						}
					}
				}
				Event::Key(KeyEvent { code, .. }) => {
					msg = format!("\rUnhandled key: {:?}", code);
					continue;
				}
				Event::Mouse(_) => {}
				Event::Resize(_, _) => {}
			}
		}
	}
}

fn underscore_lowercase(c: char) -> char {
	if c.is_uppercase() {
		c
	} else {
		'_'
	}
}
