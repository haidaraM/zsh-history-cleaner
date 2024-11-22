# Zsh History Cleaner

A command line tool to clean your .zsh history by:

- Removing duplicate commands: the first command is kept among the duplicates.
- Removing commands from a specific time range (TODO)
- Removing commands matching some patterns (TODO)

> **Disclaimer:** I'm primarily using this project as an opportunity to learn the Rust programming langage (it's my
> first project in Rust). As such, do not expect the project to be a full-featured solution for cleaning your history
> file.

## Usage

By default, the duplicates command are removed from the history.

**Note that multilines command with backslash are not supported yet.**

```shell
Clean your history by removing duplicate commands, commands matching regex etc...

Usage: zhc [OPTIONS]

Options:
  -d, --dry-run
          Dry run mode. The history file is not modified

  -H, --history-file <HISTORY_FILE>
          History file path

          [default: ~/.zsh_history]

  -n, --no-backup
          Disable history file backup. By default, a backup is written to {history_file}.{timestamp} in the current directory. Use with caution!!

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Development and test

TODO
