# Rust Lojban Parser

A Rust library for parsing Lojban text based on a Pest grammar derived from the official `gerna.peg`.

## Description

This library provides functionality to parse Lojban strings according to the grammar defined in `src/lojban.pest`. It utilizes the `pest` parser generator for efficient parsing. A key feature is the runtime validation of terminal symbols (like cmavo and gismu) against external wordlists.

## Features

*   **Pest-based Parsing:** Uses the `pest` crate for parsing.
*   **Grammar:** Based on the grammar defined in `src/lojban.pest`, which is derived from the official Lojban grammar (`gerna.peg`).
*   **Runtime Wordlist Validation:** Validates identified `cmavo` and `gismu` against `cmavo.txt` and `gismu.txt` files at runtime. This ensures that only valid Lojban words are accepted for these categories.
*   **Custom Error Handling:** Provides a specific `LojbanParseError` enum to distinguish between grammar parsing errors (`PestError`) and invalid word errors found during validation.

## Wordlists

This library requires the presence of wordlist files at runtime to validate terminal symbols. It expects the following files to be located in a `wordlists/` directory *relative to the working directory where the program using this library is executed*:

*   `wordlists/cmavo.txt`
*   `wordlists/gismu.txt`
*   `wordlists/rafsi.txt` (Note: Currently loaded but not actively used for validation in the default `parse_lojban` function).

Ensure these files are accessible from your program's execution environment.

## Usage

1.  **Add Dependency:** Add this library to your `Cargo.toml`:

    ```toml
    [dependencies]
    lojban_parser = { path = "../path/to/rust" } # Adjust the path as needed
    # Or, if published:
    # lojban_parser = "0.1.0"
    ```

2.  **Parse Lojban Text:** Use the `parse_lojban` function:

    ```rust
    use lojban_parser::{parse_lojban, LojbanParseError, Rule}; // Assuming Rule is needed from pest::iterators

    fn main() {
        // Ensure wordlists/cmavo.txt and wordlists/gismu.txt exist relative to execution path
        let lojban_text = "mi klama le zarci";

        match parse_lojban(lojban_text) {
            Ok(pairs) => {
                println!("Successfully parsed!");
                // Process the parsed pairs (pest::iterators::Pairs<Rule>)
                // Example: Iterate through top-level pairs
                for pair in pairs {
                    println!("Rule: {:?}, Text: \"{}\"", pair.as_rule(), pair.as_str());
                    // You can recursively explore inner pairs: pair.into_inner()
                }
            }
            Err(LojbanParseError::PestError(e)) => {
                eprintln!("Grammar parsing error:\n{}", e);
            }
            Err(LojbanParseError::InvalidWord { word, expected_type, span }) => {
                eprintln!("Invalid word found: '{}' at {:?}, expected a valid {}", word, span, expected_type);
            }
            Err(LojbanParseError::IoError(e)) => {
                 eprintln!("Error loading wordlist: {}", e);
            }
        }

        let invalid_text = "mi qklama le zarci"; // "qklama" is not a valid gismu
         match parse_lojban(invalid_text) {
            Ok(_) => {
                 println!("This should not succeed for invalid text.");
            }
            Err(e) => {
                 eprintln!("Correctly failed for invalid text: {}", e);
            }
         }
    }
    ```

## Building

To build the library itself (e.g., for development):

```bash
cd rust # Navigate to the library directory
cargo build
```

## Testing

To run the tests included with the library:

```bash
cd rust
cargo test

## Original version

[The original version](https://github.com/alanpost/jbogenturfahi) of this library by Alan Post is written in Scheme and is placed under `/scheme`.