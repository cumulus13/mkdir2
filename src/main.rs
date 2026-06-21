mod brace;
mod cli;
mod color;
mod mkdir;
mod pathnorm;

use clap::Parser;
use cli::Cli;
use color::{should_use_color, ColorMode, Theme};
use std::process::ExitCode;
use std::str::FromStr;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let color_mode = match ColorMode::from_str(&cli.color) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut theme = Theme {
        enabled: should_use_color(color_mode),
        emoji: !cli.no_emoji,
        ..Theme::default()
    };

    if let Err(e) = apply_custom_colors(&mut theme, &cli) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    if cli.paths.is_empty() && cli.from.is_none() {
        eprintln!("error: no directories given (pass one or more PATHs, or use --from FILE)");
        eprintln!();
        eprintln!("Try 'mkdir2 --help' for more information.");
        return ExitCode::FAILURE;
    }

    let mode = match parse_mode(cli.mode.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let targets = match mkdir::collect_targets(&cli.paths, cli.from.as_deref()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if targets.is_empty() {
        eprintln!("error: no directories to create after expansion");
        return ExitCode::FAILURE;
    }

    let opts = mkdir::Options {
        create_parents: !cli.no_parents,
        force: cli.force,
        recursive_remove: !cli.no_recursive_remove,
        interactive: cli.interactive,
        mode,
        gitkeep: cli.gitkeep,
        dry_run: cli.dry_run,
        strict: cli.strict,
        verbose: cli.verbose,
        quiet: cli.quiet,
        json: cli.json,
    };

    let summary = mkdir::run(&targets, &opts, &theme);

    if cli.json {
        match serde_json::to_string_pretty(&summary) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("error: could not serialize JSON report: {e}"),
        }
    } else if cli.stats && !cli.quiet {
        print_stats(&summary, &theme);
    }

    if summary.failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn apply_custom_colors(theme: &mut Theme, cli: &Cli) -> Result<(), String> {
    if let Some(hex) = &cli.color_success {
        theme.success = color::parse_hex(hex)?;
    }
    if let Some(hex) = &cli.color_error {
        theme.error = color::parse_hex(hex)?;
    }
    if let Some(hex) = &cli.color_warn {
        theme.warn = color::parse_hex(hex)?;
    }
    if let Some(hex) = &cli.color_info {
        theme.info = color::parse_hex(hex)?;
    }
    Ok(())
}

fn parse_mode(raw: Option<&str>) -> Result<Option<u32>, String> {
    match raw {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim_start_matches("0o");
            u32::from_str_radix(trimmed, 8)
                .map(Some)
                .map_err(|_| format!("invalid permission mode '{s}', expected octal like 0755"))
        }
    }
}

fn print_stats(summary: &mkdir::Summary, theme: &Theme) {
    println!();
    println!("{}", theme.info("Summary"));
    println!("  created:          {}", summary.created);
    println!("  already existed:  {}", summary.already_existed);
    println!("  skipped:          {}", summary.skipped);
    println!("  failed:           {}", summary.failed);
}
