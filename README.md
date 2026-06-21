# mkdir2

[![CI](https://github.com/cumulus13/mkdir2/actions/workflows/ci.yml/badge.svg)](https://github.com/cumulus13/mkdir2/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/mkdir2.svg)](https://crates.io/crates/mkdir2)
[![docs.rs](https://img.shields.io/docsrs/mkdir2)](https://docs.rs/mkdir2)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A modern, cross-platform, production-ready replacement for `mkdir` — brace
expansion, mixed `/` and `\` path separators, safe force-recreate, colored +
emoji output, JSON reports, dry-run, and more.

```
$ mkdir2 -v project/{src,tests,docs}
✅ created: project/src
✅ created: project/tests
✅ created: project/docs
```

## Why

The classic `mkdir` is fine, but it hasn't changed in decades and it behaves
differently depending on the shell and platform you're using. `mkdir2` is a
single static binary that works the same way everywhere:

- **One path syntax, every OS.** `data\2024\reports` and `data/2024/reports`
  create the exact same tree, whether you're on Linux, macOS, or Windows.
- **Brace expansion**, like Bash, but built in — no shell required, so it
  works identically from PowerShell, cmd.exe, or any script runner.
- **Safe, explicit "force recreate"** instead of a fragile `rm -rf && mkdir`
  two-step.
- **Scriptable output**: `--json`, `--stats`, `--quiet`, and proper exit
  codes for use in CI and automation.
- **Themeable**: pick your own hex colors, or turn color/emoji off entirely
  for log files and dumb terminals.

## Install

### From crates.io

```sh
cargo install mkdir2
```

### From source

```sh
git clone https://github.com/cumulus13/mkdir2
cd mkdir2
cargo install --path .
```

### Prebuilt binaries

Prebuilt binaries for Linux (gnu/musl), macOS (x86_64/aarch64), and Windows
are attached to every [GitHub release](https://github.com/cumulus13/mkdir2/releases).

## Usage

```
mkdir2 [OPTIONS] [PATH]...
```

### Basic examples

```sh
# Plain creation, parents included by default (like mkdir -p)
mkdir2 build/output

# Brace expansion — creates three directories in one call
mkdir2 project/{src,tests,docs}

# Nested + cartesian expansion
mkdir2 build/{debug,release}/{bin,lib}

# Numeric ranges, with zero-padding preserved
mkdir2 file{01..10}

# Either separator works on every platform
mkdir2 "data\2024\reports"
mkdir2 "data/2024/reports"      # identical result

# Force: delete an existing tree first, then recreate it clean
mkdir2 --force old_output

# Preview without touching the filesystem
mkdir2 --dry-run new_stuff/{a,b,c}

# Batch-create from a file, one pattern per line
mkdir2 --from dirs.txt --gitkeep
```

> **Note for Bash/zsh users:** `\` is always treated as a path separator by
> mkdir2, the same as `/`. But on an interactive Unix shell, an *unquoted*
> backslash is consumed by the shell itself as an escape character before
> mkdir2 ever sees it — so `mkdir2 test\ me` does not create `test/me`; it
> creates a single directory named `test me` (the shell strips the `\` and
> keeps the space literal). Likewise `mkdir2 test\me` becomes a single
> directory `testme` (the shell deletes the `\` entirely). To actually use
> `\` as a separator from Bash/zsh, quote the argument so the shell leaves
> it alone: `mkdir2 'test\me'` creates nested `test/me` as expected. This
> only matters for interactive Unix shells — it's a non-issue on Windows
> shells, and on patterns read from a `--from` file, since neither goes
> through Unix shell escaping.

### All options

| Flag | Description |
|---|---|
| `[PATH]...` | Directories to create. Supports brace expansion and either `/` or `\` as a separator on any platform. |
| `--from <FILE>` | Read additional path patterns from a file, one per line (`-` for stdin). Blank lines and `#` comments are ignored. |
| `--no-parents` | Disable automatic parent creation (classic strict `mkdir` behavior). Parents are created by default. |
| `-f, --force` | Delete an existing target first, then recreate it. |
| `--no-recursive-remove` | With `--force`, only remove existing *empty* directories instead of recursive trees. |
| `-i, --interactive` | Confirm before deleting an existing path with `--force`. |
| `-m, --mode <MODE>` | Octal permissions for new directories, e.g. `0755` (Unix only; warns and no-ops on Windows). |
| `--gitkeep` | Create a `.gitkeep` file inside every newly created directory. |
| `-v, --verbose` | Print each action as it happens. |
| `-q, --quiet` | Suppress all non-error output. |
| `-n, --dry-run` | Show what would be done without changing the filesystem. |
| `--stats` | Print a created/skipped/failed summary at the end. |
| `--json` | Emit a machine-readable JSON report instead of text. |
| `--strict` | Treat an already-existing target as an error (matches plain `mkdir`'s strict behavior). |
| `--color <auto\|always\|never>` | Control colored output. Honors `NO_COLOR` and TTY detection in `auto` mode. |
| `--no-emoji` | Disable emoji icons in output. |
| `--color-success/error/warn/info <HEX>` | Customize each message type's color, e.g. `--color-info '#00FFFF'`. |

Run `mkdir2 --help` for the full reference.

### Brace expansion syntax

`mkdir2` implements Bash-compatible brace expansion:

| Pattern | Expands to |
|---|---|
| `{a,b,c}` | `a`, `b`, `c` |
| `{a,{b,c}}` | `a`, `b`, `c` (nested groups) |
| `{1..5}` | `1`, `2`, `3`, `4`, `5` |
| `{01..05}` | `01`, `02`, `03`, `04`, `05` (zero-padding preserved) |
| `{1..10..2}` | `1`, `3`, `5`, `7`, `9` (step) |
| `{a..e}` | `a`, `b`, `c`, `d`, `e` |
| `a{1,2}b{x,y}` | `a1bx`, `a1by`, `a2bx`, `a2by` (cartesian product) |

### JSON output

```sh
$ mkdir2 --json batch/{a,b,c}
{
  "created": 3,
  "already_existed": 0,
  "skipped": 0,
  "failed": 0,
  "results": [
    { "path": "batch/a", "action": "created", "message": null },
    { "path": "batch/b", "action": "created", "message": null },
    { "path": "batch/c", "action": "created", "message": null }
  ]
}
```

Exit code is non-zero if any target failed, even with `--json`, so it's
safe to check `$?` in scripts.

## mkdir2 vs. classic mkdir

| Feature | `mkdir` | `mkdir2` |
|---|---|---|
| Creates parent dirs | only with `-p` | by default (`--no-parents` to opt out) |
| Brace expansion | only via shell (Bash) | built in, every platform/shell |
| `/` and `\` both work | no | yes, on every OS |
| Safe recreate-if-exists | no | `--force` (with `--no-recursive-remove`, `-i`) |
| Colored / emoji output | no | yes, fully themeable hex colors |
| Machine-readable output | no | `--json` |
| Dry-run preview | no | `-n / --dry-run` |
| Batch creation from a file | no | `--from FILE` |

## Development

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Contributing

Issues and pull requests are welcome at
[github.com/cumulus13/mkdir2](https://github.com/cumulus13/mkdir2).

## License

MIT © [Hadi Cahyadi](mailto:cumulus13@gmail.com)

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)
