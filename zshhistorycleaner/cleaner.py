import os
import re
import time
from shutil import copy2
from typing import Optional, Match, List

# Regex to parse an entry in the history file
ZSH_HISTORY_ENTRY_REGEX = r": (?P<beginning_time>\d{10}):(?P<elapsed_seconds>\d);(?P<command>.*)"
ZSH_COMPILED_REGEX = re.compile(ZSH_HISTORY_ENTRY_REGEX)


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


class ZshHistoryCleaner:
    """
    Clean history file
    """

    def __init__(self, history_file_path):
        self.history_file_path = history_file_path
        self.history_entries = self._get_entries()

    def remove_duplicates(self) -> List[ZshHistoryEntry]:
        """
        Remove duplicate commands.
        :return: A list containg the entries whith duplicates
        """
        return sorted(list(set(self.history_entries)), key=lambda ent: ent.beginning_time)

    def _get_entries(self) -> List[ZshHistoryEntry]:
        """
        Get the entries from
        :return:
        """
        lines = self._read_history_file()
        entries = []
        for current_line in lines:
            match_object = parse_history_entry(current_line)
            if match_object:
                entry = ZshHistoryEntry(raw_line=current_line, beginning_time=int(match_object.group("beginning_time")),
                                        elapsed_seconds=int(match_object.group("elapsed_seconds")),
                                        command=match_object.group("command").strip())
                entries.append(entry)
            else:
                # TODO: warning message
                pass

        return entries

    def _read_history_file(self):
        """
        Read history file and yield the lines
        :return:
        """

        with open(self.history_file_path, mode="rb") as history:
            for line in history:
                yield line.decode('utf-8')


def parse_history_entry(line: str) -> Optional[Match]:
    """
    This function parse a line in the zsh_history file
    :param line: example ": 1556053755:0;printenv"
    :return: a matched object
    """

    return ZSH_COMPILED_REGEX.search(line)


def write_history(history_file_path, entries: List[ZshHistoryEntry], backup: bool = True):
    """

    :param history_file_path:
    :param entries:
    :param backup: is backup true, the history is backup with ${history_filename}.${timestamp}
    :return:
    """
    if backup:
        backup_file_path = f"{history_file_path}.{int(time.time())}"
        print(f"Backing up '{history_file_path}' to '{backup_file_path}'")
        copy2(history_file_path, backup_file_path)

    print("Writing history...")
    # sort the entries based on the timestamp
    sorted_entries = sorted(entries, key=lambda ent: ent.beginning_time)
    with open(history_file, "w") as f:
        for e in sorted_entries:
            f.write(e.raw_line)


if __name__ == '__main__':
    history_file = os.path.expanduser("~/.zsh_history")
    cleaner = ZshHistoryCleaner(history_file)
    no_dups = cleaner.remove_duplicates()

    print(f"{len(cleaner.history_entries)} command(s) parsed successfully from the history file")
    print("Checking duplicate commands...")
    print(f"{len(cleaner.history_entries) - len(no_dups)} command(s) will be removed from the history")
    write_history(history_file, no_dups)
