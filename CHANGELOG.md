# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-06-22

### Added

- Initial release of `mkdir2`.
- Cross-platform path handling: `/` and `\` are interchangeable separators
  on every platform.
- Bash-compatible brace expansion (`{a,b,c}`, nested groups, numeric ranges
  with optional step and zero-padding, alpha ranges, cartesian products).
- Automatic parent directory creation by default, with `--no-parents` to
  opt into strict classic `mkdir` behavior.
- `-f, --force` to delete and recreate an existing target, with
  `--no-recursive-remove` and `-i, --interactive` safety controls.
- `-m, --mode` for setting Unix octal permissions on new directories.
- `--gitkeep` to drop a `.gitkeep` file in every newly created directory.
- `-v, --verbose`, `-q, --quiet`, `-n, --dry-run`, `--stats`, `--strict`.
- `--json` machine-readable report output with proper non-zero exit codes
  on failure.
- `--from FILE` for batch creation from a pattern file (or stdin via `-`).
- Themeable colored + emoji output, with `--color auto|always|never`,
  `--no-emoji`, custom `--color-success/error/warn/info` hex colors, and
  `NO_COLOR` environment variable support.

[Unreleased]: https://github.com/cumulus13/mkdir2/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cumulus13/mkdir2/releases/tag/v0.1.0
