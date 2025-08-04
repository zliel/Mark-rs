use crate::CONFIG;
use crate::config::Config;
use crate::lexer::tokenize;
use crate::parser::{parse_block, parse_inline};
use crate::types::{MdBlockElement::*, MdInlineElement::*, MdListItem, ToHtml};

use std::sync::Once;
static INIT: Once = Once::new();

fn init_test_config() {
    INIT.call_once(|| {
        CONFIG.get_or_init(Config::default);
    });
}

mod inline {
    use super::*;

    #[test]
    fn text() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("Plain text.")),
            vec![Text {
                content: String::from("Plain text.")
            }]
        );
    }

    #[test]
    fn escape_char() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("\\*escaped char\\*")),
            vec![Text {
                content: String::from("\\*escaped char\\*")
            }]
        );
    }

    #[test]
    fn bold() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("**Bold** text")),
            vec![
                Bold {
                    content: vec![Text {
                        content: String::from("Bold")
                    }]
                },
                Text {
                    content: String::from(" text")
                }
            ]
        );
    }

    #[test]
    fn italic() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("*Italic* text")),
            vec![
                Italic {
                    content: vec![Text {
                        content: String::from("Italic")
                    }]
                },
                Text {
                    content: String::from(" text")
                }
            ]
        );
    }

    #[test]
    fn multiple_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("This is **bold** and *italic* text.")),
            vec![
                Text {
                    content: String::from("This is ")
                },
                Bold {
                    content: vec![Text {
                        content: String::from("bold")
                    }]
                },
                Text {
                    content: String::from(" and ")
                },
                Italic {
                    content: vec![Text {
                        content: String::from("italic")
                    }]
                },
                Text {
                    content: String::from(" text.")
                }
            ]
        )
    }

    #[test]
    fn complex_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("This is **bold and *italic text***.")),
            vec![
                Text {
                    content: String::from("This is ")
                },
                Bold {
                    content: vec![
                        Text {
                            content: String::from("bold and ")
                        },
                        Italic {
                            content: vec![Text {
                                content: String::from("italic text")
                            }]
                        }
                    ]
                },
                Text {
                    content: String::from(".")
                }
            ]
        )
    }

    #[test]
    fn mixed_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("**_Bold and italic._**")),
            vec![Bold {
                content: vec![Italic {
                    content: vec![Text {
                        content: String::from("Bold and italic.")
                    }]
                }]
            }]
        )
    }

    #[test]
    fn mixed_emphasis_separated() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("_Italic **and bold**_")),
            vec![Italic {
                content: vec![
                    Text {
                        content: String::from("Italic ")
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("and bold")
                        }]
                    }
                ]
            }]
        )
    }

    #[test]
    fn malformed_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("***Bold and italics with *too many asterisks***")),
            vec![
                Text {
                    content: String::from("*")
                },
                Bold {
                    content: vec![
                        Text {
                            content: String::from("Bold and italics with ")
                        },
                        Italic {
                            content: vec![Text {
                                content: String::from("too many asterisks")
                            }]
                        }
                    ]
                },
            ]
        )
    }

    #[test]
    fn malformed_mixed_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("_**Bold and _not italic**")),
            vec![
                Text {
                    content: String::from("_")
                },
                Bold {
                    content: vec![
                        Text {
                            content: String::from("Bold and ")
                        },
                        Placeholder {
                            ch: '_',
                            token_position: 6
                        },
                        Text {
                            content: String::from("not italic")
                        }
                    ]
                },
            ]
        )
    }

    #[test]
    fn link() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("[link text](http://example.com)")),
            vec![Link {
                text: vec![Text {
                    content: String::from("link text")
                }],
                title: None,
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn link_with_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("[**bold link text**](http://example.com)")),
            vec![Link {
                text: vec![Bold {
                    content: vec![Text {
                        content: String::from("bold link text")
                    }]
                }],
                title: None,
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn link_with_internal_hashes() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("[link text with #hash](http://example.com)")),
            vec![Link {
                text: vec![Text {
                    content: String::from("link text with #hash")
                }],
                title: None,
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn link_with_mixed_emphasis() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize(
                "[_italic, **bold and italic**_](http://example.com)"
            )),
            vec![Link {
                text: vec![Italic {
                    content: vec![
                        Text {
                            content: String::from("italic, ")
                        },
                        Bold {
                            content: vec![Text {
                                content: String::from("bold and italic")
                            }]
                        }
                    ]
                }],
                title: None,
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn link_with_title() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("[link text](http://example.com \"Title\")")),
            vec![Link {
                text: vec![Text {
                    content: String::from("link text")
                }],
                title: Some(String::from("Title")),
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn link_with_emphasized_title() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize(
                "[**bold link text**](http://example.com \"Title with **bold**\")"
            )),
            vec![Link {
                text: vec![Bold {
                    content: vec![Text {
                        content: String::from("bold link text")
                    }]
                }],
                title: Some(String::from("Title with **bold**")),
                url: String::from("http://example.com")
            }]
        );
    }

    #[test]
    fn image() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("![alt text](http://example.com/image.png)")),
            vec![Image {
                alt_text: String::from("alt text"),
                title: None,
                url: String::from("http://example.com/image.png")
            }]
        );
    }

    #[test]
    fn image_with_title() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize(
                "![alt text](http://example.com/image.png \"Title\")"
            )),
            vec![Image {
                alt_text: String::from("alt text"),
                title: Some(String::from("Title")),
                url: String::from("http://example.com/image.png")
            }]
        );
    }

    #[test]
    fn image_with_empty_alt_text() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("![](http://example.com/image.png)")),
            vec![Image {
                alt_text: String::from(""),
                title: None,
                url: String::from("http://example.com/image.png")
            }]
        )
    }

    #[test]
    fn image_with_emphasized_alt_text() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize(
                "![**bold alt text**](http://example.com/image.png)"
            )),
            vec![Image {
                alt_text: String::from("bold alt text"), //Only plain string content is added
                title: None,
                url: String::from("http://example.com/image.png")
            }]
        );
    }

    #[test]
    fn image_with_emphasized_title() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize(
                "![alt text](http://example.com/image.png \"**bold title**\")"
            )),
            vec![Image {
                alt_text: String::from("alt text"),
                title: Some(String::from("**bold title**")),
                url: String::from("http://example.com/image.png")
            }]
        );
    }

    #[test]
    fn raw_inline_html() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("<span>Inline HTML</span>")),
            vec![Text {
                content: String::from("<span>Inline HTML</span>")
            }]
        );
    }

    #[test]
    fn malformed_raw_html_no_closing_bracket() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("<span Malformed HTML")),
            vec![Text {
                content: String::from("<span Malformed HTML")
            }]
        );
    }

    #[test]
    fn malformed_raw_html_no_closing_tag() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("<span>Unclosed HTML")),
            vec![Text {
                content: String::from("<span>Unclosed HTML")
            }]
        );
    }

    #[test]
    fn malformed_raw_html_mismatched_tags() {
        init_test_config();
        assert_eq!(
            parse_inline(&tokenize("<span>Unmatched </div> tags")),
            vec![Text {
                content: String::from("<span>Unmatched </div> tags")
            }]
        );
    }
}

mod block {
    use crate::{
        parser::{group_lines_to_blocks, parse_blocks},
        types::{MdTableCell, TableAlignment},
    };

    use super::*;

    #[test]
    fn heading() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("# Heading 1")),
            Some(Header {
                level: 1,
                content: vec![Text {
                    content: String::from("Heading 1")
                }]
            })
        );
    }

    #[test]
    fn multilevel_heading() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("### Heading 3")),
            Some(Header {
                level: 3,
                content: vec![Text {
                    content: String::from("Heading 3")
                }]
            })
        );
    }

    #[test]
    fn heading_with_internal_hashes() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("## Heading 2 with #internal #hashes")),
            Some(Header {
                level: 2,
                content: vec![Text {
                    content: String::from("Heading 2 with #internal #hashes")
                }]
            })
        );
    }

    #[test]
    fn heading_with_emphases() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("## Heading 2 with **bold words**")),
            Some(Header {
                level: 2,
                content: vec![
                    Text {
                        content: String::from("Heading 2 with ")
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("bold words")
                        }]
                    }
                ]
            })
        )
    }

    #[test]
    fn paragraph() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("This is a paragraph.")),
            Some(Paragraph {
                content: vec![Text {
                    content: String::from("This is a paragraph.")
                }]
            })
        );
    }

    #[test]
    fn multiple_paragraphs() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("First paragraph."),
                tokenize("Second paragraph.")
            ])),
            vec![Paragraph {
                content: vec![Text {
                    content: String::from("First paragraph. Second paragraph.")
                }]
            }]
        );
    }

    #[test]
    fn multiline_paragraphs() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("First line.\nSecond line.")),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from("First line.")
                    },
                    Text {
                        content: String::from("Second line.")
                    }
                ]
            })
        );
    }

    #[test]
    fn paragraph_with_emphasis() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("This is a paragraph with **bold text**.")),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from("This is a paragraph with ")
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("bold text")
                        }]
                    },
                    Text {
                        content: String::from(".")
                    }
                ]
            })
        );
    }

    #[test]
    fn paragraph_with_mixed_emphasis() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize(
                "This is a paragraph with **bold text** and *italic text*."
            )),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from("This is a paragraph with ")
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("bold text")
                        }]
                    },
                    Text {
                        content: String::from(" and ")
                    },
                    Italic {
                        content: vec![Text {
                            content: String::from("italic text")
                        }]
                    },
                    Text {
                        content: String::from(".")
                    }
                ]
            })
        );
    }

    #[test]
    fn paragraph_with_link() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize(
                "This is a paragraph with [a link](http://example.com)."
            )),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from("This is a paragraph with ")
                    },
                    Link {
                        text: vec![Text {
                            content: String::from("a link")
                        }],
                        title: None,
                        url: String::from("http://example.com")
                    },
                    Text {
                        content: String::from(".")
                    }
                ]
            })
        );
    }

    #[test]
    fn paragraph_with_image_and_emphasis() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize(
                "This is a paragraph with ![an image](http://example.com/image.png) and **bold text**."
            )),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from("This is a paragraph with ")
                    },
                    Image {
                        alt_text: String::from("an image"),
                        title: None,
                        url: String::from("http://example.com/image.png")
                    },
                    Text {
                        content: String::from(" and ")
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("bold text")
                        }]
                    },
                    Text {
                        content: String::from(".")
                    }
                ]
            })
        );
    }

    #[test]
    fn complex_paragraph() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize(
                    "This is a paragraph with **bold text**, *italic text*, and [a link](http://example.com)."
                ),
                tokenize(
                    "It also contains ![an image](http://example.com/image.png) and some `inline code`."
                ),
                tokenize("It also contains a `code span`, and the following code block:"),
                tokenize("```rust"),
                tokenize("fn main() {"),
                tokenize("    println!(\"Hello, world!\");"),
                tokenize("}"),
                tokenize("```")
            ])),
            vec![
                Paragraph {
                    content: vec![
                        Text {
                            content: String::from("This is a paragraph with ")
                        },
                        Bold {
                            content: vec![Text {
                                content: String::from("bold text")
                            }]
                        },
                        Text {
                            content: String::from(", ")
                        },
                        Italic {
                            content: vec![Text {
                                content: String::from("italic text")
                            }]
                        },
                        Text {
                            content: String::from(", and ")
                        },
                        Link {
                            text: vec![Text {
                                content: String::from("a link")
                            }],
                            title: None,
                            url: String::from("http://example.com")
                        },
                        Text {
                            content: String::from(". It also contains ")
                        },
                        Image {
                            alt_text: String::from("an image"),
                            title: None,
                            url: String::from("http://example.com/image.png")
                        },
                        Text {
                            content: String::from(" and some ")
                        },
                        Code {
                            content: String::from("inline code")
                        },
                        Text {
                            content: String::from(". It also contains a ")
                        },
                        Code {
                            content: String::from("code span")
                        },
                        Text {
                            content: String::from(", and the following code block:")
                        }
                    ]
                },
                CodeBlock {
                    language: Some(String::from("rust")),
                    lines: vec![
                        String::from("fn main() {"),
                        String::from("    println!(\"Hello, world!\");"),
                        String::from("}")
                    ]
                }
            ]
        )
    }

    #[test]
    fn unordered_list() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("- Item 1"),
                tokenize("- Item 2")
            ])),
            vec![UnorderedList {
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn unordered_list_with_nested_items() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("- Item 1"),
                tokenize("    - Nested Item 1.1"),
                tokenize("    - Nested Item 1.2"),
                tokenize("- Item 2")
            ])),
            vec![UnorderedList {
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    },
                    MdListItem {
                        content: UnorderedList {
                            items: vec![
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.1")
                                        }]
                                    }
                                },
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.2")
                                        }]
                                    }
                                }
                            ]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn unordered_list_using_asterisks() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("* Item 1"),
                tokenize("* Item 2")
            ])),
            vec![UnorderedList {
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn unordered_nested_list_with_asterisks() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("* Item 1"),
                tokenize("    * Nested Item 1.1"),
                tokenize("    * Nested Item 1.2"),
                tokenize("* Item 2")
            ])),
            vec![UnorderedList {
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    },
                    MdListItem {
                        content: UnorderedList {
                            items: vec![
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.1")
                                        }]
                                    }
                                },
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.2")
                                        }]
                                    }
                                }
                            ]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn unordered_list_with_mixed_delimiters() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("- Item 1"),
                tokenize("* New List")
            ])),
            vec![
                UnorderedList {
                    items: vec![MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    }]
                },
                UnorderedList {
                    items: vec![MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("New List")
                            }]
                        }
                    }]
                }
            ]
        );
    }

    #[test]
    fn unordered_nested_list_with_mixed_delimiters() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("- Item 1"),
                tokenize("    * Nested Item 1.1"),
                tokenize("    - Nested Item 1.2"),
                tokenize("* Item 2")
            ])),
            vec![
                UnorderedList {
                    items: vec![
                        MdListItem {
                            content: Paragraph {
                                content: vec![Text {
                                    content: String::from("Item 1")
                                }]
                            }
                        },
                        MdListItem {
                            content: UnorderedList {
                                items: vec![
                                    MdListItem {
                                        content: Paragraph {
                                            content: vec![Text {
                                                content: String::from("Nested Item 1.1")
                                            }]
                                        }
                                    },
                                    MdListItem {
                                        content: Paragraph {
                                            content: vec![Text {
                                                content: String::from("Nested Item 1.2")
                                            }]
                                        }
                                    }
                                ]
                            }
                        },
                    ]
                },
                UnorderedList {
                    items: vec![MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }]
                }
            ]
        );
    }

    #[test]
    fn unordered_list_with_inlines() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("1. **Bold Item 1**"),
                tokenize("2. *Italic Item 2*"),
                tokenize("3. [Link Item 3](http://example.com)"),
                tokenize("4. ![Image Item 4](http://example.com/image.png)"),
            ])),
            vec![OrderedList {
                starting_num: 1,
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Bold {
                                content: vec![Text {
                                    content: String::from("Bold Item 1")
                                }]
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Italic {
                                content: vec![Text {
                                    content: String::from("Italic Item 2")
                                }]
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Link {
                                text: vec![Text {
                                    content: String::from("Link Item 3")
                                }],
                                title: None,
                                url: String::from("http://example.com")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Image {
                                alt_text: String::from("Image Item 4"),
                                title: None,
                                url: String::from("http://example.com/image.png")
                            }]
                        }
                    }
                ]
            }]
        )
    }

    #[test]
    fn ordered_list() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("1. First"),
                tokenize("2. Second")
            ])),
            vec![OrderedList {
                starting_num: 1,
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("First")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Second")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn ordered_list_with_nested_items() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("1. Item 1"),
                tokenize("    1. Nested Item 1.1"),
                tokenize("    2. Nested Item 1.2"),
                tokenize("2. Item 2")
            ])),
            vec![OrderedList {
                starting_num: 1,
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 1")
                            }]
                        }
                    },
                    MdListItem {
                        content: OrderedList {
                            starting_num: 1,
                            items: vec![
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.1")
                                        }]
                                    }
                                },
                                MdListItem {
                                    content: Paragraph {
                                        content: vec![Text {
                                            content: String::from("Nested Item 1.2")
                                        }]
                                    }
                                }
                            ]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Item 2")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn ordered_list_with_different_starting_num() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("5. Fifth Item"),
                tokenize("6. Sixth Item")
            ])),
            vec![OrderedList {
                starting_num: 5,
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Fifth Item")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Text {
                                content: String::from("Sixth Item")
                            }]
                        }
                    }
                ]
            }]
        );
    }

    #[test]
    fn ordered_list_with_inlines() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("1. **Bold Item 1**"),
                tokenize("2. *Italic Item 2*"),
                tokenize("3. [Link Item 3](http://example.com)"),
                tokenize("4. ![Image Item 4](http://example.com/image.png \"Some title\")"),
            ])),
            vec![OrderedList {
                starting_num: 1,
                items: vec![
                    MdListItem {
                        content: Paragraph {
                            content: vec![Bold {
                                content: vec![Text {
                                    content: String::from("Bold Item 1")
                                }]
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Italic {
                                content: vec![Text {
                                    content: String::from("Italic Item 2")
                                }]
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Link {
                                text: vec![Text {
                                    content: String::from("Link Item 3")
                                }],
                                title: None,
                                url: String::from("http://example.com")
                            }]
                        }
                    },
                    MdListItem {
                        content: Paragraph {
                            content: vec![Image {
                                alt_text: String::from("Image Item 4"),
                                title: Some(String::from("Some title")),
                                url: String::from("http://example.com/image.png")
                            }]
                        }
                    }
                ]
            }]
        )
    }

    #[test]
    fn blockquote() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("> This is a blockquote.")),
            Some(BlockQuote {
                content: vec![Paragraph {
                    content: vec![Text {
                        content: String::from("This is a blockquote.")
                    }]
                }]
            })
        );
    }

    #[test]
    fn blockquote_with_nested_block_elements() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("> This is a blockquote with a nested list:"),
                tokenize("> - Item 1"),
                tokenize("> - Item 2")
            ])),
            vec![BlockQuote {
                content: vec![
                    Paragraph {
                        content: vec![Text {
                            content: String::from("This is a blockquote with a nested list:")
                        }]
                    },
                    UnorderedList {
                        items: vec![
                            MdListItem {
                                content: Paragraph {
                                    content: vec![Text {
                                        content: String::from("Item 1")
                                    }]
                                }
                            },
                            MdListItem {
                                content: Paragraph {
                                    content: vec![Text {
                                        content: String::from("Item 2")
                                    }]
                                }
                            }
                        ]
                    }
                ]
            }]
        );
    }

    #[test]
    fn code_block() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("```\ncode block\n```")),
            Some(CodeBlock {
                language: None,
                lines: vec![String::from("code block")]
            })
        );
    }

    #[test]
    fn fenced_code_block_with_language() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("```rust\nfn main() {}\n```")),
            Some(CodeBlock {
                language: Some(String::from("rust")),
                lines: vec![String::from("fn main() {}")]
            })
        );
    }

    #[test]
    fn raw_html_basic() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("<div>Raw HTML content</div>")),
            Some(RawHtml {
                content: String::from("<div>Raw HTML content</div>")
            })
        );
    }

    #[test]
    fn raw_html_with_attributes() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("<img src=\"image.png\" alt=\"Image\"/>")),
            Some(RawHtml {
                content: String::from("<img src=\"image.png\" alt=\"Image\"/>")
            })
        );
    }

    #[test]
    fn raw_inline_html() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("This is <span>inline HTML</span> content.")),
            Some(Paragraph {
                content: vec![Text {
                    content: String::from("This is <span>inline HTML</span> content.")
                }]
            })
        );
    }

    #[test]
    fn mixed_markdown_and_html() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize(
                "This is a paragraph with strong <strong>HTML</strong> and **Markdown**."
            )),
            Some(Paragraph {
                content: vec![
                    Text {
                        content: String::from(
                            "This is a paragraph with strong <strong>HTML</strong> and "
                        )
                    },
                    Bold {
                        content: vec![Text {
                            content: String::from("Markdown")
                        }]
                    },
                    Text {
                        content: String::from(".")
                    }
                ]
            })
        );
    }

    #[test]
    fn malformed_raw_html_no_closing_bracket() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("<div Malformed HTML")),
            Some(Paragraph {
                content: vec![Text {
                    content: String::from("<div Malformed HTML")
                }]
            })
        );
    }

    #[test]
    fn malformed_raw_html_no_closing_tag() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("<div>Unclosed HTML")),
            Some(RawHtml {
                content: String::from("<div>Unclosed HTML")
            })
        );
    }

    #[test]
    fn malformed_raw_html_mismatched_tags() {
        init_test_config();
        assert_eq!(
            parse_block(&tokenize("<div>Unmatched </span> tags")),
            Some(RawHtml {
                content: String::from("<div>Unmatched </span> tags")
            })
        );
    }

    #[test]
    fn table_all_left_align() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 |"),
                tokenize("| :-- | :-- |"),
                tokenize("| Cell 1 | Cell 2 |"),
                tokenize("| Cell 3 | Cell 4 |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 1 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 2 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ],
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 3 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 4 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ]
                ]
            }]
        );
    }

    #[test]
    fn table_mixed_align() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 | Header 3 |"),
                tokenize("| :-- | :-: | --: |"),
                tokenize("| Cell 1 | Cell 2 | Cell 3 |"),
                tokenize("| Cell 4 | Cell 5 | Cell 6 |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::Center,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 3 ")
                        }],
                        alignment: TableAlignment::Right,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 1 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 2 ")
                            }],
                            alignment: TableAlignment::Center,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 3 ")
                            }],
                            alignment: TableAlignment::Right,
                            is_header: false,
                        }
                    ],
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 4 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 5 ")
                            }],
                            alignment: TableAlignment::Center,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 6 ")
                            }],
                            alignment: TableAlignment::Right,
                            is_header: false,
                        }
                    ]
                ]
            }]
        );
    }

    #[test]
    fn table_no_align() {
        init_test_config();

        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 |"),
                tokenize("| -- | -- |"),
                tokenize("| Cell 1 | Cell 2 |"),
                tokenize("| Cell 3 | Cell 4 |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::None,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::None,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 1 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 2 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        }
                    ],
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 3 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 4 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        }
                    ]
                ]
            }]
        );
    }

    #[test]
    fn table_with_inline_content() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 |"),
                tokenize("| :-- | :-- |"),
                tokenize("| **Bold Cell** | *Italic Cell* |"),
                tokenize("| [Link](http://example.com) | ![Image](http://example.com/image.png) |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![
                                Text {
                                    content: " ".to_string()
                                },
                                Bold {
                                    content: vec![Text {
                                        content: String::from("Bold Cell")
                                    }]
                                },
                                Text {
                                    content: " ".to_string()
                                }
                            ],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![
                                Text {
                                    content: " ".to_string()
                                },
                                Italic {
                                    content: vec![Text {
                                        content: String::from("Italic Cell")
                                    }]
                                },
                                Text {
                                    content: " ".to_string()
                                }
                            ],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ],
                    vec![
                        MdTableCell {
                            content: vec![
                                Text {
                                    content: " ".to_string()
                                },
                                Link {
                                    text: vec![Text {
                                        content: String::from("Link")
                                    }],
                                    title: None,
                                    url: String::from("http://example.com")
                                },
                                Text {
                                    content: " ".to_string()
                                }
                            ],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![
                                Text {
                                    content: " ".to_string()
                                },
                                Image {
                                    alt_text: String::from("Image"),
                                    title: None,
                                    url: String::from("http://example.com/image.png")
                                },
                                Text {
                                    content: " ".to_string()
                                }
                            ],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ]
                ]
            }]
        );
    }

    #[test]
    fn table_with_empty_cells() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 |"),
                tokenize("| :-- | :-- |"),
                tokenize("| Cell 1 ||"),
                tokenize("|| Cell 4 |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::Left,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 1 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ],
                    vec![
                        MdTableCell {
                            content: vec![],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 4 ")
                            }],
                            alignment: TableAlignment::Left,
                            is_header: false,
                        }
                    ]
                ]
            }]
        );
    }

    #[test]
    fn table_with_missing_cell() {
        init_test_config();
        assert_eq!(
            parse_blocks(&group_lines_to_blocks(vec![
                tokenize("| Header 1 | Header 2 |"),
                tokenize("| -- | -- |"),
                tokenize("| Cell 1 | Cell 2 |"),
                tokenize("| Cell 3 |")
            ])),
            vec![Table {
                headers: vec![
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 1 ")
                        }],
                        alignment: TableAlignment::None,
                        is_header: true,
                    },
                    MdTableCell {
                        content: vec![Text {
                            content: String::from(" Header 2 ")
                        }],
                        alignment: TableAlignment::None,
                        is_header: true,
                    }
                ],
                body: vec![
                    vec![
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 1 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        },
                        MdTableCell {
                            content: vec![Text {
                                content: String::from(" Cell 2 ")
                            }],
                            alignment: TableAlignment::None,
                            is_header: false,
                        }
                    ],
                    vec![MdTableCell {
                        content: vec![Text {
                            content: String::from(" Cell 3 ")
                        }],
                        alignment: TableAlignment::None,
                        is_header: false,
                    }]
                ]
            }]
        )
    }
}

mod html_generation {
    use crate::parser::{group_lines_to_blocks, parse_blocks};

    use super::*;

    mod inline {
        use super::*;

        #[test]
        fn text() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("Plain text."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "Plain text."
            );
        }

        #[test]
        fn escape_char() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("\\*escaped chars work\\*"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "\\*escaped chars work\\*"
            );
        }

        #[test]
        fn bold() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("**Bold** text"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<b>Bold</b> text"
            );
        }

        #[test]
        fn italic() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("*Italic* text"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<i>Italic</i> text"
            );
        }

        #[test]
        fn mixed_emphasis() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("This is **bold** and *italic* text."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "This is <b>bold</b> and <i>italic</i> text."
            );
        }

        #[test]
        fn complex_emphasis() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("This is **bold and *italic text***."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "This is <b>bold and <i>italic text</i></b>."
            );
        }

        #[test]
        fn mixed_emphasis_separated() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("_Italic **and bold**_"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<i>Italic <b>and bold</b></i>"
            );
        }

        #[test]
        fn malformed_emphasis() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("***Bold and italics with *too many asterisks***"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "*<b>Bold and italics with <i>too many asterisks</i></b>"
            )
        }

        #[test]
        fn malformed_mixed_emphasis() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("_**Bold and _not italic**"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "_<b>Bold and _not italic</b>"
            )
        }

        #[test]
        fn link() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("[link text](http://example.com)"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<a href=\"http://example.com\" target=\"_blank\">link text⮺</a>"
            );
        }

        #[test]
        fn image() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("![alt text](http://example.com/image.png)"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<img src=\"http://example.com/image.png\" alt=\"alt text\"/>"
            );
        }

        #[test]
        fn code_span() {
            init_test_config();
            assert_eq!(
                parse_inline(&tokenize("This is `inline code`."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "This is <code>inline code</code>."
            );
        }
    }

    mod block {
        use super::*;

        #[test]
        fn plain_text_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("Plain text."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p>Plain text.</p>"
            );
        }

        #[test]
        fn bold_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("**Bold** text"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p><b>Bold</b> text</p>"
            );
        }

        #[test]
        fn italic_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("*Italic* text"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p><i>Italic</i> text</p>"
            );
        }

        #[test]
        fn mixed_emphasis_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("This is **bold** and *italic* text."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p>This is <b>bold</b> and <i>italic</i> text.</p>"
            );
        }

        #[test]
        fn link_in_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("[link text](http://example.com)"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p><a href=\"http://example.com\" target=\"_blank\">link text⮺</a></p>"
            );
        }

        #[test]
        fn image_in_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("![alt text](http://example.com/image.png)"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p><img src=\"http://example.com/image.png\" alt=\"alt text\"/></p>"
            );
        }

        #[test]
        fn code_span_in_paragraph() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("This is `inline code`."))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<p>This is <code>inline code</code>.</p>"
            );
        }

        #[test]
        fn heading() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("# Heading 1"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "\n<h1 id=\"heading-1\">Heading 1</h1>\n"
            );
        }

        #[test]
        fn multilevel_heading() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("### Heading 3"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "\n<h3 id=\"heading-3\">Heading 3</h3>\n"
            );
        }

        #[test]
        fn heading_with_emphasis() {
            init_test_config();
            assert_eq!(
                parse_block(&tokenize("## Heading 2 with **bold words**"))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "\n<h2 id=\"heading-2-with-bold-words\">Heading 2 with <b>bold words</b></h2>\n"
            );
        }

        #[test]
        fn code_block() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(
                    ["```\n", "code block", "second line", "```"]
                        .iter()
                        .map(|tokens| tokenize(tokens))
                        .collect::<Vec<_>>()
                ))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<pre class=\"non_prism\"><code class=\"non_prism\">code block</code><code class=\"non_prism\">second line</code></pre>"
            );
        }

        #[test]
        fn code_block_with_language() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(
                    ["```rust", "fn main() {}", "```"]
                        .iter()
                        .map(|tokens| tokenize(tokens))
                        .collect::<Vec<_>>()
                ))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<pre class=\"non_prism\"><code class=\"non_prism\">fn main() {}</code></pre>"
            );
        }

        #[test]
        fn unordered_list() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("- Item 1"),
                    tokenize("- Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn unordered_list_using_asterisks() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("* Item 1"),
                    tokenize("* Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn unordered_list_with_nested_items() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("- Item 1"),
                    tokenize("    - Nested Item 1.1"),
                    tokenize("    - Nested Item 1.2"),
                    tokenize("- Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<ul>\n\t\t<li>\n\t\t\t<p>Nested Item 1.1</p>\n\t\t</li>\n\t\t<li>\n\t\t\t<p>Nested Item 1.2</p>\n\t\t</li>\n\t</ul><li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn unordered_nested_list_with_asterisks() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("* Item 1"),
                    tokenize("    * Nested Item 1.1"),
                    tokenize("    * Nested Item 1.2"),
                    tokenize("* Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<ul>\n\t\t<li>\n\t\t\t<p>Nested Item 1.1</p>\n\t\t</li>\n\t\t<li>\n\t\t\t<p>Nested Item 1.2</p>\n\t\t</li>\n\t</ul><li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn unordered_list_with_mixed_delimiters() {
            // Will be two separate lists
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("- Item 1"),
                    tokenize("* Item 2"),
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n</ul><ul>\n\t<li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn unordered_nested_list_with_mixed_delimiters() {
            // Will be two separate lists
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("- Item 1"),
                    tokenize("    * Nested Item 1.1"),
                    tokenize("    - Nested Item 1.2"),
                    tokenize("* Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ul>\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<ul>\n\t\t<li>\n\t\t\t<p>Nested Item 1.1</p>\n\t\t</li>\n\t\t<li>\n\t\t\t<p>Nested Item 1.2</p>\n\t\t</li>\n\t</ul>\n</ul><ul>\n\t<li>\n\t\t<p>Item 2</p>\n\t</li>\n</ul>"
            );
        }

        #[test]
        fn ordered_list() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("1. First"),
                    tokenize("2. Second")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ol start=\"1\">\n\t<li>\n\t\t<p>First</p>\n\t</li>\n\t<li>\n\t\t<p>Second</p>\n\t</li>\n</ol>"
            );
        }

        #[test]
        fn ordered_list_with_nested_items() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("1. Item 1"),
                    tokenize("    1. Nested Item 1.1"),
                    tokenize("    2. Nested Item 1.2"),
                    tokenize("2. Item 2")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ol start=\"1\">\n\t<li>\n\t\t<p>Item 1</p>\n\t</li>\n\t<ol start=\"1\">\n\t<li>\n\t\t<p>Nested Item 1.1</p>\n\t</li>\n\t<li>\n\t\t<p>Nested Item 1.2</p>\n\t</li>\n\n\t</ol><li>\n\t\t<p>Item 2</p>\n\t</li>\n</ol>"
            );
        }

        #[test]
        fn ordered_list_with_inlines() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("1. **Bold Item 1**"),
                    tokenize("2. *Italic Item 2*"),
                    tokenize("3. [Link Item 3](http://example.com)"),
                    tokenize("4. ![Image Item 4](http://example.com/image.png \"Some title\")"),
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ol start=\"1\">\n\t<li>\n\t\t<p><b>Bold Item 1</b></p>\n\t</li>\n\t<li>\n\t\t<p><i>Italic Item 2</i></p>\n\t</li>\n\t<li>\n\t\t<p><a href=\"http://example.com\" target=\"_blank\">Link Item 3⮺</a></p>\n\t</li>\n\t<li>\n\t\t<p><img src=\"http://example.com/image.png\" alt=\"Image Item 4\" title=\"Some title\"/></p>\n\t</li>\n</ol>"
            );
        }

        #[test]
        fn ordered_list_with_different_starting_num() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("5. Item 5"),
                    tokenize("6. Item 6")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<ol start=\"5\">\n\t<li>\n\t\t<p>Item 5</p>\n\t</li>\n\t<li>\n\t\t<p>Item 6</p>\n\t</li>\n</ol>"
            );
        }

        #[test]
        fn blockquote() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![tokenize(
                    "> This is a blockquote."
                )]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<blockquote>\n<p>This is a blockquote.</p>\n</blockquote>"
            );
        }

        #[test]
        fn blockquote_with_nested_block_element() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("> This is a blockquote with a nested heading:"),
                    tokenize("> # Heading 1"),
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<blockquote>\n<p>This is a blockquote with a nested heading:</p>\n<h1 id=\"heading-1\">Heading 1</h1>\n\n</blockquote>"
            );
        }

        #[test]
        fn raw_html_basic() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("<br>",),
                    tokenize("<h1>Hello, world!</h1>")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<br>\n<h1>Hello, world!</h1>\n"
            );
        }

        #[test]
        fn raw_html_with_attributes() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![tokenize(
                    "<img src=\"image.jpg\" alt=\"An image\"/>"
                )]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<img src=\"image.jpg\" alt=\"An image\"/>\n"
            );
        }

        #[test]
        fn mixed_markdown_and_html() {
            // Note that this heading's id is abnormal because RawHtml Tokens don't store the
            // token content between tags, only the content of the tags themselves.
            // This is only a problem if headings contain raw html.
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("# This is a heading with <strong>bold text</strong> and <em>italic text</em>."),
                    tokenize("<div>Some raw HTML content</div>")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "\n<h1 id=\"this-is-a-heading-with-bold-text-and-italic-text\">This is a heading with <strong>bold text</strong> and <em>italic text</em>.</h1>\n<div>Some raw HTML content</div>\n"
            );
        }

        #[test]
        fn malformed_raw_html_no_closing_bracket() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![tokenize(
                    "<div Missing bracket"
                )]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<p><div Missing bracket</p>"
            );
        }

        #[test]
        fn malformed_raw_html_closing_tag() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![tokenize("<div>Unclosed tag")]))
                    .iter()
                    .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                    .collect::<String>(),
                "<div>Unclosed tag\n"
            );
        }

        #[test]
        fn malformed_raw_html_mismatched_tags() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![tokenize(
                    "<div>Unmatched <span> tags"
                )]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<div>Unmatched <span> tags\n"
            );
        }

        #[test]
        fn table_all_left_align() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 |"),
                    tokenize("| :-- | :-- |"),
                    tokenize("| Cell 1 | Cell 2 |"),
                    tokenize("| Cell 3 | Cell 4 |")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:left;\"> Header 2 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 1 </td>\n\t\t\t<td style=\"text-align:left;\"> Cell 2 </td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 3 </td>\n\t\t\t<td style=\"text-align:left;\"> Cell 4 </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }

        #[test]
        fn table_mixed_align() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 | Header 3 |"),
                    tokenize("| :-- | :-: | --: |"),
                    tokenize("| Cell 1 | Cell 2 | Cell 3 |"),
                    tokenize("| Cell 4 | Cell 5 | Cell 6 |")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:center;\"> Header 2 </th>\n\t\t\t<th style=\"text-align:right;\"> Header 3 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 1 </td>\n\t\t\t<td style=\"text-align:center;\"> Cell 2 </td>\n\t\t\t<td style=\"text-align:right;\"> Cell 3 </td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 4 </td>\n\t\t\t<td style=\"text-align:center;\"> Cell 5 </td>\n\t\t\t<td style=\"text-align:right;\"> Cell 6 </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }

        #[test]
        fn table_no_align() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 |"),
                    tokenize("| -- | -- |"),
                    tokenize("| Cell 1 | Cell 2 |"),
                    tokenize("| Cell 3 | Cell 4 |")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:left;\"> Header 2 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 1 </td>\n\t\t\t<td style=\"text-align:left;\"> Cell 2 </td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 3 </td>\n\t\t\t<td style=\"text-align:left;\"> Cell 4 </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }

        #[test]
        fn table_with_inline_content() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 |"),
                    tokenize("| :-- | :-- |"),
                    tokenize("| **Bold Cell** | *Italic Cell* |"),
                    tokenize(
                        "| [Link](http://example.com) | ![Image](http://example.com/image.png) |"
                    )
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:left;\"> Header 2 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> <b>Bold Cell</b> </td>\n\t\t\t<td style=\"text-align:left;\"> <i>Italic Cell</i> </td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> <a href=\"http://example.com\" target=\"_blank\">Link⮺</a> </td>\n\t\t\t<td style=\"text-align:left;\"> <img src=\"http://example.com/image.png\" alt=\"Image\"/> </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }

        #[test]
        fn table_with_empty_cells() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 |"),
                    tokenize("| :-- | :-- |"),
                    tokenize("| Cell 1 ||"),
                    tokenize("|| Cell 4 |")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:left;\"> Header 2 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 1 </td>\n\t\t\t<td style=\"text-align:left;\"></td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"></td>\n\t\t\t<td style=\"text-align:left;\"> Cell 4 </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }

        #[test]
        fn table_with_missing_cell() {
            init_test_config();
            assert_eq!(
                parse_blocks(&group_lines_to_blocks(vec![
                    tokenize("| Header 1 | Header 2 |"),
                    tokenize("| -- | -- |"),
                    tokenize("| Cell 1 | Cell 2 |"),
                    tokenize("| Cell 3 |")
                ]))
                .iter()
                .map(|el| el.to_html("test_output", "test_input", "test_rel_path"))
                .collect::<String>(),
                "<table>\n\t<thead>\n\t\t<tr>\n\t\t\t<th style=\"text-align:left;\"> Header 1 </th>\n\t\t\t<th style=\"text-align:left;\"> Header 2 </th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 1 </td>\n\t\t\t<td style=\"text-align:left;\"> Cell 2 </td>\n\t\t</tr>\n\t\t<tr>\n\t\t\t<td style=\"text-align:left;\"> Cell 3 </td>\n\t\t</tr>\n\t</tbody>\n</table>"
            );
        }
    }
}
