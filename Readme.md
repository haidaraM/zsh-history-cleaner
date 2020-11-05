# ZSH History Cleaner

![Test and Lint](https://github.com/haidaraM/zsh-history-cleaner/workflows/Test%20and%20Lint/badge.svg)

A command line tool to clean your .zsh history:
 - Remove duplicate commands
 - Remove commands matching some patterns (TODO)
 - Remove commands from a specific time range (TODO)


## Usage
By default, the duplicates command are remove from the history file. **Note that multilines command with backslash are
note supported.**
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
```shell script
$ git clone https://github.com/haidaraM/zsh-history-cleaner
$ pipenv install --dev
$ pipenv shell
$ cd tests && pytest 
``` 