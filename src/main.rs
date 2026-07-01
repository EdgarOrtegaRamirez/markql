use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io::{self, Read};

use markql::{format_results_str, output::OutputFormat, parse, query, query_markdown};

#[derive(Parser)]
#[command(
    name = "markql",
    about = "Markdown Query Language - Query Markdown documents with CSS-like selectors",
    version,
    long_about = "MarkQL is a command-line tool for querying Markdown documents.\n\n\
                   It parses Markdown into an AST and allows you to query it using\n\
                   CSS-like selectors. Find headings, code blocks, links, and more\n\
                   with powerful filter expressions."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query a markdown file and output results
    Query {
        /// The query selector (e.g., 'heading[level=2]', 'code_block[lang=python]')
        query: String,

        /// Input file (defaults to stdin)
        #[arg(short, long)]
        file: Option<String>,

        /// Output format: json, markdown, text, count, tree
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Parse markdown and output the full AST
    Ast {
        /// Input file (defaults to stdin)
        #[arg(short, long)]
        file: Option<String>,

        /// Output format: json, tree
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Show statistics about a markdown document
    Stats {
        /// Input file (defaults to stdin)
        #[arg(short, long)]
        file: Option<String>,
    },

    /// List all available node types
    Types,

    /// Run a series of example queries
    Demo,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            query: query_str,
            file,
            format: fmt,
        } => {
            let input = read_input(file.as_deref())?;
            let ast = parse(&input);
            let results = query(&ast, &query_str)
                .map_err(|e| anyhow::anyhow!("Invalid query '{}': {}", query_str, e))?;
            let output = format_results_str(&results, &fmt, &query_str);
            println!("{}", output);
        }
        Commands::Ast { file, format: fmt } => {
            let input = read_input(file.as_deref())?;
            let ast = parse(&input);

            match fmt.as_str() {
                "json" => {
                    let json = markql::output::format_results(
                        &[markql::query::executor::MatchResult {
                            node: ast,
                            path: vec![],
                            depth: 0,
                        }],
                        &OutputFormat::Json,
                        "*",
                    );
                    println!("{}", json);
                }
                "tree" => {
                    print_ast_tree(&ast, "", true);
                }
                _ => {
                    eprintln!("Unknown format: {}. Use 'json' or 'tree'.", fmt);
                    std::process::exit(1);
                }
            }
        }
        Commands::Stats { file } => {
            let input = read_input(file.as_deref())?;
            let ast = parse(&input);
            print_stats(&ast);
        }
        Commands::Types => {
            print_types();
        }
        Commands::Demo => {
            run_demo();
        }
    }

    Ok(())
}

fn read_input(file: Option<&str>) -> Result<String> {
    match file {
        Some(path) => {
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))
        }
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .context("Failed to read from stdin")?;
            Ok(input)
        }
    }
}

fn print_ast_tree(node: &markql::ast::Node, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last { "    " } else { "│   " };

    let type_name = node.type_name();
    let text = node.text_content();
    let display = if text.is_empty() {
        type_name.to_string()
    } else {
        let truncated = if text.len() > 40 {
            format!("{}...", &text[..37])
        } else {
            text
        };
        format!("{}: {}", type_name, truncated)
    };

    println!("{}{}{}", prefix, connector, display.cyan());

    let children = node.children();
    for (i, child) in children.iter().enumerate() {
        let new_prefix = format!("{}{}", prefix, child_prefix);
        print_ast_tree(child, &new_prefix, i == children.len() - 1);
    }
}

fn print_stats(node: &markql::ast::Node) {
    let mut counts = std::collections::HashMap::new();
    count_nodes(node, &mut counts);

    println!("{}", "Markdown Document Statistics".bold().underline());
    println!();

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.1));

    for (node_type, count) in &sorted {
        let bar = "█".repeat(*count.min(&50));
        println!("  {:<20} {:>5}  {}", node_type.green(), count, bar.blue());
    }

    println!();
    let total: usize = sorted.iter().map(|(_, c)| c).sum();
    println!("  Total nodes: {}", total.to_string().yellow());
}

fn count_nodes(node: &markql::ast::Node, counts: &mut std::collections::HashMap<String, usize>) {
    let type_name = node.type_name().to_string();
    *counts.entry(type_name).or_insert(0) += 1;
    for child in node.children() {
        count_nodes(child, counts);
    }
}

fn print_types() {
    println!("{}", "Available Node Types".bold().underline());
    println!();
    let types = vec![
        ("heading", "Section headings (# through ######)"),
        ("paragraph", "Block of text"),
        ("code_block", "Fenced code blocks (``` or ~~~)"),
        ("inline_code", "Inline code (`code`)"),
        ("blockquote", "Blockquotes (>)"),
        ("list", "Ordered or unordered lists"),
        ("list_item", "Individual list items"),
        ("link", "Links [text](url)"),
        ("image", "Images ![alt](url)"),
        ("emphasis", "Italic (*text*) or bold (**text**)"),
        ("thematic_break", "Horizontal rules (---, ***, ___)"),
        ("table", "Markdown tables"),
        ("table_row", "Table rows"),
        ("table_cell", "Table cells"),
        ("text", "Plain text content"),
        ("line_break", "Line breaks"),
    ];

    for (name, desc) in &types {
        println!("  {:<20} {}", name.cyan(), desc);
    }

    println!();
    println!("{}", "Filter Examples".bold().underline());
    println!();
    println!("  heading                   Match all headings");
    println!("  heading[level=2]          Match level-2 headings only");
    println!("  code_block[lang=python]   Match Python code blocks");
    println!("  link[url*=github]         Match links containing 'github'");
    println!(r#"  heading[contains("API")]  Match headings containing 'API'"#);
    println!("  list > list_item          Match direct list items");
    println!("  blockquote emphasis       Match bold/italic in blockquotes");
    println!("  heading, code_block       Match headings AND code blocks");
}

fn run_demo() {
    let demo_md = [
        "# Getting Started with MarkQL",
        "",
        "## Installation",
        "",
        "Install using cargo:",
        "",
        "```bash",
        "cargo install markql",
        "```",
        "",
        "## Usage",
        "",
        "Query markdown files with CSS-like selectors.",
        "",
        "## Features",
        "",
        "- **Fast parsing**: Built in Rust for speed",
        "- **CSS-like queries**: Familiar selector syntax",
        "- **Multiple outputs**: JSON, Markdown, Text, Count, Tree",
        "",
        "## Links",
        "",
        "- [GitHub](https://github.com/EdgarOrtegaRamirez/markql)",
        "- [Documentation](https://docs.rs/markql)",
        "",
        "> MarkQL makes it easy to find and extract specific parts of Markdown documents.",
        "",
        "---",
        "",
        "*Built with love in Rust*",
    ]
    .join("\n");

    println!("{}", "MarkQL Demo".bold().underline());
    println!();
    println!("{}", "Input Markdown:".yellow());
    println!("{}", demo_md.dimmed());
    println!();

    let queries = vec![
        ("heading", "All headings"),
        ("heading[level=1]", "Level 1 headings only"),
        ("heading[level=2]", "Level 2 headings only"),
        ("code_block", "All code blocks"),
        ("code_block[lang=bash]", "Bash code blocks"),
        ("code_block[lang=rust]", "Rust code blocks"),
        ("link", "All links"),
        ("emphasis", "All emphasis (bold/italic)"),
        ("blockquote", "All blockquotes"),
    ];

    for (q, desc) in &queries {
        println!("{}: {}", desc.green().bold(), q.cyan());
        match query_markdown(&demo_md, q) {
            Ok(results) => {
                for r in &results {
                    let text = r.node.text_content();
                    let truncated = if text.len() > 60 {
                        format!("{}...", &text[..57])
                    } else {
                        text
                    };
                    println!("  → {}", truncated);
                }
            }
            Err(e) => println!("  Error: {}", e.red()),
        }
        println!();
    }
}
