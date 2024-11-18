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

```shell script
$ zhc --help
usage: zhc [-h] [--dry-run] [--no-backup] [-H HISTORY_FILE_PATH] [-v]

Clean your Zsh history by removing duplicates command etc...

optional arguments:
  -h, --help            show this help message and exit
  --dry-run             Dry run mode. The history file is not modified
  --no-backup           Disable history file backup. A backup is written to
                        {history-file-path}.{timestamp}
  -H HISTORY_FILE_PATH, --history-file-path HISTORY_FILE_PATH
                        History file path. Default to ~/.zsh_history
  -v, --version         Print version and exitss

```

## Development and test

TODO
