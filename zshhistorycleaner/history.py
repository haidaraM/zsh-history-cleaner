import logging
import os
import re
import time
from shutil import copy2
from typing import List

# Regex to parse an entry in the history file
from zshhistorycleaner.exceptions import HistoryEntryParserError

ZSH_HISTORY_ENTRY_REGEX = r": (?P<beginning_time>\d{10}):(?P<elapsed_seconds>\d);(?P<command>.*)"
ZSH_COMPILED_REGEX = re.compile(ZSH_HISTORY_ENTRY_REGEX)

logging.basicConfig(format='%(levelname)s: %(message)s')
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)


class ZshHistoryEntry:
    """
    A zsh history entry
    """

    def __init__(self, raw_line: str, beginning_time: int, elapsed_seconds: int, command: str):
        """

        :param raw_line: the raw line from the history file
        :param beginning_time: timestamp in second for when the command was typed
        :param elapsed_seconds: duration of the command
        :param command: the actual command
        """
        self.raw_line = raw_line
        self.command = command
        self.elapsed_seconds = elapsed_seconds
        self.beginning_time = beginning_time

    def __repr__(self):
        """

        :return:
        """
        return f"{self.__class__.__name__}('{self.command}')"

    def __hash__(self):
        """
        Hash used to remove duplicates
        :return:
        """
        return hash(self.command)

    def __eq__(self, other):
        """
        Two history entries are equal if the command are equal
        :param other:
        :return:
        """
        return isinstance(other, self.__class__) and self.command == other.command


class ZshHistory:
    """
    A ZSH history file
    """

    def __init__(self, history_file_path: str = "~/.zsh_history"):
        """

        :param history_file_path:
        """
        self.history_file_path = os.path.expanduser(history_file_path)
        self.entries = self._get_entries()
        logger.info(f"{len(self.entries)} history entries read from {self.history_file_path}")

    def save(self, backup: bool = True, output_file_path=None):
        """
        Save the entries in
        :param backup: if True, a backup will be created to {.history_file_path}.{timestamp}
        :param output_file_path: the file in which save the history. Default to self.history_file_path
        :return:
        """
        if backup:
            backup_file_path = f"{self.history_file_path}.{int(time.time())}"
            logger.info(f"Backing up '{self.history_file_path}' to '{backup_file_path}'")
            copy2(self.history_file_path, backup_file_path)
        output_file_path = output_file_path or self.history_file_path

        with open(output_file_path, "w") as f:
            for e in self.entries:
                f.write(e.raw_line)
                f.write("\n")

    def remove_duplicates(self):
        """
        Remove duplicate commands.
        :return:
        """
        self.entries = sorted(list(set(self.entries)), key=lambda ent: ent.beginning_time)

    def _get_entries(self) -> List[ZshHistoryEntry]:
        """
        Get the entries from
        :return:
        """
        lines = self._read_history_file()
        line_counter = 1
        entries = []
        for current_line in lines:
            try:
                current_line = current_line.strip()
                current_line = current_line.decode("utf-8")
                if len(current_line) > 0:
                    entries.append(parse_history_entry(current_line))
            except HistoryEntryParserError as parser_error:
                logger.warning(f"Impossible to parse the line {line_counter}: '{current_line}'")
            except UnicodeError:
                # Fixme: simply ignore this error for the moment. Check what we can do later on.
                pass

            line_counter += 1

        return entries

    def _read_history_file(self):
        """
        Read history file and yield the lines
        :return:
        """
        with open(self.history_file_path, mode="rb") as history:
            for line in history:
                yield line


def parse_history_entry(line: str) -> ZshHistoryEntry:
    """
    This function parses a line in the zsh_history file
    :param line: example ": 1556053755:0;printenv"
    :return: a ZshHistoryEntry
    """
    match_object = ZSH_COMPILED_REGEX.search(line)
    if match_object:
        return ZshHistoryEntry(raw_line=line,
                               beginning_time=int(match_object.group("beginning_time")),
                               elapsed_seconds=int(match_object.group("elapsed_seconds")),
                               command=match_object.group("command").strip())

    raise HistoryEntryParserError(f"The line '{line}' doesn't match the regex {ZSH_HISTORY_ENTRY_REGEX}.")
