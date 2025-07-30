//! This module provides functionality to tokenize a line of markdown text into a vector of `Token`
//! enums.

use std::mem::take;

use crate::CONFIG;
use crate::types::Token;
use crate::utils::push_buffer_to_collection;
use unicode_categories::UnicodeCategories;
use unicode_segmentation::UnicodeSegmentation;

/// Tokenizes a line of markdown text into a vector of `Token` enums.
///
/// # Arguments
/// * `markdown_line` - A string slice representing a line of markdown text.
///
/// # Returns
/// A vector of `Token` enums representing the tokenized line.
///
/// # Example
/// ```
/// use lexer::tokenize;
/// use types::Token;
/// let tokens = tokenize("This is *italic* and **bold** text.");
/// assert_eq!(tokens.len(), 9);
/// assert_eq!(tokens[4], Token::EmphasisRun { delimiter: '*', length: 1 });
/// ```
pub fn tokenize(markdown_line: &str) -> Vec<Token> {
    if markdown_line.is_empty() {
        return vec![Token::Newline];
    }

    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer: String = String::new();

    let chars = Vec::from_iter(markdown_line.graphemes(true));
    let str_len = chars.len();

    // Loop through each character, and perform foward lookups for *
    let mut i = 0;
    while i < str_len {
        match chars[i] {
            "*" | "_" => {
                // if the current buffer isn't empty, append a Text token to the Vec<Token>
                push_buffer_to_collection(&mut tokens, &mut buffer);

                let delimiter = chars[i];
                let mut run_length = 1;
                while i + run_length < str_len && chars[i + run_length] == delimiter {
                    run_length += 1;
                }

                tokens.push(Token::EmphasisRun {
                    delimiter: delimiter.chars().next().unwrap(),
                    length: run_length,
                });

                i += run_length - 1;
            }
            "`" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                if i + 2 < str_len && chars[i + 1] == "`" && chars[i + 2] == "`" {
                    tokens.push(Token::CodeFence);
                    i += 2;
                } else {
                    tokens.push(Token::CodeTick);
                }
            }
            "\\" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                if i + 1 < str_len {
                    tokens.push(Token::Escape(String::from(chars[i + 1])));
                    i += 1;
                } else {
                    buffer.push_str(chars[i]);
                }
            }
            "-" => {
                // Check for thematic break
                push_buffer_to_collection(&mut tokens, &mut buffer);

                if i + 2 < str_len && chars[i + 1] == "-" && chars[i + 2] == "-" {
                    tokens.push(Token::ThematicBreak);
                    i += 2;
                } else {
                    tokens.push(Token::Punctuation(String::from(chars[i])));
                }
            }
            "[" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::OpenBracket);
            }
            "]" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::CloseBracket);
            }
            "(" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::OpenParenthesis);
            }
            ")" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::CloseParenthesis);
            }
            "|" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::TableCellSeparator);
            }
            ">" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                if i == 0 {
                    tokens.push(Token::BlockQuoteMarker);
                } else {
                    buffer.push_str(chars[i]);
                }
            }
            "<" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                while i + 1 < str_len && chars[i + 1] != ">" {
                    buffer.push_str(chars[i]);
                    i += 1;
                }

                if i + 1 < str_len && chars[i + 1] == ">" {
                    buffer.push_str(chars[i]);
                    buffer.push_str(chars[i + 1]);
                    tokens.push(Token::RawHtmlTag(take(&mut buffer)));
                    i += 1;
                } else {
                    buffer.push_str(chars[i]);
                }
            }
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                // Check for valid ordered list marker
                if i + 2 < str_len && chars[i + 1] == "." && chars[i + 2] == " " {
                    // Check if the line STARTS with a number followed by a dot and space
                    if i == 0 || tokens.last() == Some(&Token::Tab) {
                        push_buffer_to_collection(&mut tokens, &mut buffer);
                        tokens.push(Token::OrderedListMarker(chars[i].to_owned() + chars[i + 1]));

                        i += 2;
                        continue;
                    } else {
                        // If the line does not start with a number followed by a dot and space,
                        // treat it as a regular text token
                        buffer.push_str(chars[i]);
                    }
                } else {
                    buffer.push_str(chars[i]);
                }
            }
            "\t" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::Tab);
            }
            " " => {
                // Will be configurable later, but for now we'll stick to 4 spaces = 1 tab
                let tab_size = CONFIG.get().unwrap().lexer.tab_size;
                if i + tab_size <= str_len && chars[i + 1..i + tab_size].iter().all(|&c| c == " ") {
                    push_buffer_to_collection(&mut tokens, &mut buffer);
                    tokens.push(Token::Tab);
                    i += tab_size; // i won't increment after continue, so we do it here
                    continue;
                }

                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::Whitespace);
            }
            "" | "\n" => {
                push_buffer_to_collection(&mut tokens, &mut buffer);

                tokens.push(Token::Newline);
            }
            // Note that graphemes() returns strings because graphemes can consist of things like a
            // char + a modifier
            _ if is_punctuation(chars[i]) => {
                push_buffer_to_collection(&mut tokens, &mut buffer);
                tokens.push(Token::Punctuation(String::from(chars[i])));
            }
            _ => buffer.push_str(chars[i]),
        }

        i += 1;
    }

    // If the current buffer isn't empty when the loop is over, append it to the tokens vector
    push_buffer_to_collection(&mut tokens, &mut buffer);

    tokens
}

/// Helper function to determine if a string is a single punctuation character.
///
/// # Arguments
/// * `input_str` - A string slice to check.
///
/// # Returns
/// Returns `true` if the string is a single punctuation character or symbol currency, otherwise
/// `false`.
///
/// # Example
/// ```
/// use lexer::is_punctuation;
/// assert!(is_punctuation("!"));
/// assert!(!is_punctuation("Hello"));
/// assert!(is_punctuation("$"));
/// ```
fn is_punctuation(input_str: &str) -> bool {
    if let Some(ch) = input_str.chars().next() {
        ch.is_punctuation() || ch.is_symbol_currency()
    } else {
        false
    }
}

#[cfg(test)]
mod test;
