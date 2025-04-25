extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate once_cell;

use pest::Parser;
use pest::error::Error as PestError;
use pest::iterators::Pairs;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use once_cell::sync::Lazy;
use std::fmt;

// --- Wordlist Loading ---

static CMAVO_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    load_wordlist("../scheme/wordlists/cmavo.txt").expect("Failed to load cmavo wordlist")
});

static GISMU_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    load_wordlist("../scheme/wordlists/gismu.txt").expect("Failed to load gismu wordlist")
});

// Note: rafsi.txt is loaded but not used in validation yet, as the grammar
// doesn't directly expose rafsi in a way that's easily validated here.
// Lujvo construction logic would typically handle rafsi validation.
static _RAFSI_SET: Lazy<HashSet<String>> = Lazy::new(|| {
    load_wordlist("../scheme/wordlists/rafsi.txt").expect("Failed to load rafsi wordlist")
});


/// Helper function to load a wordlist file into a HashSet.
fn load_wordlist(file_path: &str) -> io::Result<HashSet<String>> {
    println!("Loading wordlist from: {}", file_path);
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = Path::new(&manifest_dir).join(file_path);
    println!("Full path: {}", path.display());
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut set = HashSet::new();
    let mut count = 0;
    for line in reader.lines() {
        let line_content = line?;
        // Split the line by whitespace and take the first word (the cmavo)
        if let Some(word) = line_content.split_whitespace().next() {
             // Handle the leading dot if present in some cmavo lists
            let clean_word = word.strip_prefix('.').unwrap_or(word).to_string();
            if !clean_word.is_empty() {
                 set.insert(clean_word);
                 count += 1;
            }
        }
    }
    println!("Loaded {} words from {}", count, file_path);
    Ok(set)
}

// --- Custom Error Type ---

#[derive(Debug)]
pub enum LojbanParseError {
    PestError(PestError<Rule>),
    InvalidWord {
        word: String,
        expected_type: String, // e.g., "cmavo", "gismu"
        span: (usize, usize),
    },
    IoError(io::Error), // Added for file loading errors during lazy init (though expect used above)
}

impl fmt::Display for LojbanParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LojbanParseError::PestError(e) => write!(f, "Pest parsing error: {}", e),
            LojbanParseError::InvalidWord { word, expected_type, span } => {
                write!(f, "Invalid word '{}' found at {:?}, expected a valid {}", word, span, expected_type)
            }
            LojbanParseError::IoError(e) => write!(f, "IO error loading wordlist: {}", e),
        }
    }
}

impl std::error::Error for LojbanParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LojbanParseError::PestError(e) => Some(e),
            LojbanParseError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<PestError<Rule>> for LojbanParseError {
    fn from(error: PestError<Rule>) -> Self {
        LojbanParseError::PestError(error)
    }
}

impl From<io::Error> for LojbanParseError {
     fn from(error: io::Error) -> Self {
         LojbanParseError::IoError(error)
     }
}


// --- Parsing and Validation Logic ---

/// Parses a Lojban text string and validates terminal symbols against wordlists.
pub fn parse_lojban(input: &str) -> Result<Pairs<'_, Rule>, LojbanParseError> {
    // Initialize wordlists (Lazy ensures this happens only once)
    Lazy::force(&CMAVO_SET);
    Lazy::force(&GISMU_SET);
    Lazy::force(&_RAFSI_SET);

    let pairs = LojbanParser::parse(Rule::text, input)?;
    validate_words(pairs.clone())?; // Clone pairs for validation pass
    Ok(pairs)
}

/// Recursively validates words within parsed pairs against loaded wordlists.
#[derive(Parser)]
#[grammar = "src/lojban.pest"]
struct LojbanParser;

fn validate_words(pairs: Pairs<Rule>) -> Result<(), LojbanParseError> {
    for pair in pairs {
        match pair.as_rule() {
            Rule::word => {
                let word = pair.as_str();
                let span = pair.as_span();
                
                // Try cmavo first, then gismu
                if !CMAVO_SET.contains(word) && !GISMU_SET.contains(word) {
                    return Err(LojbanParseError::InvalidWord {
                        word: word.to_string(),
                        expected_type: "lojban word".to_string(), // More general since Rule::word doesn't specify type
                        span: (span.start(), span.end()),
                    });
                }
            }
            Rule::text => validate_words(pair.into_inner())?,
            _ => (),
        }
    }
    Ok(())
}


// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse_valid() {
        let input = "mi"; // Should be parsed as cmavo
        match parse_lojban(input) {
            Ok(_) => assert!(true, "Successfully parsed valid cmavo"),
            Err(e) => panic!("Failed to parse valid input: {}", e),
        }
    }

    #[test]
    fn test_phrase_parse_valid() {
        let input = "mi klama le zarci"; // Should be parsed as phrase
        match parse_lojban(input) {
            Ok(_) => assert!(true, "Successfully parsed valid phrase"),
            Err(e) => panic!("Failed to parse valid input: {}", e),
        }
    }

    #[test]
    fn test_basic_parse_invalid_cmavo() {
        let input = "qle"; // Invalid cmavo
        match parse_lojban(input) {
            Ok(_) => panic!("Should not parse invalid cmavo"),
            Err(LojbanParseError::InvalidWord { word, expected_type, .. }) => {
                assert_eq!(word, "qle");
                assert_eq!(expected_type, "lojban word"); // Match the more general error type
            }
            Err(e) => panic!("Wrong error type: {}", e),
        }
    }

    #[test]
    fn test_basic_parse_invalid_gismu() {
        let input = "qklama"; // Invalid gismu/brivla
        match parse_lojban(input) {
            Ok(_) => panic!("Should not parse invalid gismu"),
            Err(LojbanParseError::InvalidWord { word, expected_type, .. }) => {
                assert_eq!(word, "qklama");
                assert_eq!(expected_type, "lojban word"); // Match the more general error type
            }
            Err(e) => panic!("Wrong error type: {}", e),
        }
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        match parse_lojban(input) {
            Ok(_) => panic!("Should not parse empty input"),
            Err(LojbanParseError::PestError(_)) => assert!(true),
            Err(e) => panic!("Wrong error type: {}", e),
        }
    }
}