#[derive(Debug, Error)]
pub enum Error {
	#[error("Unknown file extension, only .bf and .bfc are supported")]
	UnknownFileExtension,
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("Failed to read input")]
	CouldNotReadInput,
	#[error("Missing opening bracket for bracket at position {0}")]
	MissingOpeningBracket(usize),
	#[error("Missing closing bracket for bracket at position {0}")]
	MissingClosingBracket(usize),
}
