import argparse
import os
import sys

from zshhistorycleaner import __prog__
from zshhistorycleaner.history import ZshHistory
from zshhistorycleaner.history import logger




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
            history.save(backup=parsed_args.no_backup)


if __name__ == '__main__':
    main(sys.argv)
