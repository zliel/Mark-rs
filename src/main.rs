mod config;
mod error;
mod html_generator;
mod io;
mod lexer;
mod parser;
mod thread_pool;
mod types;
mod utils;

use clap::{Parser, command};
use env_logger::Env;
use log::{error, info};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use crate::config::{Config, init_config};
use crate::error::Error;
use crate::html_generator::{generate_html, generate_index};
use crate::io::{
    copy_css_to_output_dir, copy_favicon_to_output_dir, read_input_dir, write_default_css_file,
    write_html_to_file,
};
use crate::lexer::tokenize;
use crate::parser::{group_lines_to_blocks, parse_blocks};
use crate::thread_pool::ThreadPool;
use crate::types::Token;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(
    author = "Zackary Liel",
    version = "1.3.3",
    about = "A Commonmark compliant markdown parser and static site generator.",
    override_usage = "markrs [OPTIONS] <INPUT_DIR>"
)]
struct Cli {
    #[arg(value_name = "INPUT_DIR")]
    input_dir: String,
    #[arg(short, long, default_value = "")]
    config: String,
    #[arg(short, long, default_value = "./output")]
    output_dir: String,
    #[arg(short, long, default_value = "false")]
    recursive: bool,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    #[arg(short, long, default_value = "4")]
    num_threads: usize,
    #[arg(
        short = 'O',
        long,
        default_value = "false",
        help = "Open the generated index.html in the default web browser."
    )]
    open: bool,
}

fn main() -> Result<(), Error> {
    match run() {
        Ok(_) => {
            info!("Static site generation completed successfully.");
            Ok(())
        }
        Err(e) => {
            error!("An error occurred: {e}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let input_dir = &cli.input_dir;
    let config_path = &cli.config;
    let run_recursively = &cli.recursive;
    let num_threads = cli.num_threads;

    // Setup
    let env = if cli.verbose {
        Env::default().default_filter_or("info")
    } else {
        Env::default().default_filter_or("warn")
    };
    env_logger::Builder::from_env(env).init();

    init_config(config_path)?;
    let config = CONFIG.get().unwrap();
    let file_contents = read_input_dir(input_dir, run_recursively)?;
    let mut file_names: Vec<String> = Vec::with_capacity(file_contents.len());

    let thread_pool = ThreadPool::build(num_threads).map_err(|e| {
        error!("Failed to create thread pool: {e}");
        e
    })?;
    let cli = Arc::new(cli);

    for (file_path, file_content) in file_contents {
        info!("Generating HTML for file: {}", file_path);

        file_names.push(file_path.clone());

        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    generate_static_site(cli, &file_path, &file_content).unwrap_or_else(|e| {
                        error!("Failed to generate HTML for {file_path}: {e}");
                    });
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool: {e}");
                e
            })?;
    }

    thread_pool
        .execute({
            let cli = Arc::clone(&cli);
            move || {
                let index_html = generate_index(&file_names);
                write_html_to_file(&index_html, &cli.output_dir, "index.html").unwrap_or_else(
                    |e| {
                        error!("Failed to write index.html: {e}");
                    },
                );
            }
        })
        .map_err(|e| {
            error!("Failed to execute job in thread pool for index generation: {e}");
            e
        })?;

    let css_file = &config.html.css_file;
    if css_file != "default" && !css_file.is_empty() {
        info!("Using custom CSS file: {}", css_file);
        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    copy_css_to_output_dir(css_file, &cli.output_dir).unwrap_or_else(|e| {
                        error!("Failed to copy CSS file: {e}");
                    });
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool for copying CSS file: {e}");
                e
            })?;
    } else {
        info!("Using default CSS file.");

        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    write_default_css_file(&cli.output_dir).unwrap_or_else(|e| {
                        error!("Failed to write default CSS file: {e}");
                    });
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool for using default CSS: {e}");
                e
            })?;
    }

    let favicon_path = &config.html.favicon_file;
    if !favicon_path.is_empty() {
        info!("Copying favicon from: {}", favicon_path);
        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    copy_favicon_to_output_dir(favicon_path, &cli.output_dir).unwrap_or_else(|e| {
                        error!("Failed to copy favicon: {e}");
                    });
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool for favicon copy: {e}");
                e
            })?;
    } else {
        info!("No favicon specified in config.");
    }

    thread_pool.join_all();

    Ok(())
}

fn generate_static_site(cli: Arc<Cli>, file_path: &str, file_contents: &str) -> Result<(), Error> {
    // Tokenizing
    let mut tokenized_lines: Vec<Vec<Token>> = Vec::new();
    for line in file_contents.split('\n') {
        tokenized_lines.push(tokenize(line));
    }

    // Parsing
    let blocks = group_lines_to_blocks(tokenized_lines);
    let parsed_elements = parse_blocks(&blocks);

    // HTML Generation
    let generated_html = generate_html(
        file_path,
        &parsed_elements,
        &cli.output_dir,
        &cli.input_dir,
        file_path,
    );

    let html_relative_path = if file_path.ends_with(".md") {
        file_path.trim_end_matches(".md").to_string() + ".html"
    } else {
        file_path.to_string() + ".html"
    };

    let output_path = Path::new(&cli.output_dir).join(&html_relative_path);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    write_html_to_file(&generated_html, &cli.output_dir, &html_relative_path)?;

    Ok(())
}
