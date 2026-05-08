mod config;
mod error;
mod html_generator;
mod io;
mod lexer;
mod parser;
mod thread_pool;
mod types;
mod utils;
mod watch;

use clap::{Parser, command};
use env_logger::Env;
use log::{error, info};
use std::path::{Path, PathBuf};
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
        help = "Open the generated index.html in the default web browser."
    )]
    open: bool,
    #[arg(short, long, default_value = "", num_args = 0.., help = "Exclude files or directories from the input directory. Can be specified multiple times, or as a space-separated list.")]
    exclude: Vec<String>,
    #[arg(
        short,
        long,
        default_value = "false",
        help = "Watch for changes and rebuild automatically."
    )]
    watch: bool,
    #[arg(
        long,
        default_value = "false",
        help = "Open the markdown file with syntax highlighting alongside its generated HTML."
    )]
    preview: bool,
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
    let cli = Arc::new(Cli::parse());

    // Setup logging once
    let env = if cli.verbose {
        Env::default().default_filter_or("info")
    } else {
        Env::default().default_filter_or("warn")
    };
    env_logger::Builder::from_env(env).init();

    // Create thread pool once
    let thread_pool = Arc::new(ThreadPool::build(cli.num_threads).map_err(|e| {
        error!("Failed to create thread pool: {e}");
        e
    })?);

    if cli.watch {
        let cli_clone = Arc::clone(&cli);
        let input_dir = cli.input_dir.clone();
        let output_dir = cli.output_dir.clone();
        let pool_clone = Arc::clone(&thread_pool);

        let cli_clone2 = Arc::clone(&cli);
        let input_dir_clone = cli.input_dir.clone();

        if let Err(e) = watch::start_watch_mode(
            input_dir,
            output_dir,
            // Full build callback (for initial build)
            move || match build(Arc::clone(&cli_clone), &pool_clone) {
                Ok(_) => Ok(()),
                Err(e) => Err(Box::new(e)),
            },
            // Incremental build callback (for file changes)
            move |changed_paths: &[PathBuf]| {
                let cli = Arc::clone(&cli_clone2);
                let input_base = Path::new(&input_dir_clone);
                let mut needs_index_rebuild = false;
                let mut processed_files: Vec<String> = Vec::new();

                for changed_path in changed_paths {
                    // Only process markdown files
                    if !changed_path.extension().is_some_and(|ext| ext == "md") {
                        info!("Skipping non-markdown file: {:?}", changed_path);
                        continue;
                    }

                    // Read the file content
                    let file_content = match std::fs::read_to_string(changed_path) {
                        Ok(content) => content,
                        Err(e) => {
                            error!("Failed to read file {:?}: {}", changed_path, e);
                            continue;
                        }
                    };

                    // Get relative path from input directory
                    let relative_path = match changed_path.strip_prefix(input_base) {
                        Ok(rel) => rel.to_string_lossy().to_string(),
                        Err(_) => {
                            // Fallback to filename if strip_prefix fails
                            changed_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        }
                    };

                    // Check if this is a new file (output HTML doesn't exist yet)
                    let html_relative = if relative_path.ends_with(".md") {
                        relative_path.trim_end_matches(".md").to_string() + ".html"
                    } else {
                        relative_path.clone() + ".html"
                    };
                    let output_path = Path::new(&cli.output_dir).join(&html_relative);
                    if !output_path.exists() {
                        info!("New file detected: {}", relative_path);
                        needs_index_rebuild = true;
                    }

                    processed_files.push(relative_path.clone());
                    info!("Rebuilding: {}", relative_path);

                    if let Err(e) =
                        generate_static_site(Arc::clone(&cli), &relative_path, &file_content)
                    {
                        error!("Failed to rebuild {:?}: {}", changed_path, e);
                    }
                }

                // Regenerate index if new files were added
                if needs_index_rebuild {
                    info!("Regenerating index.html for new files...");
                    // Get all markdown files in input directory
                    if let Ok(all_files) =
                        read_input_dir(&cli.input_dir, &cli.recursive, &cli.exclude)
                    {
                        let file_names: Vec<String> =
                            all_files.iter().map(|(name, _)| name.clone()).collect();
                        let index_html = generate_index(&file_names, cli.watch);
                        if let Err(e) =
                            write_html_to_file(&index_html, &cli.output_dir, "index.html")
                        {
                            error!("Failed to update index.html: {}", e);
                        } else {
                            info!("Index updated with {} files.", file_names.len());
                        }
                    }
                }

                Ok(())
            },
        ) {
            error!("Watch mode failed: {}", e);
            std::process::exit(1);
        }
    } else {
        build(cli, &thread_pool)?;
        // Wait for all threads to finish in non-watch mode
        // Note: join_all consumes the pool. We need to unwrap the Arc.
        if let Ok(pool) = Arc::try_unwrap(thread_pool) {
            pool.join_all();
        } else {
            error!("Failed to unwrap thread pool for cleanup");
        }
    }

    Ok(())
}

fn build(cli: Arc<Cli>, thread_pool: &ThreadPool) -> Result<(), Error> {
    let input_dir = &cli.input_dir;
    let config_path = &cli.config;
    let run_recursively = &cli.recursive;

    // Use try_init_config to avoid double initialization error in watch mode
    if CONFIG.get().is_none() {
        init_config(config_path)?;
    }

    let config = CONFIG.get().unwrap();
    let file_contents = read_input_dir(input_dir, run_recursively, &cli.exclude)?;
    let mut file_names: Vec<String> = Vec::with_capacity(file_contents.len());

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&cli.output_dir) {
        error!(
            "Failed to create output directory {}: {}",
            cli.output_dir, e
        );
        return Err(e.into());
    }

    // Synchronization channel
    let (tx, rx) = std::sync::mpsc::channel();
    let mut task_count = 0;

    for (file_path, file_content) in file_contents {
        info!("Generating HTML for file: {}", file_path);

        file_names.push(file_path.clone());

        let tx = tx.clone();
        task_count += 1;

        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    generate_static_site(cli, &file_path, &file_content).unwrap_or_else(|e| {
                        error!("Failed to generate HTML for {file_path}: {e}");
                    });
                    let _ = tx.send(());
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool: {e}");
                e
            })?;
    }

    // Index Generation
    let tx_index = tx.clone();
    task_count += 1;
    thread_pool
        .execute({
            let cli = Arc::clone(&cli);
            move || {
                let index_html = generate_index(&file_names, cli.watch);
                write_html_to_file(&index_html, &cli.output_dir, "index.html").unwrap_or_else(
                    |e| {
                        error!("Failed to write index.html: {e}");
                    },
                );
                let _ = tx_index.send(());
            }
        })
        .map_err(|e| {
            error!("Failed to execute job in thread pool for index generation: {e}");
            e
        })?;

    let css_file = &config.html.css_file;
    let tx_css = tx.clone();
    task_count += 1;
    if css_file != "default" && !css_file.is_empty() {
        info!("Using custom CSS file: {}", css_file);
        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    copy_css_to_output_dir(css_file, &cli.output_dir).unwrap_or_else(|e| {
                        error!("Failed to copy CSS file: {e}");
                    });
                    let _ = tx_css.send(());
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
                    let _ = tx_css.send(());
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
        let tx_fav = tx.clone();
        task_count += 1;
        thread_pool
            .execute({
                let cli = Arc::clone(&cli);
                move || {
                    copy_favicon_to_output_dir(favicon_path, &cli.output_dir).unwrap_or_else(|e| {
                        error!("Failed to copy favicon: {e}");
                    });
                    let _ = tx_fav.send(());
                }
            })
            .map_err(|e| {
                error!("Failed to execute job in thread pool for favicon copy: {e}");
                e
            })?;
    } else {
        info!("No favicon specified in config.");
    }

    // Drop original sender to prevent hanging
    drop(tx);

    // Wait for all tasks to complete
    for _ in 0..task_count {
        if let Err(e) = rx.recv() {
            error!("Failed to receive completion signal: {}", e);
        }
    }

    // In watch mode, we rely on the server to serve files, so we might not need to open the browser via cli.
    // But the user might still want to open it initially.
    if cli.open && !cli.watch {
        let index_path = Path::new(&cli.output_dir).join("index.html");
        if index_path.exists() {
            if let Err(e) = webbrowser::open(&index_path.to_string_lossy()) {
                error!("Failed to open index.html in browser: {e}");
            } else {
                info!("Opened index.html in browser.");
            }
        } else {
            error!(
                "index.html does not exist at path: {}",
                index_path.display()
            );
        }
    }
    // If watching, the watch loop might print the URL.

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
        cli.watch,
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
