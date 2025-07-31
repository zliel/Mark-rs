//! This module provides functionality to generate HTML from markdown block elements.

use ammonia::clean;

use crate::CONFIG;
use crate::config::Config;
use crate::types::{MdBlockElement, ToHtml};
use crate::utils::build_rel_prefix;

/// Generates an HTML string from a vector of MdBlockElements
///
/// # Arguments
/// * `file_name` - The name of the markdown file, used to set the title of the HTML document.
/// * `md_elements` - A vector of `MdBlockElement` instances representing the markdown content.
/// * `output_dir` - The directory where the generated HTML file will be saved.
/// * `input_dir` - The directory where the markdown files are located, used for relative paths.
/// * `html_rel_path` - The relative path to the HTML file from the output directory, used for
///   linking resources.
///
/// # Returns
/// Returns a `String` containing the generated HTML.
pub fn generate_html(
    file_name: &str,
    md_elements: &[MdBlockElement],
    output_dir: &str,
    input_dir: &str,
    html_rel_path: &str,
) -> String {
    let mut html_output = String::new();
    let config = CONFIG.get().unwrap();

    let head = generate_head(file_name, html_rel_path, config);

    let mut body = String::from("\t<body>\n");
    body.push_str(&indent_html(&generate_navbar(html_rel_path), 2));
    body.push_str("\n\t\t<div id=\"content\">");

    let inner_html: String = md_elements
        .iter()
        .map(|element| element.to_html(output_dir, input_dir, html_rel_path))
        .collect::<Vec<String>>()
        .join("\n");

    let inner_html = if config.html.sanitize_html {
        let mut builder = ammonia::Builder::default();
        builder
            .add_tag_attributes("a", &["href", "title", "target"])
            .add_tag_attribute_values("a", "target", &["_blank", "_self"])
            .add_tag_attributes("pre", &["class"])
            .add_tag_attributes("code", &["class"])
            .add_tags(&["iframe"])
            .add_tag_attributes(
                "iframe",
                &[
                    "src",
                    "width",
                    "height",
                    "title",
                    "frameborder",
                    "allowfullscreen",
                ],
            );
        for tag in &["h1", "h2", "h3", "h4", "h5", "h6"] {
            builder.add_tag_attributes(tag, &["id"]);
        }

        builder.clean(&inner_html).to_string()
    } else {
        inner_html
    };

    body.push_str(&indent_html(&inner_html, 3));
    body.push_str("\n\t\t</div>");

    if config.html.use_prism {
        body.push_str(
            "\n\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/components/prism-core.min.js\" integrity=\"sha512-Uw06iFFf9hwoN77+kPl/1DZL66tKsvZg6EWm7n6QxInyptVuycfrO52hATXDRozk7KWeXnrSueiglILct8IkkA==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>",
        );
        body.push_str("\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/line-numbers/prism-line-numbers.min.js\" integrity=\"sha512-BttltKXFyWnGZQcRWj6osIg7lbizJchuAMotOkdLxHxwt/Hyo+cl47bZU0QADg+Qt5DJwni3SbYGXeGMB5cBcw==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>");
        body.push_str(
            "\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/autoloader/prism-autoloader.min.js\" integrity=\"sha512-SkmBfuA2hqjzEVpmnMt/LINrjop3GKWqsuLSSB3e7iBmYK7JuWw4ldmmxwD9mdm2IRTTi0OxSAfEGvgEi0i2Kw==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>"
        );
        body.push_str("\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/toolbar/prism-toolbar.min.js\" integrity=\"sha512-st608h+ZqzliahyzEpETxzU0f7z7a9acN6AFvYmHvpFhmcFuKT8a22TT5TpKpjDa3pt3Wv7Z3SdQBCBdDPhyWA==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>");
        body.push_str("\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/copy-to-clipboard/prism-copy-to-clipboard.min.js\" integrity=\"sha512-/kVH1uXuObC0iYgxxCKY41JdWOkKOxorFVmip+YVifKsJ4Au/87EisD1wty7vxN2kAhnWh6Yc8o/dSAXj6Oz7A==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>");
        body.push_str("\n\t\t<script src=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/show-language/prism-show-language.min.js\" integrity=\"sha512-d1t+YumgzdIHUL78me4B9NzNTu9Lcj6RdGVbdiFDlxRV9JTN9s+iBQRhUqLRq5xtWUp1AD+cW2sN2OlST716fw==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\"></script>");
    }

    body.push_str("\n\t</body>\n");

    html_output.push_str(&head);
    html_output.push_str(&body);
    html_output.push_str("</html>\n");

    html_output
}

/// Generates the index HTML file that lists all pages
///
/// # Arguments
/// * `file_names` - A slice of `String` containing the names of the markdown files.
///
/// # Returns
/// Returns a `String` containing the generated HTML for the index page.
pub fn generate_index(file_names: &[String]) -> String {
    let mut html_output = String::new();

    let head = generate_head("index", "index.html", CONFIG.get().unwrap());

    let mut body = String::from("\t<body>\n");
    body.push_str(&generate_navbar("index.html"));
    body.push_str("\n\t<div id=\"content\">\n");
    body.push_str("<h1>All Pages</h1>\n");

    file_names.iter().for_each(|file_name| {
        body.push_str(&format!(
            "<a href=\"./{}.html\">{}</a><br>\n",
            file_name.trim_end_matches(".md"),
            format_title(file_name)
        ));
    });

    body.push_str("\n</div>\n\t</body>\n");

    html_output.push_str(&head);
    html_output.push_str(&body);
    html_output.push_str("</html>\n");

    html_output
}

/// Generates the HTML head section
///
/// # Arguments
/// * `file_name` - The name of the markdown file, used to set the title of the HTML document.
/// * `html_rel_path` - The relative path to the HTML file from the output directory, used for
///   linking
fn generate_head(file_name: &str, html_rel_path: &str, config: &Config) -> String {
    let mut head = String::from(
        r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
    "#,
    );

    // Remove the file extension from the file name and make it title case
    let title = format_title(file_name);
    head.push_str(&format!("\t<title>{}</title>\n", title));

    let favicon_file = &config.html.favicon_file;
    if !favicon_file.is_empty() {
        let mut favicon_path = build_rel_prefix(html_rel_path);
        favicon_path.push("media");
        favicon_path.push(favicon_file.rsplit("/").next().unwrap());
        let favicon_href = favicon_path.to_string_lossy();

        head.push_str(&format!(
            "\t<link rel=\"icon\" href=\"{}\">\n",
            favicon_href
        ));
    }

    let css_file = &config.html.css_file;
    let mut css_path = build_rel_prefix(html_rel_path);
    css_path.push("styles.css");
    let css_href = css_path.to_string_lossy();

    if css_file == "default" {
        head.push_str(&format!(
            "\t\t<link rel=\"stylesheet\" href=\"{}\">\n",
            css_href
        ));
    } else {
        head.push_str(&format!(
            "\t\t<link rel=\"stylesheet\" href=\"{}\">\n",
            css_file
        ));
    }

    if config.html.use_prism {
        if !config.html.prism_theme.is_empty() {
            let theme = if config.html.sanitize_html {
                &clean(&config.html.prism_theme)
            } else {
                &config.html.prism_theme
            };

            head.push_str(&format!("\t\t<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/prism-themes/1.9.0/prism-{}.min.css\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\" />", theme));
        } else {
            head.push_str("\t\t<link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/prismjs@1.30.0/themes/prism-okaidia.min.css\">");
        }
        head.push_str("\t\t<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/toolbar/prism-toolbar.min.css\" integrity=\"sha512-Dqf5696xtofgH089BgZJo2lSWTvev4GFo+gA2o4GullFY65rzQVQLQVlzLvYwTo0Bb2Gpb6IqwxYWtoMonfdhQ==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\" />");
        head.push_str("\t\t<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/prism/1.30.0/plugins/line-numbers/prism-line-numbers.min.css\" integrity=\"sha512-cbQXwDFK7lj2Fqfkuxbo5iD1dSbLlJGXGpfTDqbggqjHJeyzx88I3rfwjS38WJag/ihH7lzuGlGHpDBymLirZQ==\" crossorigin=\"anonymous\" referrerpolicy=\"no-referrer\" />");
    }

    head.push_str("\t</head>\n");
    head
}

/// Generates the HTML for the navigation bar
fn generate_navbar(html_rel_path: &str) -> String {
    let mut navbar = String::from("<header>\n\t<nav>\n\t\t<ul>\n");

    let mut home_path = build_rel_prefix(html_rel_path);
    home_path.push("index.html");
    let home_href = home_path.to_string_lossy();

    navbar.push_str(&format!(
        "\t\t\t<li><a href=\"{}\">Home</a></li>",
        home_href
    ));
    navbar.push_str("\n\t\t</ul>\n\t</nav>\n</header>\n\n");
    navbar
}
/// Formats the file name to create a title for the HTML document
///
/// # Arguments
/// * `file_name` - The name of the file, typically ending with `.md`.
///
/// # Returns
/// The formatted title (i.e. "my_test_page.md" -> "My Test Page")
fn format_title(file_name: &str) -> String {
    let title = file_name.trim_end_matches(".md").replace('_', " ");

    title
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Indents each line of the given HTML string by the specified number of tabs.
pub fn indent_html(html: &str, level: usize) -> String {
    let indent = "\t".repeat(level);
    html.lines()
        .map(|line| {
            let first_non_whitespace_token = line.chars().find(|c| !c.is_whitespace());

            match first_non_whitespace_token {
                Some('<') => format!("{indent}{line}"),
                Some(_) => line.to_string(),
                None => line.to_string(), // If the line is empty or only whitespace, return it unchanged
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generates a default CSS stylesheet as a string.
pub fn generate_default_css() -> String {
    r#"
    body {
    background-color: #121212;
    color: #e0e0e0;
    font-family:
        -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu,
        Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
    line-height: 1.75;
    margin: 0;
    padding: 0;
    }

    /* Card-like container for the page content */
    #content {
    background-color: #1e1e1e;
    max-width: 780px;
    margin: 1.5rem auto;
    padding: 2rem;
    border-radius: 12px;
    box-shadow: 0 0 0 1px #2c2c2c;
    }

    header {
    background-color: #1a1a1a;
    border-bottom: 1px solid #333;
    position: sticky;
    top: 0;
    z-index: 1000;
    }

    nav {
    padding: 1rem 2rem;
    display: flex;
    justify-content: flex-start;
    }

    nav ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    gap: 1rem;
    }

    nav ul li {
    margin: 0;
    }

    nav ul li a {
    color: #ddd;
    text-decoration: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    transition: background-color 0.2s ease, color 0.2s ease;
    }

    nav ul li a:hover {
    background-color: #2f2f2f;
    color: #fff;
    }

    nav ul li a.active {
    background-color: #4ea1f3;
    color: #121212;
    }
    h1,
    h2,
    h3,
    h4,
    h5,
    h6 {
    color: #ffffff;
    line-height: 1.3;
    margin-top: 2rem;
    margin-bottom: 1rem;
    }

    h1 {
    font-size: 2.25rem;
    border-bottom: 2px solid #2c2c2c;
    padding-bottom: 0.3rem;
    }
    h2 {
    font-size: 1.75rem;
    border-bottom: 1px solid #2c2c2c;
    padding-bottom: 0.2rem;
    }
    h3 {
    font-size: 1.5rem;
    }
    h4 {
    font-size: 1.25rem;
    }
    h5,
    h6 {
    font-size: 1rem;
    font-weight: normal;
    }

    p {
    margin-bottom: 1.2rem;
    }

    a {
    color: #4ea1f3;
    text-decoration: none;
    transition: color 0.2s ease-in-out;
    }
    a:hover {
    color: #82cfff;
    text-decoration: underline;
    }

    img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1.5rem auto;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    /* Styles for when "use_prism = false" is set in config.toml */
    pre.non_prism {
    background-color: #2a2a2a;
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    font-size: 0.9rem;
    box-shadow: inset 0 0 0 1px #333;
    }
    pre.non_prism::before {
    counter-reset: listing;
    }
    code.non_prism {
    font-family: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
    font-style: normal;
    background-color: #2a2a2a;
    padding: 0.2em 0.4em;
    border-radius: 4px;
    font-size: 0.95em;
    color: #dcdcdc;
    }
    pre.non_prism code.non_prism {
    counter-increment: listing;
    padding: 0 0.4em;
    text-align: left;
    float: left;
    clear: left;
    }
    pre.non_prism code.non_prism::before {
    content: counter(listing) ". ";
    display: inline-block;
    font-size: 0.85em;
    float: left;
    height: 1em;
    padding-top: 0.2em;
    padding-left: auto;
    margin-left: auto;
    text-align: right;
    }

    code {
    font-style: normal;
    }

    blockquote {
    border-left: 4px solid #555;
    padding: 0.1rem 1rem;
    color: #aaa;
    font-style: italic;
    margin: 1.5rem 0;
    background-color: #1a1a1a;
    border-radius: 2px;
    }

    .toolbar-item {
    font-style: normal;
    margin-right: 0.2em;
    }

    ul,
    ol {
    padding-left: 1.5rem;
    margin-bottom: 1.2rem;
    }
    li {
    margin-bottom: 0.5rem;
    }

    table {
    width: 100%;
    border-spacing: 0;
    margin: 2rem 0;
    background-color: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    overflow: hidden;
    font-size: 0.95rem;
    }

    th,
    td {
    padding: 0.75rem 1rem;
    text-align: left;
    }

    th {
    background-color: #2a2a2a;
    color: #ffffff;
    font-weight: 600;
    }

    tr:nth-child(even) td {
    background-color: #222;
    }

    tr:hover td {
    background-color: #2f2f2f;
    }

    td {
    color: #ddd;
    border-top: 1px solid #333;
    }

    hr {
    border: none;
    border-top: 1px solid #333;
    margin: 2rem 0;
    }
    "#
    .to_string()
}
