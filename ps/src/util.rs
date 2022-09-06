//! Module implementing utility functions.

/// Parses a list of strings from the given string.
/// On syntax error, the function returns an error.
pub fn parse_str_list(s: &str) -> Vec<String> {
	s.split(| c: char | c == ' ' || c == '\t' || c == ',' )
		.map(| s | s.to_string())
		.collect()
}

/// Parses a list of numbers from the given string.
/// On syntax error, the function returns an error.
pub fn parse_nbr_list(s: &str) -> Result<Vec<u32>, ()> {
	let iter = s.split(| c: char | c == ' ' || c == '\t' || c == ',' );
	let mut list = Vec::new();

	for e in iter {
		list.push(e.parse::<u32>().map_err(|_| ())?);
	}

	Ok(list)
}
