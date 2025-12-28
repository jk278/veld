//! Veld RSX Formatter - Configurable RSX formatting tool
//!
//! Usage:
//!   cargo run --bin veld-fmt -- [OPTIONS] [FILES]
//!   veld-fmt --check src/**/*.rs
//!   veld-fmt --indent-width 2 src/main.rs

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

// CLI argument parsing
struct Args {
  files: Vec<PathBuf>,
  check: bool,
  indent_width: Option<usize>,
  use_tabs: bool,
  help: bool,
}

impl Args {
  fn parse() -> Self {
    let args: Vec<String> = std::env::args().collect();
    let mut files = Vec::new();
    let mut check = false;
    let mut indent_width = None;
    let mut use_tabs = false;
    let mut help = false;

    let mut i = 1;
    while i < args.len() {
      match args[i].as_str() {
        "--check" | "-c" => check = true,
        "--tabs" | "-t" => use_tabs = true,
        "--help" | "-h" => help = true,
        "--indent-width" | "-w" => {
          if i + 1 < args.len() {
            indent_width = args[i + 1].parse().ok();
            i += 1;
          }
        }
        arg if arg.starts_with("--indent-width=") => {
          let value = arg.strip_prefix("--indent-width=").unwrap();
          indent_width = value.parse().ok();
        }
        arg if arg.starts_with('-') => {
          eprintln!("Unknown option: {}", arg);
          help = true;
        }
        arg => files.push(PathBuf::from(arg)),
      }
      i += 1;
    }

    if files.is_empty() {
      files = collect_rs_files(".");
    }

    Args {
      files,
      check,
      indent_width,
      use_tabs,
      help,
    }
  }

  fn indent_width(&self) -> usize {
    self.indent_width.unwrap_or(2)
  }
}

fn collect_rs_files(dir: &str) -> Vec<PathBuf> {
  let mut files = Vec::new();
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        let path_str = path.to_string_lossy();
        if !path_str.contains("target") && !path_str.contains(".git") {
          files.extend(collect_rs_files(path.to_str().unwrap()));
        }
      } else if path.extension().map_or(false, |ext| ext == "rs") {
        files.push(path);
      }
    }
  }
  files
}

fn print_help() {
  println!("Veld RSX Formatter - Configurable RSX formatting tool");
  println!();
  println!("Usage:");
  println!("  veld-fmt [OPTIONS] [FILES]...");
  println!("  cargo run --bin veld-fmt -- [OPTIONS] [FILES]...");
  println!();
  println!("Options:");
  println!("  -c, --check              Check if files are formatted (no modifications)");
  println!("  -w, --indent-width N     Set indent width (default: 2)");
  println!("  -t, --tabs               Use tabs instead of spaces");
  println!("  -h, --help               Print this help");
  println!();
  println!("Examples:");
  println!("  veld-fmt src/main.rs                    Format single file");
  println!("  veld-fmt --indent-width 4 src/          Use 4 spaces");
  println!("  veld-fmt --tabs src/                   Use tabs");
  println!("  veld-fmt --check                       Check formatting");
}

fn main() {
  let args = Args::parse();

  if args.help {
    print_help();
    return;
  }

  if args.files.is_empty() {
    println!("No Rust files found to format.");
    return;
  }

  println!("Formatting {} file(s)...", args.files.len());
  println!(
    "  Indent: {} {}",
    args.indent_width(),
    if args.use_tabs { "tabs" } else { "spaces" }
  );
  println!();

  let mut formatted = 0;
  let mut already_formatted = 0;
  let mut failed = 0;

  for file in &args.files {
    match format_file(file, args.indent_width(), args.use_tabs, args.check) {
      Ok(true) => {
        formatted += 1;
        println!("✓ {}", file.display());
      }
      Ok(false) => {
        already_formatted += 1;
      }
      Err(e) => {
        failed += 1;
        eprintln!("✗ {}: {}", file.display(), e);
      }
    }
  }

  println!();
  println!("Summary:");
  println!("  Formatted: {}", formatted);
  println!("  Already formatted: {}", already_formatted);
  if failed > 0 {
    println!("  Failed: {}", failed);
    process::exit(1);
  } else if args.check && formatted > 0 {
    println!();
    println!("Some files need formatting!");
    process::exit(1);
  }
}

fn format_file(
  path: &Path,
  indent_width: usize,
  use_tabs: bool,
  check_only: bool,
) -> Result<bool, String> {
  use dioxus_autofmt::{apply_formats, try_fmt_file, IndentOptions, IndentType};

  // Read file content
  let content = fs::read_to_string(path).map_err(|e| format!("Failed to read: {}", e))?;

  // Check if file contains RSX macro
  if !content.contains("rsx!") {
    return Ok(false);
  }

  // Parse the file as Rust code
  let parsed = syn::parse_file(&content)
    .map_err(|e| format!("Failed to parse Rust file: {}", e))?;

  // Format using dioxus-autofmt with custom options
  let indent_type = if use_tabs {
    IndentType::Tabs
  } else {
    IndentType::Spaces
  };
  let opts = IndentOptions::new(indent_type, indent_width, true);

  let blocks = try_fmt_file(&content, &parsed, opts)
    .map_err(|e| format!("Failed to format RSX: {}", e))?;

  if blocks.is_empty() {
    return Ok(false);
  }

  if check_only {
    return Ok(true);
  }

  // Apply the formatted blocks to get the final content
  let formatted = apply_formats(&content, blocks);

  // Write formatted content back
  fs::write(path, formatted).map_err(|e| format!("Failed to write: {}", e))?;

  Ok(true)
}
