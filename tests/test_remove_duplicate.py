from zshhistorycleaner.main import remove_duplicate_entries, ZshHistoryEntry


def test_empty_entries():
    """
    Empty list
    :return:
    """
    assert remove_duplicate_entries([]) == []


def test_with_duplicate_commands():
    """

    :return:
    """
    entry_1 = ZshHistoryEntry(raw_line=": 1583846895:0;rm CHANGELOG.md", beginning_time=1583846895,
                              command="rm CHANGELOG.md", elapsed_seconds=0, line_number=1)
    entry_2 = ZshHistoryEntry(raw_line=": 1234578999:0;rm CHANGELOG.md", beginning_time=1583846895,
                              command="rm CHANGELOG.md", elapsed_seconds=0, line_number=1)

    assert remove_duplicate_entries([entry_1, entry_2]) == [entry_1]


def test_without_duplicate_commands():
    """

    :return:
    """
    entry_1 = ZshHistoryEntry(raw_line=": 1583846895:0;rm CHANGELOG.md", beginning_time=1583846895,
                              command="rm CHANGELOG.md", elapsed_seconds=0, line_number=1)
    entry_2 = ZshHistoryEntry(raw_line=": 1234578999:0;ls", beginning_time=1583846895,
                              command="ls", elapsed_seconds=0, line_number=1)

    assert remove_duplicate_entries([entry_1, entry_2]) == [entry_1, entry_2]
