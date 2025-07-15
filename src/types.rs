//! This module defines the types used in the markdown parser, including tokens, inline elements,
//! block elements, and a cursor for navigating through tokens.

use crate::{io::copy_image_to_output_dir, utils::build_rel_prefix};

pub trait ToHtml {
    /// Converts the implementing type to an String representing its HTML equivalent.
    fn to_html(&self, output_dir: &str, input_dir: &str, html_rel_path: &str) -> String;
}

/// Represents the different types of tokens that can be found in a markdown line.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Text(String),
    EmphasisRun { delimiter: char, length: usize },
    Punctuation(String),
    OpenBracket,
    CloseBracket,
    OpenParenthesis,
    CloseParenthesis,
    TableCellSeparator,
    OrderedListMarker(String),
    Whitespace,
    CodeTick,
    CodeFence,
    ThematicBreak,
    Escape(String),
    Tab,
    Newline,
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        Token::Text(s.to_string())
    }
}

/// Represents block-level markdown elements.
#[derive(Debug, PartialEq)]
pub enum MdBlockElement {
    Header {
        level: u8,
        content: Vec<MdInlineElement>,
    },
    Paragraph {
        content: Vec<MdInlineElement>,
    },
    CodeBlock {
        language: Option<String>,
        lines: Vec<String>,
    },
    ThematicBreak,
    UnorderedList {
        items: Vec<MdListItem>,
    },
    OrderedList {
        items: Vec<MdListItem>,
    },
    Table {
        headers: Vec<MdTableCell>,
        body: Vec<Vec<MdTableCell>>,
    },
}

impl ToHtml for MdBlockElement {
    fn to_html(&self, output_dir: &str, input_dir: &str, html_rel_path: &str) -> String {
        match self {
            MdBlockElement::Header { level, content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<h{level}>{inner_html}</h{level}>")
            }
            MdBlockElement::Paragraph { content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<p>{inner_html}</p>")
            }
            MdBlockElement::CodeBlock { language, lines } => {
                let code = lines.join("\n");
                match language {
                    Some(language) => {
                        format!("<pre><code class=\"language-{language}\">{code}</code></pre>")
                    }
                    None => {
                        format!("<pre><code>{code}</code></pre>")
                    }
                }
            }
            MdBlockElement::ThematicBreak => "<hr>".to_string(),
            MdBlockElement::UnorderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<ul>{inner_items}</ul>")
            }
            MdBlockElement::OrderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<ol>{inner_items}</ol>")
            }
            MdBlockElement::Table { headers, body } => {
                let header_html = headers
                    .iter()
                    .map(|cell| cell.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();

                let body_html = body
                    .iter()
                    .map(|row| {
                        let cell_html = row
                            .iter()
                            .map(|cell| cell.to_html(output_dir, input_dir, html_rel_path))
                            .collect::<String>();

                        format!("<tr>{cell_html}</tr>")
                    })
                    .collect::<Vec<_>>()
                    .join("");

                format!(
                    "<table><thead><tr>{header_html}</tr></thead><tbody>{body_html}</tbody></table>"
                )
            }
        }
    }
}

/// Represents a list item in markdown, which can contain block elements.
///
/// # Fields
/// * `content` - The content of the list item, which can be any block-level markdown element.
#[derive(Debug, PartialEq)]
pub struct MdListItem {
    pub content: MdBlockElement,
}

impl ToHtml for MdListItem {
    fn to_html(&self, output_dir: &str, input_dir: &str, html_rel_path: &str) -> String {
        match &self.content {
            MdBlockElement::UnorderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<ul>{inner_items}</ul>")
            }
            MdBlockElement::OrderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<ol>{inner_items}</ol>")
            }
            _ => {
                let inner_html = self.content.to_html(output_dir, input_dir, html_rel_path);
                format!("<li>{inner_html}</li>")
            }
        }
    }
}

/// Represents a cell in a markdown table.
#[derive(Debug, PartialEq, Clone)]
pub struct MdTableCell {
    pub content: Vec<MdInlineElement>,
    pub alignment: TableAlignment,
    pub is_header: bool,
}

impl ToHtml for MdTableCell {
    fn to_html(&self, output_dir: &str, input_dir: &str, html_rel_path: &str) -> String {
        let inner_html = self
            .content
            .iter()
            .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
            .collect::<String>();

        let text_alignment = match self.alignment {
            TableAlignment::Left | TableAlignment::None => "left",
            TableAlignment::Center => "center",
            TableAlignment::Right => "right",
        };

        match self.is_header {
            true => format!("<th style=\"text-align:{text_alignment};\">{inner_html}</th>"),
            false => format!("<td style=\"text-align:{text_alignment};\">{inner_html}</td>"),
        }
    }
}

/// Represents the alignment of table cells in markdown tables.
#[derive(Debug, PartialEq, Clone)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
    None,
}

/// Represents inline markdown elements (text, bold/italic, link, etc.)
#[derive(Debug, PartialEq, Clone)]
pub enum MdInlineElement {
    Text {
        content: String,
    },
    Bold {
        content: Vec<MdInlineElement>,
    },
    Italic {
        content: Vec<MdInlineElement>,
    },
    Link {
        text: Vec<MdInlineElement>,
        title: Option<String>,
        url: String,
    },
    Image {
        alt_text: String,
        title: Option<String>,
        url: String,
    },
    Code {
        content: String,
    },
    Placeholder,
}

impl From<String> for MdInlineElement {
    fn from(s: String) -> Self {
        MdInlineElement::Text {
            content: s.to_string(),
        }
    }
}

impl ToHtml for MdInlineElement {
    fn to_html(&self, output_dir: &str, input_dir: &str, html_rel_path: &str) -> String {
        match self {
            MdInlineElement::Text { content } => content.clone(),
            MdInlineElement::Bold { content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<b>{}</b>", inner_html)
            }
            MdInlineElement::Italic { content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<i>{}</i>", inner_html)
            }
            MdInlineElement::Link { text, title, url } => {
                let label_html = text
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();

                if url.contains("youtube.com") && url.contains("v=") {
                    let video_id = url
                        .split("v=")
                        .nth(1)
                        .and_then(|s| s.split('&').next())
                        .unwrap_or("");

                    return format!(
                        r#"<div class="video-container">
                        <iframe width="560" height="315" src="https://www.youtube.com/embed/{}" 
                        title="YouTube video player" frameborder="0" allowfullscreen></iframe>
                        </div>"#,
                        video_id
                    );
                }

                // Links to external URLs will open in a new tab
                if url.starts_with("http") {
                    match title {
                        Some(text) => {
                            format!(
                                "<a href=\"{url}\" title=\"{text}\" target=\"_blank\">{label_html}⮺</a>"
                            )
                        }
                        None => format!("<a href=\"{url}\" target=\"_blank\">{label_html}⮺</a>"),
                    }
                } else {
                    match title {
                        Some(text) => {
                            format!("<a href=\"{url}\" title=\"{text}\">{label_html}</a>")
                        }
                        None => format!("<a href=\"{url}\">{label_html}</a>"),
                    }
                }
            }
            MdInlineElement::Image {
                alt_text,
                title,
                url,
            } => {
                let mut media_url = url.clone();

                // If the image uses a relative path, copy it to the output directory
                if !url.starts_with("http") {
                    if let Err(e) = copy_image_to_output_dir(url, output_dir, input_dir) {
                        eprintln!("Error copying image: {e}");
                    }

                    // Update the URL to point to the copied image in the output directory
                    let url = url.rsplit('/').next().unwrap_or(url);

                    let rel_prefix = build_rel_prefix(html_rel_path);

                    media_url = format!("./{}/media/{}", rel_prefix.to_string_lossy(), url);
                }

                match title {
                    Some(text) => {
                        format!("<img src=\"{media_url}\" alt=\"{alt_text}\" title=\"{text}\"/>")
                    }
                    None => format!("<img src=\"{media_url}\" alt=\"{alt_text}\"/>"),
                }
            }
            MdInlineElement::Code { content } => format!("<code>{content}</code>"),
            MdInlineElement::Placeholder => unreachable!(),
        }
    }
}

/// Cursor for navigating through a vector of tokens
///
/// This struct provides methods to access the current token, peek ahead or behind, and advance the
/// cursor position.
///
/// # Fields
/// * `tokens` - A vector of tokens to navigate through.
/// * `current_position` - The current position of the cursor within the token vector.
#[derive(Debug)]
pub struct TokenCursor {
    pub tokens: Vec<Token>,
    pub current_position: usize,
}

impl TokenCursor {
    /// Returns the current token, if any.
    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current_position)
    }

    /// Returns the nth next token, if any.
    ///
    /// # Arguments
    /// * `n` - The number of tokens to look ahead.
    ///
    /// # Returns
    /// An `Option` containing a reference to the token if it exists, or `None` if it is out of
    /// bounds.
    pub fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current_position + n)
    }

    /// Returns the nth previous token, if any.
    ///
    /// # Arguments
    /// * `n` - The number of tokens to look behind.
    ///
    /// # Returns
    /// An `Option` containing a reference to the token if it exists, or `None` if it is out of
    pub fn _peek_behind(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current_position - n)
    }

    /// Moves the cursor forward one position.
    pub fn advance(&mut self) {
        if self.current_position < self.tokens.len() {
            self.current_position += 1;
        }
    }

    /// Sets the cursor's position to the specified position.
    ///
    /// # Arguments
    /// * `pos` - The position to set the cursor to.
    ///
    /// # Panics
    /// Panics if the position is out of bounds for the token list.
    pub fn _set_position(&mut self, pos: usize) {
        if pos < self.tokens.len() {
            self.current_position = pos;
        } else {
            panic!("Position {pos} is out of bounds for the TokenCursor");
        }
    }

    /// Returns the current position of the cursor.
    pub fn position(&self) -> usize {
        self.current_position
    }

    /// Returns whether the cursor is at the end of the token list.
    pub fn is_at_eof(&self) -> bool {
        self.current_position >= self.tokens.len()
    }
}

/// Manages Delimiter runs in a markdown document.
/// A delimiter run is a sequence of the same character (e.g., `*`, `_`, `~`) that can be used for
/// bold/italic writing.
///
/// # Fields
/// * `ch` - The character that represents the delimiter (e.g., `*`, `_`, `~`).
/// * `run_length` - The number of times the delimiter character appears in a row.
/// * `token_position` - The position of the first token in this delimiter run.
/// * `parsed_position` - The position in the Vec<MdInlineElement> where the content of this
///   delimiter run will be stored.
/// * `active` - Whether this delimiter run is currently active (i.e., it has not been closed).
/// * `can_open` - Whether this delimiter can open a new emphasis run (e.g., it is left-flanking).
/// * `can_close` - Whether this delimiter can close an existing emphasis run (e.g., it is
///   right-flanking).
#[derive(Debug, Clone)]
pub struct Delimiter {
    pub ch: char,
    pub run_length: usize,
    pub token_position: usize,
    pub parsed_position: usize,
    pub active: bool,
    pub can_open: bool,  //Must be left-flanking
    pub can_close: bool, //Must be right-flanking
}

impl Delimiter {
    /// Determines whether a delimiter is "Left", "Right", or "Both" flanking
    /// For exmample, it is left flanking if it's not followed by non-whitespace, and either:
    /// 1. Not followed by punctuation
    /// 2. Followed by punctuation and preceded by whitespace or punctuation
    ///
    /// Modifies the `can_open` and `can_close` fields in-place based on the classification.
    ///
    /// See https://spec.commonmark.org/0.31.2/#left-flanking-delimiter-run for more information.
    ///
    /// # Arguments
    /// * `tokens` - A slice of tokens to classify the delimiter against.
    pub fn classify_flanking(&mut self, tokens: &[Token]) {
        let before = if self.token_position > 0 {
            Some(&tokens[self.token_position - 1])
        } else {
            None
        };

        let after = tokens.get(self.token_position + 1);
        let followed_by_whitespace = after.is_none_or(is_whitespace);
        let followed_by_punctuation = after.is_some_and(is_punctuation);

        let preceded_by_whitespace = before.is_none_or(is_whitespace);
        let preceded_by_punctuation = before.is_some_and(is_punctuation);

        let is_left_flanking = if followed_by_whitespace {
            false
        } else if !followed_by_punctuation {
            true
        } else {
            preceded_by_whitespace || preceded_by_punctuation
        };

        let is_right_flanking = if preceded_by_whitespace {
            false
        } else if !preceded_by_punctuation {
            true
        } else {
            followed_by_whitespace || followed_by_punctuation
        };

        let delimiter_char = self.ch;

        // Apply Rule of 3 and underscore restrictions
        let is_underscore = delimiter_char == '_';

        if is_underscore {
            self.can_open = is_left_flanking && (!is_right_flanking || followed_by_punctuation);

            self.can_close = is_right_flanking && (!is_left_flanking || followed_by_punctuation);
        } else {
            self.can_open = is_left_flanking;
            self.can_close = is_right_flanking;
        }
    }
}

/// Helper function to determine if a token is whitespace or newline.
///
/// # Arguments
/// * `token` - The token to check.
fn is_whitespace(token: &Token) -> bool {
    matches!(token, Token::Newline | Token::Whitespace)
}

/// Helper function to determine if a token is punctuation.
///
/// # Arguments
/// * `token` - The token to check.
fn is_punctuation(token: &Token) -> bool {
    matches!(
        token,
        Token::Punctuation(_)
            | Token::EmphasisRun {
                delimiter: _,
                length: _
            }
            | Token::OpenBracket
            | Token::CloseBracket
            | Token::OpenParenthesis
            | Token::CloseParenthesis
    )
}
