//! This module defines the types used in the markdown parser, including tokens, inline elements,
//! block elements, and a cursor for navigating through tokens.

use log::warn;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::html_generator::indent_html;
use crate::{CONFIG, io::copy_image_to_output_dir, utils::build_rel_prefix};

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
    BlockQuoteMarker,
    RawHtmlTag(String),
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
    BlockQuote {
        content: Vec<MdBlockElement>,
    },
    RawHtml {
        content: String,
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

                format!("\n<h{level}>{inner_html}</h{level}>\n")
            }
            MdBlockElement::Paragraph { content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<p>{inner_html}</p>")
            }
            MdBlockElement::CodeBlock { language, lines } => {
                let language_class = match language {
                    Some(language) => format!("language-{language}"),
                    None => "language-none".to_string(),
                };

                if CONFIG.get().unwrap().html.use_prism {
                    let code = lines.join("\n");

                    format!(
                        "<pre class=\"{language_class} line-numbers\" style=\"white-space: pre-wrap;\" data-prismjs-copy=\"📋\">\n<code class=\"{language_class} line-numbers\">{code}</code></pre>"
                    )
                } else {
                    let code = lines
                        .iter()
                        .map(|line| format!("<code class=\"non_prism\">{line}</code>"))
                        .collect::<String>();

                    format!("<pre class=\"non_prism\">{code}</pre>")
                }
            }
            MdBlockElement::ThematicBreak => "<hr>".to_string(),
            MdBlockElement::UnorderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();

                let inner_items = indent_html(&inner_items, 1);
                format!("<ul>\n{inner_items}\n</ul>")
            }
            MdBlockElement::OrderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();

                let inner_items = indent_html(&inner_items, 1);
                format!("<ol>\n{inner_items}\n</ol>")
            }
            MdBlockElement::Table { headers, body } => {
                let header_html = headers
                    .iter()
                    .map(|cell| cell.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<Vec<_>>()
                    .join("\n");

                let header_html = indent_html(&header_html, 3);

                let body_html = body
                    .iter()
                    .map(|row| {
                        let cell_html = row
                            .iter()
                            .map(|cell| cell.to_html(output_dir, input_dir, html_rel_path))
                            .collect::<Vec<_>>()
                            .join("\n");

                        let cell_html = indent_html(&cell_html, 1);

                        format!("<tr>\n{cell_html}\n</tr>")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let body_html = indent_html(&body_html, 2);

                format!(
                    "<table>\n\t<thead>\n\t\t<tr>\n{header_html}\n\t\t</tr>\n\t</thead>\n\t<tbody>\n{body_html}\n\t</tbody>\n</table>"
                )
            }
            MdBlockElement::BlockQuote { content } => {
                let inner_html = content
                    .iter()
                    .map(|el| el.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();

                format!("<blockquote>\n{inner_html}\n</blockquote>")
            }
            MdBlockElement::RawHtml { content } => {
                format!("{}\n", content)
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
                let inner_items = indent_html(&inner_items, 1);
                format!("<ul>\n{inner_items}\n</ul>")
            }
            MdBlockElement::OrderedList { items } => {
                let inner_items = items
                    .iter()
                    .map(|item| item.to_html(output_dir, input_dir, html_rel_path))
                    .collect::<String>();
                format!("<ol>\n{inner_items}\n</ol>")
            }
            _ => {
                let inner_html = indent_html(
                    &self.content.to_html(output_dir, input_dir, html_rel_path),
                    1,
                );
                format!("<li>\n{inner_html}\n</li>\n")
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
                        warn!("Unable to copy image {url}: {e}");
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
/// * `parsed_position` - The position in the `Vec<MdInlineElement>` where the content of this
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
    /// See i<https://spec.commonmark.org/0.31.2/#left-flanking-delimiter-run> for more information.
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

#[derive(Debug)]
pub struct PoolCreationError {
    pub message: String,
}

impl std::fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PoolCreationError: {}", self.message)
    }
}


impl Error for PoolCreationError {}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<Self, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError {
                message: "Thread pool size must be greater than 0".to_string(),
            });
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::build(id, Arc::clone(&receiver)).map_err(|e| {
                PoolCreationError {
                    message: format!("Failed to create worker thread {}: {}", id, e),
                }
            })?);
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn join_all(self) {
        drop(self.sender); // Close the channel to signal workers to exit

        for worker in self.workers {
            if let Err(e) = worker.thread.join() {
                warn!("Worker thread {} failed to join: {:?}", worker.id, e);
            }
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Box::new(f);

        self.sender.send(job).unwrap_or_else(|e| {
            warn!("Failed to send job to thread pool: {}", e);
        });
    }
}

#[derive(Debug)]
pub struct WorkerCreationError {
    pub message: String,
}

impl std::fmt::Display for WorkerCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorkerCreationError: {}", self.message)
    }
}

impl Error for WorkerCreationError {}

struct Worker {
    pub id: usize,
    pub thread: thread::JoinHandle<()>,
}

impl Worker {
    fn build(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Self, Box<dyn Error>> {
        let builder = thread::Builder::new();

        let thread = builder
            .spawn(move || {
                loop {
                    let job_result = {
                        let receiver = receiver.lock().unwrap();
                        receiver.recv()
                    };

                    match job_result {
                        Ok(job) => {
                            job();
                        }
                        Err(_) => {
                            break; // Exit the loop if the channel is closed
                        }
                    }
                }
            })
            .map_err(|e| {
                Box::new(WorkerCreationError {
                    message: format!("Failed to spawn thread {}: {}", id, e),
                })
            })?;

        Ok(Worker { id, thread })
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
