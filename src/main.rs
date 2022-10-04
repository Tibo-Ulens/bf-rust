#![feature(iterator_try_collect)]

#[macro_use]
extern crate thiserror;

use std::path::PathBuf;

use clap::{Arg, Command};

mod error;
mod interpret;

use error::Error;
use interpret::Interpreter;

struct Config {
	file: std::path::PathBuf,
}

fn make_config() -> Result<Config, Error> {
	let matches = Command::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg_required_else_help(true)
		.arg(Arg::new("file").help("The brainfuck file to run").index(1).required(true))
		.get_matches();

	let file = matches.get_one::<String>("file").ok_or::<Error>(Error::NoFile)?;

	Ok(Config { file: PathBuf::from(file) })
}

fn main() {
	let config = match make_config() {
		Ok(c) => c,
		Err(e) => {
			eprintln!("{}", e);
			return;
		},
	};

	let mut interpreter = match Interpreter::new(&config.file) {
		Ok(i) => i,
		Err(e) => {
			eprintln!("{}", e);
			return;
		},
	};

	match interpreter.interpret() {
		Ok(_) => (),
		Err(e) => {
			eprintln!("{}", e);
			return;
		},
	}
}
