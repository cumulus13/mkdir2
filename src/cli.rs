//! Command-line interface definition.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mkdir2",
    version,
    author = "Hadi Cahyadi <cumulus13@gmail.com>",
    about = "A modern, cross-platform, production-ready replacement for mkdir.",
    long_about = "mkdir2 creates directories like the classic `mkdir`, but adds brace \
expansion, mixed path separators (`/` and `\\` work the same on every \
platform), safe force-recreate, colored/emoji output, JSON reporting, \
dry-run, and batch creation from a file.",
    after_help = "EXAMPLES:\n  \
mkdir2 project/{src,tests,docs}\n  \
mkdir2 -v build/{debug,release}/{bin,lib}\n  \
mkdir2 file{01..10}\n  \
mkdir2 --force old_output\n  \
mkdir2 --from dirs.txt --gitkeep\n  \
mkdir2 \"data\\\\2024\\\\reports\"   # works the same on Linux and Windows"
)]
pub struct Cli {
    /// Directories to create. Supports brace expansion such as
    /// `project/{src,tests,docs}` or `file{01..10}`, and either `/` or `\`
    /// as a separator on any platform.
    #[arg(value_name = "PATH")]
    pub paths: Vec<String>,

    /// Read additional path patterns from a file, one per line (use `-` to
    /// read from stdin). Blank lines and lines starting with `#` are
    /// ignored.
    #[arg(long, value_name = "FILE")]
    pub from: Option<String>,

    /// Do not automatically create missing parent directories (classic
    /// `mkdir` behavior). By default, parent directories are always
    /// created as needed.
    #[arg(long)]
    pub no_parents: bool,

    /// If a target already exists, delete it first and recreate it fresh.
    #[arg(short, long)]
    pub force: bool,

    /// When combined with --force, only remove existing *empty*
    /// directories instead of deleting non-empty trees recursively.
    #[arg(long)]
    pub no_recursive_remove: bool,

    /// Ask for confirmation before deleting an existing path with --force.
    #[arg(short, long)]
    pub interactive: bool,

    /// Set permissions on newly created directories (octal, e.g. 0755).
    /// Unix only; ignored with a warning on Windows.
    #[arg(short = 'm', long, value_name = "MODE")]
    pub mode: Option<String>,

    /// Create a `.gitkeep` file inside every newly created directory.
    #[arg(long)]
    pub gitkeep: bool,

    /// Print each action as it happens.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all non-error output.
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show what would be done without changing the filesystem.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Print a summary of created/skipped/failed counts at the end.
    #[arg(long)]
    pub stats: bool,

    /// Emit a machine-readable JSON report instead of human-readable text.
    #[arg(long)]
    pub json: bool,

    /// Treat an already-existing target as an error instead of a silent
    /// no-op (matches the strict semantics of plain `mkdir`).
    #[arg(long)]
    pub strict: bool,

    /// Control colored output: auto (default), always, or never.
    #[arg(long, value_name = "auto|always|never", default_value = "auto")]
    pub color: String,

    /// Disable emoji icons in output.
    #[arg(long)]
    pub no_emoji: bool,

    /// Hex color used for success messages, e.g. #00FF00.
    #[arg(long, value_name = "HEX")]
    pub color_success: Option<String>,

    /// Hex color used for error messages, e.g. #FF0000.
    #[arg(long, value_name = "HEX")]
    pub color_error: Option<String>,

    /// Hex color used for warning messages, e.g. #FFA500.
    #[arg(long, value_name = "HEX")]
    pub color_warn: Option<String>,

    /// Hex color used for informational messages, e.g. #00FFFF.
    #[arg(long, value_name = "HEX")]
    pub color_info: Option<String>,
}
