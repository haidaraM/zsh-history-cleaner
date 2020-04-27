import argparse
import os
import sys

from zshhistorycleaner import __version__, __prog__
from zshhistorycleaner.cleaner import ZshHistory
from zshhistorycleaner.cleaner import logger


def get_parser() -> argparse.ArgumentParser:
    main_parser = argparse.ArgumentParser(description="Clean your Zsh history by removing duplicates command etc...",
                                          prog=__prog__)

    main_parser.add_argument("--no-backup", action="store_false",
                             help="Disable history file backup. A backup is written to {history-file-path}.{timestamp}")

    main_parser.add_argument("-H", "--history-file-path", help="History file path. Default to ~/.zsh_history",
                             default="~/.zsh_history")

    main_parser.add_argument("-V", "--version", dest="version", action="version", version='%(prog)s ' + __version__,
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
    history_file_path = os.path.expanduser(parsed_args.history_file_path)

    history = ZshHistory(history_file_path)
    entries_number = len(history.entries)
    logger.info("Removing duplicate commands...")
    history.remove_duplicates()

    logger.info(f"{entries_number - len(history.entries)} command(s) will be removed from the history")

    if entries_number != len(history.entries):
        logger.info("Saving")
        history.save(backup=parsed_args.no_backup)


if __name__ == '__main__':
    main(sys.argv)
