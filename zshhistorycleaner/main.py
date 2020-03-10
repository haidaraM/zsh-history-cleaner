import argparse
import os
import re
import time
from shutil import copy2
from typing import Optional, Match, List

command_regex = r": (?P<beginning_time>\d{10}):(?P<elapsed_seconds>\d);(?P<command>.*)"
compiled_regex = re.compile(command_regex)


class ZshHistoryEntry:
    """
    A zsh history entry
    """

    def __init__(self, raw_line: str, beginning_time: int, elapsed_seconds: int, command: str, line_number: int):
        """

        :param raw_line: the raw line from the history file
        :param beginning_time: timestamp in second for when the command was typed
        :param elapsed_seconds: duration of the command
        :param command: the actual command
        :param line_number: line number of the command in the history file (1-based)
        """
        self.command = command
        self.elapsed_seconds = elapsed_seconds
        self.beginning_time = beginning_time
        self.raw_line = raw_line
        self.line_number = line_number

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


def remove_duplicate_entries(entries: List[ZshHistoryEntry]) -> List[ZshHistoryEntry]:
    """
    Remove the duplicate commands from the entries
    :return:
    """
    return list(set(entries))


def get_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="")

    return parser


def read_history(history_file_path: str = "~/.zsh_history"):
    """
    Read history file and yield the lines
    :param history_file_path:
    :return:
    """

    with open(history_file_path, mode="rb") as history:
        for line in history:
            yield line.decode('utf-8')


def parse_zsh_command(line: str) -> Optional[Match]:
    """
    This function parse a line in the zsh_history file
    :param line: example ": 1556053755:0;printenv"
    :return: a matched object
    """

    return compiled_regex.search(line)


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
    sorted_entries = sorted(entries, key=lambda e: e.beginning_time)
    with open(history_file, "w") as f:
        for e in sorted_entries:
            f.write(e.raw_line)


if __name__ == '__main__':
    history_file = os.path.expanduser("~/.zsh_history")
    lines = list(read_history(history_file))

    command_entries = []
    for counter, current_line in enumerate(lines):
        match_object = parse_zsh_command(current_line)
        if match_object:
            entry = ZshHistoryEntry(raw_line=current_line, beginning_time=int(match_object.group("beginning_time")),
                                    elapsed_seconds=int(match_object.group("elapsed_seconds")),
                                    command=match_object.group("command"), line_number=counter + 1)
            command_entries.append(entry)

    print(f"{len(command_entries)} command(s) parsed successfully from the history file")
    print("Checking duplicate commands...")
    commands_without_dup = remove_duplicate_entries(command_entries)
    print(f"{len(command_entries) - len(commands_without_dup)} command(s) will be removed from the history")
    write_history(history_file, commands_without_dup)
