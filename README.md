# Zsh History Cleaner

A command line tool to clean your .zsh history by:

- Removing duplicate commands: the first command is kept among the duplicates.
- Removing commands from a specific time range (TODO)
- Removing commands matching some patterns (TODO)

> [!WARNING]  
> **Disclaimer:** I'm primarily using this project as an opportunity to learn the Rust programming language (it's my
> first project in Rust). As such, do not expect the project to be a full-featured solution for cleaning your history
> file.

## Installation

TODO: Publish to crates.io and add installation instructions:

```shell
cargo install zsh-history-cleaner
zhc --help
```

## Usage

```
Clean your history by removing duplicate commands, commands matching regex, etc...

By default, all the duplicate commands are removed.

Usage: zhc [OPTIONS]

Options:
  -d, --dry-run
          Dry run mode. The history file is not modified when this flag is used

  -H, --history-file <HISTORY_FILE>
          The history file to use

          [default: ~/.zsh_history]

  -n, --no-backup
          [USE WITH CAUTION!!] Disable history file backup. By default, a backup is written to '{history_file}.{timestamp}' in the current directory

  -k, --keep-duplicates
          Should we keep duplicate commands in the history file?

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## Development and test

```shell
cargo build
cargo test
```

