# Mark-rs

[![CI](https://github.com/zliel/Mark-rs/actions/workflows/CI.yml/badge.svg)](https://github.com/zliel/Mark-rs/actions/workflows/CI.yml)
[![Publish](https://github.com/zliel/Mark-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/zliel/Mark-rs/actions/workflows/publish.yml)
[![Crates.io Version](https://img.shields.io/crates/v/mark-rs)](https://crates.io/crates/mark-rs)
[![docs](https://img.shields.io/badge/docs-main-blue)](https://zliel.github.io/Mark-rs/markrs/index.html)

Mark-rs is a multithreaded, 100% Commonmark-compliant Markdown parser and static site generator written in Rust.
It is designed to be fast, efficient, and easy to use.

## Table of Contents

<!--toc:start-->

- [Mark-rs](#mark-rs)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Options](#options)
  - [Configuration](#configuration)
  - [⚠️Note: Raw HTML](#note-raw-html)
  <!--toc:end-->

## Features

- Fast Markdown parsing
- HTML generation
- Multithreaded
- Custom configuration
- Easy-to-use CLI

## Preview

For the example input:

```markdown
# Hello, World

This is some sample Markdown content.
Here's a picture saying "Hello, World!":
![Image with a black background and white text saying "Hello, World!"](https://www.dummyimage.com/600x400/000/fff&text=Hello,+World!)
```

The following HTML page will be generated:
![Image of the generated HTML page with matching content](./media/example_screenshot.png)

## Installation

To install Mark-rs, you need to have Rust installed on your system. You can install Rust using [rustup](https://rustup.rs/).

### Install via Cargo

Once you have Rust installed, you can install Mark-rs using Cargo:

```bash
cargo install mark-rs
```

**Note**: Make sure to have the `~/.cargo/bin` directory in your `PATH` environment variable so you can run the `markrs` command from anywhere.

If it isn't already in your `PATH`, you can adding the following line to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`, etc.):

```bash:
export PATH="$HOME/.cargo/bin:$PATH"
```

On Windows, you can add the `C:\Users\your_user\.cargo\bin` directory to your `PATH` environment variable.

### Install via Pre-built Binaries

You can also download pre-built binaries for your platform from the [releases page](https://github.com/zliel/Mark-rs/releases)

From there, you can download the appropriate binary for your operating system and architecture, extract it, and use it directly.

### Updates

If Mark-rs was installed using Cargo, you can update it to the latest version by running:

```bash
cargo install mark-rs
```

If you installed Mark-rs using pre-built binaries, you can download the latest version from the [releases page](https://github.com/zliel/Mark-rs/releases)

## Usage

To use Mark-rs, you can run the following command in your terminal:

```bash
markrs [OPTIONS] <INPUT_DIR>
```

Where `<INPUT_DIR>` is the path to the directory of Markdown files you want to parse. The output will be written to `/output` by default.

### Options

You can also use the following CLI arguments to customize the behavior of Mark-rs:

- `-c, --config <CONFIG>`: Specify a custom configuration file (default: `./config.toml`).
- `-o, --output-dir <OUTPUT_DIR>`: Specify the output directory for the generated HTML files (default: `/output`).
- `-n --num_threads <NUM_THREADS>`: Specify the number of threads to use (default: 4).
- `-r, --recursive`: Recursively parse all Markdown files in the specified directory and its subdirectories. (default: false if not present)
- `-v, --verbose`: Enable verbose output, which will print additional information while the program is running.
- `-e, --exclude <EXCLUDED_FILES>`: Exclude specific files or directories from being parsed. You can specify multiple files or directories by separating them with spaces.
- `-O, --open`: Open the generated index.html in the default web browser.
- `-h, --help`: Display help information.
- `-V, --version`: Display the version of Mark-rs.

## Configuration

You can customize Mark-rs's behavior by specifying a config file to use. If a config file is not specified, then the default configuration directory will be checked; if no config file already exists, then the default `config.toml` file will be written.

The default configuration directories (defined by the [`dirs` crate](https://docs.rs/dirs/latest/dirs/) ) are:

| Platform | Value                                 | Example                                             |
| -------- | ------------------------------------- | --------------------------------------------------- |
| Linux    | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/your_user/.config/markrs                      |
| macOS    | `$HOME`/Library/Application Support   | /Users/your_user/Library/Application Support/markrs |
| Windows  | `{FOLDERID_RoamingAppData}`           | C:\Users\your_user\AppData\Roaming\markrs           |

Here is the default configuration:

```toml
# Tokenization
[lexer]
tab_size = 4

# HTML Generation
[html]
css_file = "default" # "default" for the default styles
favicon_file = ""    # Empty for no favicon
use_prism = false    # If "true", the CDN links for PrismJS will be used for codeblock highlighting
# Note that `use_prism = true` this will add `<script>` and `<link>` elements to the page
prism_theme = "vsc-dark-plus" # Will only take effect if "use_prism" is set to "true"
# See https://github.com/PrismJS/prism-themes for themes and https://cdnjs.com/libraries/prism-themes for what to set "prism_theme" to
sanitize_html = true # If "true", the HTML will be sanitized to prevent XSS attacks
```

## Note: Raw HTML

Mark-rs supports using raw HTML in input Markdown files, but it should be noted that using raw HTML can lead to security vulnerabilities, such as XSS (Cross-Site Scripting) attacks, if the input is not properly sanitized. Therefore, it is recommended to use raw HTML with caution and only when necessary. By default, Mark-rs will sanitize the generated HTML before it is written to each file, but you should still be careful when using raw HTML.

If you want to use raw HTML without any sanitizing, you can disable HTML sanitizing by setting `sanitize_html = false` in your `config.toml` file.

For more information on XSS attacks, see [OWASP](https://owasp.org/www-community/attacks/xss/) and the [OWASP XSS Prevention Cheat Sheet.](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
