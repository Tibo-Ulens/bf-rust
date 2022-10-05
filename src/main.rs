#![feature(iterator_try_collect)]

#[macro_use]
extern crate thiserror;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Arg, Command};

mod error;
mod interpret;
mod transformers;

use error::Error;
use interpret::Interpreter;
use transformers::Transformer;

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

fn main_() -> Result<(), Error> {
	let config = make_config()?;

	let file = File::open(&config.file)?;
	let bytes: Vec<u8> = file.bytes().try_collect()?;

	let instructions = Transformer::transform(&bytes)?;
	let mut interpreter = Interpreter::new(&instructions);

	interpreter.interpret()?;

	Ok(())
}

fn main() {
	match main_() {
		Ok(_) => (),
		Err(e) => eprintln!("{}", e),
	}
}
