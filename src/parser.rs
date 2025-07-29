//! This module contains the parser for converting tokenized Markdown lines into structured
//! Markdown elements.
//!
//! It provides functions to parse block-level elements like headings, lists, and code blocks,
//! as well as inline elements like links, images, and emphasis.

use log::warn;

use crate::CONFIG;
use crate::types::{
    Delimiter, MdBlockElement, MdInlineElement, MdListItem, MdTableCell, TableAlignment, Token,
    TokenCursor,
};
use crate::utils::push_buffer_to_collection;

/// Parses a vector of tokenized markdown lines into a vector of block-level Markdown elements.
///
/// # Arguments
/// * `markdown_lines` - A vector of vectors, where each inner vector contains tokens representing a line of markdown.
///
/// # Returns
/// A vector of parsed block-level Markdown elements.
pub fn parse_blocks(markdown_lines: &[Vec<Token>]) -> Vec<MdBlockElement> {
    let mut block_elements: Vec<MdBlockElement> = Vec::new();

    for line in markdown_lines {
        if let Some(element) = parse_block(line) {
            block_elements.push(element)
        }
    }

    block_elements
}

/// Parses a single line of tokens into a block-level Markdown element.
///
/// # Arguments
/// * `line` - A vector of tokens representing a single line of markdown.
///
/// # Returns
/// An `Option<MdBlockElement>`, returning `None` for empty lines
fn parse_block(line: &[Token]) -> Option<MdBlockElement> {
    let first_token = line.first();

    match first_token {
        Some(Token::Punctuation(string)) if string == "#" => Some(parse_heading(line)),
        Some(Token::Punctuation(string)) if string == "-" => {
            // Note that setext headings have already been handled in the group_lines_to_blocks
            // function by this point
            if line.len() == 1 {
                // If the line only contains a dash, then it is a thematic break
                Some(MdBlockElement::ThematicBreak)
            } else {
                Some(parse_unordered_list(line))
            }
        }
        Some(Token::OrderedListMarker(_)) => Some(parse_ordered_list(line)),
        Some(Token::CodeFence) => Some(parse_codeblock(line)),
        Some(Token::ThematicBreak) => Some(MdBlockElement::ThematicBreak),
        Some(Token::TableCellSeparator) => Some(parse_table(line)),
        Some(Token::BlockQuoteMarker) => Some(parse_blockquote(line)),
        Some(Token::RawHtmlTag(_)) => Some(parse_raw_html(line)),
        Some(Token::Tab) => Some(parse_indented_codeblock(line)),
        Some(Token::Newline) => None,
        _ => Some(MdBlockElement::Paragraph {
            content: parse_inline(line),
        }),
    }
}

/// Parses an indented code block from a vector of tokens.
///
/// Note that CommonMark defines indented code blocks as lines that start with at least 4 spaces or
/// a tab. However, this implementation only focuses on tabs, the size of which is defined in
/// `config.toml`.
///
/// # Arguments
/// * `line` - A vector of tokens representing an indented code block.
///
/// # Returns
/// An `MdBlockElement::CodeBlock` containing the parsed code content.
fn parse_indented_codeblock(line: &[Token]) -> MdBlockElement {
    let mut code_content: Vec<String> = Vec::new();
    let mut line_buffer: String = String::new();

    let lines_split_by_newline = line.split(|token| token == &Token::Newline);

    lines_split_by_newline.for_each(|token_line| {
        if token_line.is_empty() {
            return;
        }

        for token in &token_line[1..] {
            match token {
                Token::Tab => {
                    line_buffer.push_str(&" ".repeat(CONFIG.get().unwrap().lexer.tab_size));
                }
                Token::Text(string) | Token::Punctuation(string) => line_buffer.push_str(string),
                Token::Whitespace => line_buffer.push(' '),
                Token::Newline => {
                    push_buffer_to_collection(&mut code_content, &mut line_buffer);
                    line_buffer.clear();
                }
                Token::Escape(esc_char) => {
                    line_buffer.push_str(&format!("\\{esc_char}"));
                }
                Token::OrderedListMarker(string) => line_buffer.push_str(string),
                Token::EmphasisRun { delimiter, length } => {
                    line_buffer.push_str(&delimiter.to_string().repeat(*length))
                }
                Token::OpenParenthesis => line_buffer.push('('),
                Token::CloseParenthesis => line_buffer.push(')'),
                Token::OpenBracket => line_buffer.push('['),
                Token::CloseBracket => line_buffer.push(']'),
                Token::TableCellSeparator => line_buffer.push('|'),
                Token::CodeTick => line_buffer.push('`'),
                Token::CodeFence => line_buffer.push_str("```"),
                Token::BlockQuoteMarker => line_buffer.push('>'),
                Token::ThematicBreak => line_buffer.push_str("---"),
                Token::RawHtmlTag(tag_content) => {
                    // This should never be the first token, but inline html is allowed
                    let escaped_tag = tag_content.replace("<", "&lt;").replace(">", "&gt;");
                    line_buffer.push_str(&escaped_tag);
                }
            }
        }

        push_buffer_to_collection(&mut code_content, &mut line_buffer);
    });

    MdBlockElement::CodeBlock {
        language: None,
        lines: code_content,
    }
}

/// Parses raw HTML tags from a vector of tokens into an `MdBlockElement::RawHtml`.
///
/// # Arguments
/// * `line` - A vector of tokens representing a line of raw HTML.
///
/// # Returns
/// An `MdBlockElement::RawHtml` containing the parsed HTML content.
fn parse_raw_html(line: &[Token]) -> MdBlockElement {
    let mut html_content = String::new();
    for token in line {
        match token {
            Token::RawHtmlTag(tag_content) => html_content.push_str(tag_content),
            Token::Text(string) | Token::Punctuation(string) => html_content.push_str(string),
            Token::Whitespace => html_content.push(' '),
            Token::Escape(esc_char) => {
                html_content.push_str(&format!("\\{esc_char}"));
            }
            Token::Newline => html_content.push('\n'),
            Token::OrderedListMarker(string) => html_content.push_str(string),
            Token::EmphasisRun { delimiter, length } => {
                html_content.push_str(&delimiter.to_string().repeat(*length))
            }
            Token::OpenParenthesis => html_content.push('('),
            Token::CloseParenthesis => html_content.push(')'),
            Token::OpenBracket => html_content.push('['),
            Token::CloseBracket => html_content.push(']'),
            Token::TableCellSeparator => html_content.push('|'),
            Token::CodeTick => html_content.push('`'),
            Token::CodeFence => html_content.push_str("```"),
            Token::BlockQuoteMarker => html_content.push('>'),
            Token::Tab => {
                html_content.push_str(&" ".repeat(CONFIG.get().unwrap().lexer.tab_size));
            }
            Token::ThematicBreak => html_content.push_str("---"),
        }
    }

    MdBlockElement::RawHtml {
        content: html_content,
    }
}

/// Parses a blockquote from a vector of tokens into an `MdBlockElement::BlockQuote`.
///
/// # Arguments
/// * `line` - A vector of tokens representing a blockquote.
///
/// # Returns
/// An `MdBlockElement::BlockQuote` containing the parsed content, or a `MdBlockElement::Paragraph`
/// if the content is empty.
fn parse_blockquote(line: &[Token]) -> MdBlockElement {
    let lines_split_by_newline = line.split(|token| token == &Token::Newline);

    let inner_blocks: Vec<Vec<Token>> = lines_split_by_newline
        .map(|tokens| {
            let mut result = Vec::new();
            if tokens.first() == Some(&Token::BlockQuoteMarker)
                && tokens.get(1) == Some(&Token::Whitespace)
            {
                result.extend_from_slice(&tokens[2..]);
            } else if tokens.first() == Some(&Token::BlockQuoteMarker) {
                result.extend_from_slice(&tokens[1..]);
            } else {
                result.extend_from_slice(tokens);
            }
            result
        })
        .collect();

    let grouped_inner_blocks = group_lines_to_blocks(inner_blocks);

    let content = parse_blocks(&grouped_inner_blocks);

    if content.is_empty() {
        MdBlockElement::Paragraph {
            content: parse_inline(line),
        }
    } else {
        MdBlockElement::BlockQuote { content }
    }
}

/// Parses a vector of tokens representing an ordered list into an `MdBlockElement::OrderedList`.
///
/// Calls the more generic `parse_list` function, which parses nested list items
///
/// # Arguments
/// * `list` - A vector of tokens representing an ordered list.
///
/// # Returns
/// An `MdBlockElement` representing the ordered list.
fn parse_ordered_list(list: &[Token]) -> MdBlockElement {
    parse_list(
        list,
        |tokens| {
            matches!(
                tokens.first(),
                Some(Token::OrderedListMarker(_)) if tokens.get(1) == Some(&Token::Whitespace)
            )
        },
        |items| MdBlockElement::OrderedList { items },
    )
}

/// Parses a vector of tokens representing an unordered list into an `MdBlockElement::UnorderedList`.
///
/// Calls the more generic `parse_list` function, which parses nested list items
///
/// # Arguments
/// * `list` - A vector of tokens representing an unordered list.
///
/// # Returns
/// An `MdBlockElement` representing the unordered list.
fn parse_unordered_list(list: &[Token]) -> MdBlockElement {
    parse_list(
        list,
        |tokens| {
            matches!(tokens.first(), Some(Token::Punctuation(string)) if string == "-" && tokens.get(1) == Some(&Token::Whitespace)
            )
        },
        |items| MdBlockElement::UnorderedList { items },
    )
}

/// Generic list parser used to reduce code duplication between ordered and unordered lists.
///
/// Handles splitting lines, identifying list items, and parsing nested lists. The behavior is
/// determined by a predicate for identifying list items and a constructor for the resulting block.
///
/// # Arguments
/// * `list` - The tokens to parse.
/// * `is_list_item` - Predicate to identify a top-level list item.
/// * `make_block` - Constructor for the resulting `MdBlockElement`.
///
/// # Returns
/// An `MdBlockElement` representing either an ordered or unordered list, depending on the passed in constructor.
fn parse_list<F, G>(list: &[Token], is_list_item: F, make_block: G) -> MdBlockElement
where
    F: Fn(&[Token]) -> bool,
    G: Fn(Vec<MdListItem>) -> MdBlockElement,
{
    let lists_split_by_newline = list
        .split(|token| token == &Token::Newline)
        .collect::<Vec<_>>();
    let mut list_items: Vec<MdListItem> = Vec::new();

    let mut i = 0;
    while i < lists_split_by_newline.len() {
        let line = lists_split_by_newline[i];
        if is_list_item(line) {
            let content_tokens = &line[2..];
            if let Some(content) = parse_block(content_tokens) {
                list_items.push(MdListItem { content })
            }

            // Check for consecutive tab-indented lines (nested list)
            let mut nested_lines: Vec<Vec<Token>> = Vec::new();
            let mut j = i + 1;
            while j < lists_split_by_newline.len() {
                let nested_line = lists_split_by_newline[j];
                if nested_line.first() == Some(&Token::Tab) {
                    let mut nested = nested_line.to_vec();
                    while !nested.is_empty() && nested[0] == Token::Tab {
                        nested.remove(0);
                    }
                    nested_lines.push(nested);
                    j += 1;
                } else {
                    break;
                }
            }

            if !nested_lines.is_empty() {
                // Flatten nested lines into a single Vec<Token> separated by Newline
                let mut nested_tokens: Vec<Token> = Vec::new();
                for (k, l) in nested_lines.into_iter().enumerate() {
                    if k > 0 {
                        nested_tokens.push(Token::Newline);
                    }
                    nested_tokens.extend(l);
                }

                // Recursively parse nested list, try ordered first, fallback to unordered
                let nested_block = if let Some(Token::OrderedListMarker(_)) = nested_tokens.first()
                {
                    parse_ordered_list(&nested_tokens)
                } else {
                    parse_unordered_list(&nested_tokens)
                };

                list_items.push(MdListItem {
                    content: nested_block,
                });

                i = j - 1; // Skip processed nested lines
            }
        }
        i += 1;
    }

    // Use the passed in constructor to create the List element
    make_block(list_items)
}

/// Parses a vector of tokens representing a code block into an `MdBlockElement::CodeBlock`.
///
/// Extracts the language (if specified) and the code content.
///
/// # Arguments
/// * `line` - A vector of tokens representing a code block.
///
/// # Returns
/// An `MdBlockElement` representing the code block.
fn parse_codeblock(line: &[Token]) -> MdBlockElement {
    let mut code_content: Vec<String> = Vec::new();
    let mut language = None;
    let mut line_buffer: String = String::new();
    let mut lines_split_by_newline = line
        .split(|token| token == &Token::Newline)
        .collect::<Vec<_>>();

    if let Some(Token::Text(string)) = line.get(1) {
        language = Some(string.clone());
        lines_split_by_newline.remove(0);
    }

    lines_split_by_newline.iter().for_each(|line| {
        if line.is_empty() {
            return;
        }

        for token in line.iter() {
            match token {
                Token::Text(string) | Token::Punctuation(string) => line_buffer.push_str(string),
                Token::Whitespace => line_buffer.push(' '),
                Token::Newline => {
                    push_buffer_to_collection(&mut code_content, &mut line_buffer);
                    line_buffer.clear();
                }
                Token::Tab => {
                    line_buffer.push_str(&" ".repeat(CONFIG.get().unwrap().lexer.tab_size));
                }
                Token::Escape(esc_char) => {
                    line_buffer.push_str(&format!("\\{esc_char}"));
                }
                Token::OrderedListMarker(string) => line_buffer.push_str(string),
                Token::EmphasisRun { delimiter, length } => {
                    line_buffer.push_str(&delimiter.to_string().repeat(*length))
                }
                Token::OpenParenthesis => line_buffer.push('('),
                Token::CloseParenthesis => line_buffer.push(')'),
                Token::OpenBracket => line_buffer.push('['),
                Token::CloseBracket => line_buffer.push(']'),
                Token::TableCellSeparator => line_buffer.push('|'),
                Token::CodeTick => line_buffer.push('`'),
                Token::CodeFence => {}
                Token::BlockQuoteMarker => line_buffer.push('>'),
                Token::RawHtmlTag(tag_content) => {
                    let escaped_tag = tag_content.replace("<", "&lt;").replace(">", "&gt;");
                    line_buffer.push_str(&escaped_tag);
                }
                Token::ThematicBreak => line_buffer.push_str("---"),
            }
        }

        push_buffer_to_collection(&mut code_content, &mut line_buffer);
    });

    push_buffer_to_collection(&mut code_content, &mut line_buffer);

    MdBlockElement::CodeBlock {
        language,
        lines: code_content,
    }
}

/// Parses a vector of tokens representing a heading into an `MdBlockElement::Header`.
///
/// Determines the heading level and parses the heading content.
///
/// # Arguments
/// * `line` - A vector of tokens representing a heading line.
///
/// # Returns
/// An `MdBlockElement` representing the heading, or a paragraph if the heading is invalid.
fn parse_heading(line: &[Token]) -> MdBlockElement {
    let mut heading_level = 0;
    let mut i = 0;
    while let Some(token) = line.get(i) {
        match token {
            Token::Punctuation(string) => {
                if string == "#" {
                    heading_level += 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
        i += 1;
    }

    // At this point, we should be at a non-# token or the end of the line
    if i >= line.len() || line.get(i) != Some(&Token::Whitespace) {
        return MdBlockElement::Paragraph {
            content: parse_inline(line),
        };
    }

    MdBlockElement::Header {
        level: heading_level,
        content: parse_inline(&line[i + 1..]),
    }
}

/// Parses GitHub-style tables from the input vector of tokens.
pub fn parse_table(line: &[Token]) -> MdBlockElement {
    let rows = line
        .split(|token| token == &Token::Newline)
        .collect::<Vec<_>>();

    if rows.len() < 3 {
        return MdBlockElement::Paragraph {
            content: parse_inline(line),
        };
    }

    let header_row = rows
        .first()
        .expect("Table should have at least a header row");

    let alignment_row = rows.get(1).expect("Table should have an alignment row");

    let alignments: Vec<TableAlignment> = split_row(alignment_row)
        .into_iter()
        .map(|cell_content| {
            let content: String = cell_content
                .iter()
                .filter_map(|token| match token {
                    Token::Text(s) => {
                        warn!("Table alignment should not contain text as it could result in unexpected behavior: {s}");
                        Some(s.to_owned())
                    }
                    Token::Punctuation(s) => Some(s.to_owned()),
                    Token::ThematicBreak => Some("---".to_string()),
                    _ => None,
                })
                .collect();

            match (content.starts_with(':'), content.ends_with(':')) {
                (true, true) => TableAlignment::Center,
                (true, false) => TableAlignment::Left,
                (false, true) => TableAlignment::Right,
                _ => TableAlignment::None,
            }
        })
        .collect();

    let headers: Vec<MdTableCell> = split_row(header_row)
        .into_iter()
        .enumerate()
        .map(|(i, cell_content)| MdTableCell {
            content: parse_inline(cell_content),
            alignment: alignments.get(i).cloned().unwrap_or(TableAlignment::None),
            is_header: true,
        })
        .collect();

    let body: Vec<Vec<MdTableCell>> = rows
        .iter()
        .skip(2)
        .map(|row| {
            split_row(row)
                .into_iter()
                .enumerate()
                .map(|(i, cell_tokens)| MdTableCell {
                    content: parse_inline(cell_tokens),
                    alignment: alignments.get(i).cloned().unwrap_or(TableAlignment::None),
                    is_header: false,
                })
                .collect()
        })
        .collect();

    MdBlockElement::Table { headers, body }
}

/// Helper function to split a row of tokens into individual cells.
///
/// By removing the starting and ending "|" characters, it ensures that the row is
/// split into the proper number of cells.
fn split_row(row: &[Token]) -> Vec<&[Token]> {
    let mut cells: Vec<&[Token]> = row
        .split(|token| token == &Token::TableCellSeparator)
        .collect();

    if let Some(first) = cells.first() {
        if first.is_empty() {
            cells.remove(0);
        }
    }
    if let Some(last) = cells.last() {
        if last.is_empty() {
            cells.pop();
        }
    }

    cells
}

/// Parses a vector of tokens into a vector of inline Markdown elements (i.e. links, images,
/// bold/italics, etc.).
///
/// # Arguments
/// * `markdown_tokens` - A vector of tokens representing inline markdown content.
///
/// # Returns
/// A vector of parsed inline Markdown elements.
pub fn parse_inline(markdown_tokens: &[Token]) -> Vec<MdInlineElement> {
    let mut parsed_inline_elements: Vec<MdInlineElement> = Vec::new();

    let mut cursor: TokenCursor = TokenCursor {
        tokens: markdown_tokens.to_vec(),
        current_position: 0,
    };

    let mut delimiter_stack: Vec<Delimiter> = Vec::new();

    let mut buffer: String = String::new();

    let mut current_token: &Token;
    while !cursor.is_at_eof() {
        current_token = cursor.current().expect("Token should be valid markdown");

        match current_token {
            Token::EmphasisRun { delimiter, length } => {
                push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer);

                delimiter_stack.push(Delimiter {
                    run_length: *length,
                    ch: *delimiter,
                    token_position: cursor.position(),
                    parsed_position: parsed_inline_elements.len(),
                    active: true,
                    can_open: true,
                    can_close: true,
                });

                parsed_inline_elements.push(MdInlineElement::Placeholder);
            }
            Token::OpenBracket => {
                push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer);

                let link_element =
                    parse_link_type(&mut cursor, |label, title, url| MdInlineElement::Link {
                        text: label,
                        title,
                        url,
                    });
                parsed_inline_elements.push(link_element);
            }
            Token::CodeTick => {
                // Search for a matching code tick, everything else is text
                cursor.advance();
                push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer);

                let code_content = parse_code_span(&mut cursor);

                if cursor.current() != Some(&Token::CodeTick) {
                    parsed_inline_elements.push(MdInlineElement::Text {
                        content: format!("`{code_content}`"),
                    });
                } else {
                    parsed_inline_elements.push(MdInlineElement::Code {
                        content: code_content,
                    });
                }
            }
            Token::Punctuation(string) if string == "!" => {
                if cursor.peek_ahead(1) != Some(&Token::OpenBracket) {
                    // If the next token is not an open bracket, treat it as text
                    buffer.push('!');
                    cursor.advance();
                    continue;
                }

                push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer);
                cursor.advance(); // Advance to the open bracket

                let image =
                    parse_link_type(&mut cursor, |label, title, url| MdInlineElement::Image {
                        alt_text: flatten_inline(&label),
                        title,
                        url,
                    });

                parsed_inline_elements.push(image);
            }
            Token::Escape(esc_char) => buffer.push_str(&format!("\\{esc_char}")),
            Token::Text(string) | Token::Punctuation(string) => buffer.push_str(string),
            Token::OrderedListMarker(string) => buffer.push_str(string),
            Token::Whitespace => buffer.push(' '),
            Token::CloseBracket => buffer.push(']'),
            Token::OpenParenthesis => buffer.push('('),
            Token::CloseParenthesis => buffer.push(')'),
            Token::ThematicBreak => buffer.push_str("---"),
            Token::TableCellSeparator => buffer.push('|'),
            Token::BlockQuoteMarker => buffer.push('>'),
            Token::RawHtmlTag(tag_content) => buffer.push_str(tag_content),
            _ => push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer),
        }

        cursor.advance();
    }

    push_buffer_to_collection(&mut parsed_inline_elements, &mut buffer);

    delimiter_stack
        .iter_mut()
        .for_each(|el| el.classify_flanking(&cursor.tokens));

    resolve_emphasis(&mut parsed_inline_elements, &mut delimiter_stack);

    parsed_inline_elements
}

/// Parses a code span starting from the current position of the cursor.
///
/// # Arguments
/// * `cursor` - A mutable reference to a `TokenCursor` that tracks the current position in the
///
/// # Returns
/// A string containing the content of the code span, excluding the opening and closing code ticks.
fn parse_code_span(cursor: &mut TokenCursor) -> String {
    let mut code_content: String = String::new();
    while let Some(next_token) = cursor.current() {
        match next_token {
            Token::CodeTick => break,
            Token::Text(string) | Token::Punctuation(string) => code_content.push_str(string),
            Token::OrderedListMarker(string) => code_content.push_str(string),
            Token::Escape(ch) => code_content.push_str(&format!("\\{ch}")),
            Token::OpenParenthesis => code_content.push('('),
            Token::CloseParenthesis => code_content.push(')'),
            Token::OpenBracket => code_content.push('['),
            Token::CloseBracket => code_content.push(']'),
            Token::TableCellSeparator => code_content.push('|'),
            Token::EmphasisRun { delimiter, length } => {
                code_content.push_str(&delimiter.to_string().repeat(*length))
            }
            Token::Whitespace => code_content.push(' '),
            Token::Tab => code_content.push_str(&" ".repeat(CONFIG.get().unwrap().lexer.tab_size)),
            Token::Newline => code_content.push('\n'),
            Token::ThematicBreak => code_content.push_str("---"),
            Token::BlockQuoteMarker => code_content.push('>'),
            Token::RawHtmlTag(tag_content) => code_content.push_str(tag_content),
            Token::CodeFence => {}
        }

        cursor.advance();
    }

    code_content
}

/// Helper function used in `parse_link_type` to circumvent Rust's limitation on closure recursion
fn make_image(label: Vec<MdInlineElement>, title: Option<String>, uri: String) -> MdInlineElement {
    MdInlineElement::Image {
        alt_text: flatten_inline(&label),
        title,
        url: uri,
    }
}

/// Helper function used in `parse_link_type` to circumvent Rust's limitation on closure recursion
fn make_link(label: Vec<MdInlineElement>, title: Option<String>, uri: String) -> MdInlineElement {
    MdInlineElement::Link {
        text: label,
        title,
        url: uri,
    }
}

/// Parses a link type (either a link or an image) from the current position of the cursor.
///
/// # Arguments
/// * `cursor` - A mutable reference to a `TokenCursor` that tracks the current position in the
///   token stream.
/// * `make_element` - A closure that takes the parsed label elements, optional title, and URI,
///   and returns an `MdInlineElement` representing the link or image.
///
/// # Returns
/// An `MdInlineElement` representing the parsed link or image.
fn parse_link_type<F>(cursor: &mut TokenCursor, make_element: F) -> MdInlineElement
where
    F: Fn(Vec<MdInlineElement>, Option<String>, String) -> MdInlineElement,
{
    let mut label_elements: Vec<MdInlineElement> = Vec::new();
    let mut label_buffer = String::new();
    let mut delimiter_stack: Vec<Delimiter> = Vec::new();
    cursor.advance(); // Move past the open bracket
    while let Some(token) = cursor.current() {
        match token {
            Token::CloseBracket => {
                push_buffer_to_collection(&mut label_elements, &mut label_buffer);
                break;
            }
            Token::OpenBracket => {
                push_buffer_to_collection(&mut label_elements, &mut label_buffer);

                let inner_link = parse_link_type(cursor, make_link);
                label_elements.push(inner_link);
            }
            Token::EmphasisRun { delimiter, length } => {
                push_buffer_to_collection(&mut label_elements, &mut label_buffer);
                delimiter_stack.push(Delimiter {
                    run_length: *length,
                    ch: *delimiter,
                    token_position: cursor.position(),
                    parsed_position: label_elements.len(),
                    active: true,
                    can_open: true,
                    can_close: true,
                });
                label_elements.push(MdInlineElement::Placeholder);
            }
            Token::Punctuation(s) if s == "!" => {
                if cursor.peek_ahead(1) != Some(&Token::OpenBracket) {
                    label_buffer.push('!');
                    cursor.advance();
                    continue;
                }

                push_buffer_to_collection(&mut label_elements, &mut label_buffer);
                cursor.advance(); // Advance to the open bracket
                let inner_image = parse_link_type(cursor, make_image);

                label_elements.push(inner_image);
            }
            Token::Text(s) | Token::Punctuation(s) => label_buffer.push_str(s),
            Token::OrderedListMarker(s) => label_buffer.push_str(s),
            Token::Escape(ch) => label_buffer.push_str(&format!("\\{ch}")),
            Token::Whitespace => label_buffer.push(' '),
            Token::ThematicBreak => label_buffer.push_str("---"),
            Token::OpenParenthesis => label_buffer.push('('),
            Token::CloseParenthesis => label_buffer.push(')'),
            Token::TableCellSeparator => label_buffer.push('|'),
            Token::BlockQuoteMarker => label_buffer.push('>'),
            _ => {}
        }
        cursor.advance();
    }

    push_buffer_to_collection(&mut label_elements, &mut label_buffer);
    resolve_emphasis(&mut label_elements, &mut delimiter_stack);

    // If we didn't find a closing bracket, treat it as text
    if cursor.current() != Some(&Token::CloseBracket) {
        return MdInlineElement::Text {
            content: format!("[{}", flatten_inline(&label_elements)),
        };
    }

    // At this point we should have parentheses for the uri, otherwise treat it as a
    // text element
    if cursor.peek_ahead(1) != Some(&Token::OpenParenthesis) {
        cursor.advance();
        return MdInlineElement::Text {
            content: format!("[{}]", flatten_inline(&label_elements)),
        };
    }

    cursor.advance(); // Move to '('

    let mut uri = String::new();
    let mut title = String::new();
    let mut is_building_title = false;
    let mut is_valid_title = true;
    let mut has_opening_quote = false;

    while let Some(token) = cursor.current() {
        if !is_building_title {
            match token {
                Token::CloseParenthesis => break,
                Token::Text(s) | Token::Punctuation(s) => uri.push_str(s),
                Token::OrderedListMarker(s) => uri.push_str(s),
                Token::Escape(ch) => uri.push_str(&format!("\\{ch}")),
                Token::Whitespace => is_building_title = true,
                Token::ThematicBreak => uri.push_str("---"),
                Token::TableCellSeparator => uri.push('|'),
                Token::BlockQuoteMarker => uri.push('>'),
                Token::RawHtmlTag(tag_content) => uri.push_str(tag_content),
                _ => {}
            }
        } else {
            match token {
                Token::CloseParenthesis => break,
                Token::Punctuation(s) if s == "\"" => {
                    if has_opening_quote {
                        is_valid_title = true;
                        is_building_title = false;
                    } else {
                        has_opening_quote = true;
                        is_valid_title = false;
                    }
                }
                Token::Text(s) | Token::Punctuation(s) => title.push_str(s),
                Token::OrderedListMarker(s) => title.push_str(s),
                Token::Escape(ch) => title.push_str(&format!("\\{ch}")),
                Token::EmphasisRun { delimiter, length } => {
                    title.push_str(&delimiter.to_string().repeat(*length))
                }
                Token::OpenBracket => title.push('['),
                Token::CloseBracket => title.push(']'),
                Token::OpenParenthesis => title.push('('),
                Token::TableCellSeparator => title.push('|'),
                Token::Tab => title.push('\t'),
                Token::Newline => title.push_str("\\n"),
                Token::Whitespace => title.push(' '),
                Token::CodeTick => title.push('`'),
                Token::CodeFence => title.push_str("```"),
                Token::ThematicBreak => title.push_str("---"),
                Token::BlockQuoteMarker => title.push('>'),
                Token::RawHtmlTag(tag_content) => {
                    warn!(
                        "Raw HTML tags in titles can result in unexpected behavior: {tag_content}"
                    );
                    title.push_str(tag_content);
                }
            }
        }
        cursor.advance();
    }

    // If we didn't find a closing parenthesis or if the title is invalid, treat it as text
    if cursor.current() != Some(&Token::CloseParenthesis) {
        return MdInlineElement::Text {
            content: format!("[{}]({} ", flatten_inline(&label_elements), uri),
        };
    } else if !title.is_empty() && !is_valid_title {
        return MdInlineElement::Text {
            content: format!("[{}]({} {})", flatten_inline(&label_elements), uri, title),
        };
    }

    make_element(label_elements, Some(title).filter(|t| !t.is_empty()), uri)
}

/// Flattens a vector of inline Markdown elements into a single string.
///
/// # Arguments
/// * `elements` - A vector of inline Markdown elements to flatten.
///
/// # Returns
/// A string containing the concatenated content of all inline elements
fn flatten_inline(elements: &[MdInlineElement]) -> String {
    let mut result = String::new();
    for element in elements {
        match element {
            MdInlineElement::Text { content } => result.push_str(content),
            MdInlineElement::Bold { content } => result.push_str(&flatten_inline(content)),
            MdInlineElement::Italic { content } => result.push_str(&flatten_inline(content)),
            MdInlineElement::Code { content } => result.push_str(content),
            MdInlineElement::Link { text, .. } => result.push_str(&flatten_inline(text)),
            MdInlineElement::Image { alt_text, .. } => result.push_str(alt_text),
            _ => {}
        }
    }
    result
}

/// Parses (resolves) emphasis in a vector of inline Markdown elements.
///
/// Modifies the elements in place to convert delimiter runs into bold or italic elements as appropriate.
///
/// # Arguments
/// * `elements` - A mutable reference to a vector of inline Markdown elements.
/// * `delimiter_stack` - A mutable reference to a slice of delimiters.
fn resolve_emphasis(elements: &mut Vec<MdInlineElement>, delimiter_stack: &mut [Delimiter]) {
    if delimiter_stack.len() == 1 {
        // If there is only one delimiter, it cannot be resolved to emphasis
        if delimiter_stack[0].active {
            elements[delimiter_stack[0].parsed_position] = MdInlineElement::Text {
                content: delimiter_stack[0].ch.to_string(),
            };
        }
        return;
    }

    for i in 0..delimiter_stack.len() {
        if !delimiter_stack[i].active || !delimiter_stack[i].can_close {
            continue;
        }

        // At this point we have a valid closer
        let closer = delimiter_stack[i].clone();

        for j in (0..i).rev() {
            if !delimiter_stack[j].active || !delimiter_stack[j].can_open {
                continue;
            }

            let opener = delimiter_stack[j].clone();

            // Check if the opener and closer have the same delimiter
            if !closer.ch.eq(&opener.ch) {
                continue;
            }

            // Rule of 3: If the total length of the run is a multiple of 3 and both run lengths
            // are not divisible by 3, they are not valid for emphasis
            let length_total = closer.run_length + opener.run_length;
            if ((closer.can_open && closer.can_close) || (opener.can_open && opener.can_close))
                && (length_total % 3 == 0
                    && closer.run_length % 3 != 0
                    && opener.run_length % 3 != 0)
            {
                continue;
            }

            // Prefer making bold connections first
            let delimiters_used = if closer.run_length >= 2 && opener.run_length >= 2 {
                2
            } else {
                1
            };

            // Replace the placeholders with the new element
            let range_start = if opener.run_length > delimiters_used {
                opener.parsed_position + 1
            } else {
                opener.parsed_position
            };

            let range_end = if closer.run_length >= delimiters_used {
                closer.parsed_position
            } else {
                closer.parsed_position + 1
            };

            // Map the delimiters used to bold/italic respectively
            let element_to_insert = match delimiters_used {
                2 => MdInlineElement::Bold {
                    content: elements[range_start + 1..range_end].to_vec(),
                },
                1 => MdInlineElement::Italic {
                    content: elements[range_start + 1..range_end].to_vec(),
                },
                _ => unreachable!(),
            };

            elements.splice(range_start..=range_end, vec![element_to_insert]);
            let num_elements_removed = range_end - range_start;

            // closer.parsed_position -= num_elements_removed;

            // Update the parsed positions of the delimiters
            (0..delimiter_stack.len()).for_each(|k| {
                if delimiter_stack[k].parsed_position > closer.parsed_position {
                    delimiter_stack[k].parsed_position -= num_elements_removed;
                }
            });

            delimiter_stack[i].run_length = delimiter_stack[i]
                .run_length
                .saturating_sub(delimiters_used);
            delimiter_stack[j].run_length = delimiter_stack[j]
                .run_length
                .saturating_sub(delimiters_used);

            if delimiter_stack[i].run_length == 0 {
                delimiter_stack[i].active = false;
            }
            if delimiter_stack[j].run_length == 0 {
                delimiter_stack[j].active = false;
            }
        }
    }

    // For all delimiters that are still active, replace the placeholders with Text elements
    delimiter_stack.iter_mut().for_each(|el| {
        if el.active && el.parsed_position < elements.len() {
            elements[el.parsed_position] = MdInlineElement::Text {
                content: el.ch.to_string(),
            };
        }
    });
}

/// Groups adjacent tokenized lines into groups (blocks) for further parsing.
///
/// # Arguments
/// * `tokenized_lines` - A vector of vectors, where each inner vector contains tokens representing a line of markdown.
///
/// # Returns
/// A vector of vectors, where each inner vector represents a grouped block of tokens.
pub fn group_lines_to_blocks(mut tokenized_lines: Vec<Vec<Token>>) -> Vec<Vec<Token>> {
    let mut blocks: Vec<Vec<Token>> = Vec::new();
    let mut current_block: Vec<Token> = Vec::new();
    let mut previous_block: Vec<Token>;
    let lines = tokenized_lines.iter_mut();
    let mut is_inside_code_block = false;
    for line in lines {
        previous_block = blocks.last().unwrap_or(&Vec::new()).to_vec();

        // Appending all tokens between two code fences to one block
        if is_inside_code_block && line.first() != Some(&Token::CodeFence) {
            // If we are inside a code block, then we just append the line to the current block
            attach_to_previous_block(&mut blocks, &mut previous_block, line, Some(Token::Newline));
            continue;
        } else if is_inside_code_block && line.first() == Some(&Token::CodeFence) {
            // If we are inside a code block and the line starts with a code fence, then we end the
            // code block
            is_inside_code_block = false;
            attach_to_previous_block(&mut blocks, &mut previous_block, line, None);
            continue;
        }

        match line.first() {
            Some(Token::Punctuation(string)) if string == "#" => {
                // For ATX headings, it must all be on one line
                blocks.push(line.to_owned());
            }
            Some(Token::Punctuation(string)) if string == "-" => {
                group_dashed_lines(&mut blocks, &mut current_block, &mut previous_block, line);
            }
            Some(Token::Tab) => {
                group_tabbed_lines(&mut blocks, &mut current_block, &mut previous_block, line);
            }
            Some(Token::OrderedListMarker(_)) => {
                group_ordered_list(&mut blocks, &mut current_block, &mut previous_block, line);
            }
            Some(Token::ThematicBreak) => {
                // Check if the previous line starts with anything other than a heading
                // If so, then this is actually a setext heading 2
                if let Some(previous_line_start) = previous_block.first() {
                    match previous_line_start {
                        Token::Punctuation(string) if string == "#" => {
                            blocks.push(line.to_owned());
                        }
                        Token::Newline => blocks.push(line.to_owned()),
                        _ => {
                            previous_block.insert(0, Token::Punctuation(String::from("#")));
                            previous_block.insert(1, Token::Punctuation(String::from("#")));
                            previous_block.insert(2, Token::Whitespace);
                            blocks.pop();
                            blocks.push(previous_block.clone());
                        }
                    }
                } else {
                    current_block.extend_from_slice(line);
                }
            }
            Some(Token::BlockQuoteMarker) => {
                if let Some(previous_line_start) = previous_block.first() {
                    if matches!(previous_line_start, Token::BlockQuoteMarker) {
                        attach_to_previous_block(
                            &mut blocks,
                            &mut previous_block,
                            line,
                            Some(Token::Newline),
                        );
                    } else {
                        current_block.extend_from_slice(line);
                    }
                } else {
                    current_block.extend_from_slice(line);
                }
            }
            Some(Token::CodeTick) => {
                current_block.extend_from_slice(line);
            }
            Some(Token::CodeFence) => {
                if !is_inside_code_block {
                    is_inside_code_block = true;
                    current_block.extend_from_slice(line);
                } else {
                    is_inside_code_block = false;
                    current_block.extend_from_slice(line);
                    blocks.push(current_block.clone());
                    current_block.clear();
                }
            }
            Some(Token::Text(string)) if string == "=" => {
                let has_trailing_content = line.iter().skip(1).any(|token| match token {
                    Token::Text(s) if s == "=" => false,
                    Token::Whitespace | Token::Tab | Token::Newline => false,
                    _ => true,
                });

                // Setext heading 1
                if let Some(previous_line_start) = previous_block.first() {
                    if !has_trailing_content && matches!(previous_line_start, Token::Text(_)) {
                        group_setext_heading_one(&mut blocks, &mut previous_block);
                    } else {
                        group_text_lines(
                            &mut blocks,
                            &mut current_block,
                            &mut previous_block,
                            line,
                        );
                    }
                } else {
                    current_block.extend_from_slice(line);
                }
            }
            Some(Token::Text(_)) => {
                group_text_lines(&mut blocks, &mut current_block, &mut previous_block, line);
            }
            Some(Token::TableCellSeparator) => {
                group_table_rows(&mut blocks, &mut current_block, &mut previous_block, line);
            }
            Some(Token::Whitespace) => {
                group_lines_with_leading_whitespace(
                    &mut blocks,
                    &mut current_block,
                    &mut previous_block,
                    line,
                );
            }
            _ => {
                // Catch-all for everything else
                current_block.extend_from_slice(line);
            }
        }

        if !current_block.is_empty() {
            blocks.push(current_block.clone());
        }

        current_block.clear();
    }
    blocks
}

/// Groups lines beginning with "|" denoting Markdown tables.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_table_rows(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if let Some(previous_line_start) = previous_block.first() {
        if previous_line_start == &Token::TableCellSeparator {
            attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
        } else {
            current_block.extend_from_slice(line);
        }
    } else {
        current_block.extend_from_slice(line);
    }
}

/// Groups text lines into blocks based on the previous block's content.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_text_lines(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if !previous_block.is_empty() {
        if matches!(previous_block.first(), Some(Token::Text(_))) {
            attach_to_previous_block(blocks, previous_block, line, Some(Token::Whitespace));
        } else if matches!(previous_block.first(), Some(Token::Punctuation(_))) {
            // If the previous block was a heading, then this is a new paragraph
            current_block.extend_from_slice(line);
        } else {
            // If the previous block was empty, then this is a new paragraph
            current_block.extend_from_slice(line);
        }
    } else {
        // If the previous block was empty, then this is a new paragraph
        current_block.extend_from_slice(line);
    }
}

/// Groups Setext heading 1 lines into a block by prepending the previous block with "# ".
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `previous_block` - A mutable reference to the previous block, which is modified to become a
///   Setext heading 1.
fn group_setext_heading_one(blocks: &mut Vec<Vec<Token>>, previous_block: &mut Vec<Token>) {
    previous_block.insert(0, Token::Punctuation(String::from("#")));
    previous_block.insert(1, Token::Whitespace);

    // Swap previous block in
    blocks.pop();
    blocks.push(previous_block.clone());
}

/// Groups ordered list lines into a block by appending the line to the previous block if it is
/// part of the same list.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_ordered_list(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if let Some(previous_line_start) = previous_block.first() {
        match previous_line_start {
            Token::OrderedListMarker(_) if previous_block.get(1) == Some(&Token::Whitespace) => {
                // If the previous block is a list, then we append the line to it
                attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
            }
            _ => {
                current_block.extend_from_slice(line);
            }
        }
    } else {
        current_block.extend_from_slice(line);
    }
}

/// Attaches the current line to the previous block, optionally adding a separator token.
fn attach_to_previous_block(
    blocks: &mut Vec<Vec<Token>>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
    separator: Option<Token>,
) {
    if let Some(separator) = separator {
        previous_block.push(separator);
    }

    previous_block.extend_from_slice(line);
    blocks.pop();
    blocks.push(previous_block.clone());
}

/// Groups tabbed lines into blocks based on the previous block's content.
///
/// Note that this function short-circuits when the first token of the line is a raw HTML tag,
/// to allow for indented HTML.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_tabbed_lines(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if line.len() == 1 {
        current_block.extend_from_slice(line);
        return;
    }

    let non_whitespace_index = line
        .iter()
        .position(|token| !matches!(token, Token::Whitespace | Token::Tab | Token::Newline));

    if let Some(first_content_token) = line.get(non_whitespace_index.unwrap_or(0)) {
        if matches!(first_content_token, Token::RawHtmlTag(_))
            && matches!(previous_block.first(), Some(Token::RawHtmlTag(_)))
        {
            // If the first token is a raw HTML tag, we attach the line to the previous block
            let line_to_attach = line
                .iter()
                .skip_while(|t| matches!(t, Token::Whitespace | Token::Tab | Token::Newline))
                .cloned()
                .collect::<Vec<Token>>();

            attach_to_previous_block(
                blocks,
                previous_block,
                &line_to_attach,
                Some(Token::Newline),
            );

            return;
        } else if matches!(first_content_token, Token::RawHtmlTag(_)) {
            current_block.extend(
                line.iter()
                    .skip_while(|t| matches!(t, Token::Whitespace | Token::Tab | Token::Newline))
                    .cloned(),
            );
            return;
        }

        if !previous_block.is_empty() {
            let previous_line_start = previous_block.first();
            match previous_line_start {
                Some(Token::Punctuation(string))
                    if string == "-" && previous_block.get(1) == Some(&Token::Whitespace) =>
                {
                    // If the previous block is a list, then we append the line to it
                    attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
                }
                Some(Token::OrderedListMarker(_))
                    if previous_block.get(1) == Some(&Token::Whitespace) =>
                {
                    // If the previous block is an ordered list, then we append the
                    // line to it
                    attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
                }
                Some(Token::RawHtmlTag(_)) => {
                    attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
                }
                Some(Token::Tab) => {
                    attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
                }
                _ => {
                    // If the previous block is not a list, then we just add the
                    // line to the current block
                    current_block.extend_from_slice(line);
                }
            }
        } else {
            // If the previous block is empty, then we just add the line to the
            // current block
            current_block.extend_from_slice(line);
        }
    }
}

/// Groups lines with leading whitespace into blocks based on the previous block's content.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_lines_with_leading_whitespace(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if let Some(first_content_token) = line
        .iter()
        .find(|t| !matches!(t, Token::Whitespace | Token::Tab | Token::Newline))
    {
        if let Some(previous_line_start) = previous_block.first() {
            match previous_line_start {
                Token::Whitespace => {
                    // Check if the previous line has non-whitespace content
                    if line
                        .iter()
                        .any(|t| !matches!(t, Token::Whitespace | Token::Tab | Token::Newline))
                    {
                        attach_to_previous_block(
                            blocks,
                            previous_block,
                            line,
                            Some(Token::Newline),
                        );
                    } else {
                        current_block.extend_from_slice(line);
                    }
                }
                Token::RawHtmlTag(_) => {
                    if matches!(first_content_token, Token::RawHtmlTag(_)) {
                        // If the first token is a raw HTML tag, we attach the line to the previous block
                        attach_to_previous_block(
                            blocks,
                            previous_block,
                            line,
                            Some(Token::Newline),
                        );
                    } else {
                        current_block.extend_from_slice(line);
                    }
                }
                Token::Punctuation(string) if string == "-" => {
                    if matches!(first_content_token, Token::Punctuation(_)) {
                        attach_to_previous_block(
                            blocks,
                            previous_block,
                            line,
                            Some(Token::Newline),
                        );
                    } else {
                        current_block.extend_from_slice(line);
                    }
                }
                Token::Text(_) | Token::Punctuation(_) => {
                    attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
                }
                _ => {
                    // Append the line to current block, excluding leading whitespace
                    current_block.extend(
                        line.iter()
                            .skip_while(|t| {
                                matches!(t, Token::Whitespace | Token::Tab | Token::Newline)
                            })
                            .cloned(),
                    );
                }
            }
        } else {
            current_block.extend_from_slice(line);
        }
    }
}

/// Groups dashed lines into blocks based on the previous block's content.
///
/// # Arguments
/// * `blocks` - A mutable reference to a vector of blocks, where each block is a vector of tokens.
/// * `current_block` - A mutable reference to the current block being processed.
/// * `previous_block` - A mutable reference to the previous block, used for context.
/// * `line` - A mutable reference to the current line being processed, which is a vector of
///   tokens.
fn group_dashed_lines(
    blocks: &mut Vec<Vec<Token>>,
    current_block: &mut Vec<Token>,
    previous_block: &mut Vec<Token>,
    line: &[Token],
) {
    if let Some(previous_line_start) = previous_block.first() {
        match previous_line_start {
            Token::Punctuation(string)
                if string == "-" && previous_block.get(1) == Some(&Token::Whitespace) =>
            {
                // Then it is either the start of a list or part of a list

                attach_to_previous_block(blocks, previous_block, line, Some(Token::Newline));
            }
            Token::Punctuation(string) if string == "#" => {
                blocks.push(line.to_owned());
            }
            _ => {
                if line.len() > 1 {
                    current_block.extend_from_slice(line);
                } else {
                    // Then this is a Setext heading 2
                    previous_block.insert(0, Token::Punctuation(String::from("#")));
                    previous_block.insert(1, Token::Punctuation(String::from("#")));
                    previous_block.insert(2, Token::Whitespace);
                    blocks.pop();
                    blocks.push(previous_block.clone());
                }
            }
        }
    } else {
        current_block.extend_from_slice(line);
    }
}

#[cfg(test)]
mod test;
