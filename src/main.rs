mod config;
mod html_generator;
mod io;
mod lexer;
mod parser;
mod types;
mod utils;

use clap::{Parser, command};
use env_logger::Env;
use log::{error, info};
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use crate::config::{Config, init_config};
use crate::html_generator::{generate_html, generate_index};
use crate::io::{
    copy_css_to_output_dir, copy_favicon_to_output_dir, read_input_dir, write_default_css_file,
    write_html_to_file,
};
use crate::lexer::tokenize;
use crate::parser::{group_lines_to_blocks, parse_blocks};
use crate::types::{ThreadPool, Token};

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Parser, Debug, Clone)]
#[command(
    author = "Zackary Liel",
    version = "1.3.2",
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
}

fn main() -> Result<(), Box<dyn Error>> {
    match run() {
        Ok(_) => {
            info!("Static site generation completed successfully.");
            Ok(())
        }
        Err(e) => {
            error!("An error occurred: {}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let input_dir = &cli.input_dir;
    let config_path = &cli.config;
    let run_recursively = &cli.recursive;
    let num_threads = cli.num_threads;
    let output_dir = &cli.output_dir.clone();

    // Setup
    let env = if cli.verbose {
        Env::default().default_filter_or("info")
    } else {
        Env::default().default_filter_or("warn")
    };
    env_logger::Builder::from_env(env).init();

    init_config(config_path)?;
    let file_contents = read_input_dir(input_dir, run_recursively)?;
    let mut file_names: Vec<String> = Vec::new();

    let thread_pool = ThreadPool::build(num_threads)?;
    let cli = Arc::new(Mutex::new(cli));
    for (file_path, file_content) in file_contents {
        info!("Generating HTML for file: {}", file_path);
        file_names.push(file_path.clone());
        let cli_clone = Arc::clone(&cli);

        thread_pool.execute(move || {
            generate_static_site(cli_clone, &file_path.clone(), &file_content)
                .unwrap_or_else(|e| error!("Failed to generate HTML for {}: {}", &file_path, e));
        });
    }

    thread_pool.join_all();

    let index_html = generate_index(&file_names);
    write_html_to_file(&index_html, output_dir, "index.html")?;

    let css_file = CONFIG.get().unwrap().html.css_file.clone();
    if css_file != "default" && !css_file.is_empty() {
        info!("Using custom CSS file: {}", css_file);
        copy_css_to_output_dir(&css_file, &cli.lock().unwrap().output_dir)?;
    } else {
        info!("Using default CSS file.");
        write_default_css_file(&cli.lock().unwrap().output_dir)?;
    }

    let favicon_path = CONFIG.get().unwrap().html.favicon_file.clone();
    if !favicon_path.is_empty() {
        info!("Copying favicon from: {}", favicon_path);
        copy_favicon_to_output_dir(&favicon_path, &cli.lock().unwrap().output_dir)?;
    } else {
        info!("No favicon specified in config.");
    }

    Ok(())
}

fn generate_static_site(
    cli: Arc<Mutex<Cli>>,
    file_path: &String,
    file_contents: &str,
) -> Result<(), Box<dyn Error>> {
    // Tokenizing
    let cli = cli.lock().unwrap().clone();
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
