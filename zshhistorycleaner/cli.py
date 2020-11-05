import argparse
import os
import sys

from zshhistorycleaner import __version__, __prog__
from zshhistorycleaner.history import ZshHistory
from zshhistorycleaner.history import logger


def get_parser() -> argparse.ArgumentParser:
    main_parser = argparse.ArgumentParser(description="Clean your Zsh history by removing duplicates command etc...",
                                          prog=__prog__)

    main_parser.add_argument("--dry-run", action="store_true", help="Dry run mode. The history file is not modified")

    main_parser.add_argument("--no-backup", action="store_false",
                             help="Disable history file backup. A backup is written to {history-file-path}.{timestamp}")

    main_parser.add_argument("-H", "--history-file", help="History file path. Default to ~/.zsh_history",
                             default="~/.zsh_history")

    main_parser.add_argument("-v", "--version", dest="version", action="version", version='%(prog)s ' + __version__,
                             help="Print version and exits")

    return main_parser


def main(args=None):
    """
    CLI entry point
    :return:
    """
    args = args or sys.argv
    parser = get_parser()
    parsed_args = parser.parse_args(args[1:])

    history_file = os.path.expanduser(parsed_args.history_file)
    dry_run = parsed_args.dry_run
    history = ZshHistory(history_file)
    entries_number = len(history.entries)

    logger.info("Removing duplicate commands...")
    history.remove_duplicates()

    logger.info(f"{entries_number - len(history.entries)} command(s) will be removed from the history")

    if entries_number != len(history.entries):
        if not dry_run:
            logger.info("Saving")
            history.save(backup=parsed_args.no_backup)


if __name__ == '__main__':
    main(sys.argv)
