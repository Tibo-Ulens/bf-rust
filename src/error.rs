#[derive(Debug, Error)]
pub enum Error {
	#[error("No file was provided")]
	NoFile,
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("Failed to read input")]
	CouldNotReadInput,
	#[error("Missing opening bracket for bracket at position {0}")]
	MissingOpeningBracket(usize),
	#[error("Missing closing bracket for bracket at position {0}")]
	MissingClosingBracket(usize),
}
