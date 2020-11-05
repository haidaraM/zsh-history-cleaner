from zshhistorycleaner.cleaner import ZshHistory


def test_empty_entries(tmpdir):
    """
    Empty an empty history file
    :return:
    """
    hist_file = tmpdir.join("empty.history")
    hist_file.write("")
    cleaner = ZshHistory(hist_file.strpath)
    assert len(cleaner.entries) == 0
    cleaner.remove_duplicates()
    assert len(cleaner.entries) == 0


def test_only_duplicate_commands(tmpdir):
    """
    Test for an history file containg only duplicates command
    :return:
    """
    entry1 = ": 1583848895:0;rm CHANGELOG.md"
    entry2 = ": 1583848896:0;rm CHANGELOG.md"
    hist_file = tmpdir.join("history.dup")
    hist_file.write(f"""
    {entry1}
    {entry2}
    """)

    history = ZshHistory(hist_file.strpath)
    assert len(history.entries) == 2
    history.remove_duplicates()
    assert len(history.entries) == 1
    assert history.entries[0].command == "rm CHANGELOG.md"


def test_without_duplicate_commands(tmpdir):
    """
    Test for an history file with no duplicated
    :return:
    """

    hist_file = tmpdir.join("history.no_dup")
    hist_file.write(f"""
    : 1583848895:0;rm CHANGELOG.md
    : 1583848896:0;ls
    """)

    history = ZshHistory(hist_file.strpath)
    assert len(history.entries) == 2
    history.remove_duplicates()
    assert len(history.entries) == 2

    # remove_duplicates should not alter the order
    assert history.entries[0].command == "rm CHANGELOG.md"
    assert history.entries[1].command == "ls"
