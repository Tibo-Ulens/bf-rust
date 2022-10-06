//! TODO: make it better

#![feature(iterator_try_collect)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use bf_rust::error::Error;
use bf_rust::interpret::Interpreter;
use bf_rust::link::link;
use bf_rust::optimise::Optimiser;
use bf_rust::transpile::{read_bytecode, transpile};
use clap::{Arg, ArgAction, Command};

struct Config {
	input_path:         PathBuf,
	output_path:        Option<PathBuf>,
	emit_bytecode:      bool,
	combine_clears:     bool,
	group_instructions: bool,
}

fn make_config() -> Result<Config, Error> {
	let matches = Command::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg_required_else_help(true)
		.arg(
			Arg::new("emit_bytecode")
				.help("If set, emit bytecode instead of running the file")
				.short('b')
				.long("emit-bytecode")
				.action(ArgAction::SetTrue),
		)
		.arg(
			Arg::new("output_file")
				.help("The file to write the bytecode to")
				.short('p')
				.long("output")
				.action(ArgAction::Set),
		)
		.arg(
			Arg::new("optimisation")
				.help("Specify what optimisations to apply")
				.short('o')
				.long("optimise")
				.action(ArgAction::Set)
				.value_delimiter(',')
				.value_parser(["all", "combine-clears", "group-instructions"]),
		)
		.arg(Arg::new("file").help("The brainfuck file to run").index(1).required(true))
		.get_matches();

	// Unwrap is safe as file is required
	let file = matches.get_one::<String>("file").unwrap();

	let output_path = matches.get_one::<String>("output_file").map(PathBuf::from);

	let optimisations: Vec<String> = match matches.get_many::<String>("optimisation") {
		Some(vals) => vals.cloned().collect(),
		None => vec![],
	};

	Ok(Config {
		input_path: PathBuf::from(file),
		output_path,
		emit_bytecode: matches.get_flag("emit_bytecode"),

		combine_clears: optimisations.contains(&"combine-clears".to_owned())
			|| optimisations.contains(&"all".to_owned()),
		group_instructions: optimisations.contains(&"group-instructions".to_owned())
			|| optimisations.contains(&"all".to_owned()),
	})
}

fn main_() -> Result<(), Error> {
	let config = make_config()?;

	let file = File::open(&config.input_path)?;
	let bytes: Vec<u8> = file.bytes().try_collect()?;

	let extension = match config.input_path.extension() {
		Some(ext) => ext.to_str().unwrap(),
		None => return Err(Error::UnknownFileExtension),
	};

	let instructions = match extension {
		"bf" => transpile(&bytes),
		"bfc" => read_bytecode(&bytes),
		_ => return Err(Error::UnknownFileExtension),
	};

	let mut optimised_instructions = instructions;
	if config.combine_clears {
		optimised_instructions = Optimiser::combine_clears(&optimised_instructions);
	}
	if config.group_instructions {
		optimised_instructions = Optimiser::group_instructions(&optimised_instructions);
	}

	let linked_instructions = link(optimised_instructions)?;

	let mut interpreter = Interpreter::new(&linked_instructions);
	if config.emit_bytecode {
		let output_path = match config.output_path {
			Some(p) => p,
			None => {
				let mut original = config.input_path;
				original.set_extension("bfc");
				original
			},
		};

		let mut output_writer = File::create(output_path)?;

		interpreter.write_bytecode(&mut output_writer)?;
	} else {
		interpreter.run()?;
	}

	Ok(())
}

fn main() {
	match main_() {
		Ok(_) => (),
		Err(e) => eprintln!("{}", e),
	}
}
