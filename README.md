# Zsh History Cleaner

A command line tool to clean your .zsh history by:

- Removing duplicate commands: the first command is kept among the duplicates.
- Removing commands matching some patterns (TODO)
- Removing commands from a specific time range (TODO)

> **Disclaimer:** I'm primarily using this project as an opportunity to learn the Rust programming langage (it's my
> first project in Rust).

## Usage

By default, the duplicates command are removed from the history.

**Note that multilines command with backslash are not supported yet.**

```shell
$ zhc --help
Clean your Zsh history.

Usage: zhc [OPTIONS]

Options:
  -d, --dry-run                      Dry run mode. The history file is not modified
  -H, --history-file <HISTORY_FILE>  History file path [default: ~/.zsh_history]
  -n, --no-backup                    Disable history file backup. A backup is written to {history-file-path}.{timestamp}. Use with caution!
  -h, --help                         Print help
  -V, --version                      Print version


```

## Development and test

TODO
