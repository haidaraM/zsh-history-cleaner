import pytest

from zshhistorycleaner.history import parse_history_entry
from zshhistorycleaner.exceptions import HistoryEntryParserError


def test_empty_command():
    """
    An empty command should return an empty command
    :return:
    """
    with pytest.raises(HistoryEntryParserError) as ctx:
        parse_history_entry("")


def test_no_timestamp():
    """
    A command without timestamp should not fail
    :return:
    """
    with pytest.raises(HistoryEntryParserError) as ctx:
        parse_history_entry("ls;")


def test_simple_command():
    """
    Test a simple command
    :return:
    """
    command = ": 1556053755:2;printenv"
    parsed_command = parse_history_entry(command)
    assert parsed_command is not None
    assert parsed_command.command == "printenv"
    assert parsed_command.beginning_time == 1556053755
    assert parsed_command.elapsed_seconds == 2


def test_complex_command():
    """
    Test a complex command with special characters
    :return:
    """
    command = ": 1557138761:0;for d in VWT.*; do l $d; done"
    parsed_command = parse_history_entry(command)
    assert parsed_command is not None
    assert parsed_command.command == "for d in VWT.*; do l $d; done"
    assert parsed_command.beginning_time == 1557138761
    assert parsed_command.elapsed_seconds == 0
