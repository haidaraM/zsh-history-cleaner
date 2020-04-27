# ZSH History Cleaner
A command line tool to clean your .zsh history:
 - Remove duplicate commands: trailing space are removed when checking duplicates
 - Remove command matching some patterns (TODO)
 - Remove command from a specific time range (TODO)


## Usage
By default, the duplicates command are remove from the history file. **Note that multilines command with backslash are
note supported.**
```shell script
$ zhc --help
Clean your Zsh history by removing duplicates command etc...

optional arguments:
  -h, --help            show this help message and exit
  --no-backup           Disable history file backup. A backup is written to
                        {history-file-path}.{timestamp}
  -H HISTORY_FILE_PATH, --history-file-path HISTORY_FILE_PATH
                        History file path. Default to ~/.zsh_history
  -V, --version         Print version and exits

```

## Development and test
```shell script
$ git clone https://github.com/haidaraM/zsh-history-cleaner
$ pipenv install --dev
$ pipenv shell
$ cd tests && pytest 
``` 