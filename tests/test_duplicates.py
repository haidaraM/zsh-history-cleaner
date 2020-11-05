from zshhistorycleaner.history import ZshHistory


def test_only_duplicate_commands(tmpdir):
    """
    Test for an history file containg only duplicates command
    :return:
    """
    hist_file = tmpdir.join("history.dup")
    hist_file.write(f""": 1583848895:0;rm CHANGELOG.md
    : 1583848896:0;rm CHANGELOG.md
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
    hist_file.write(f""": 1583848895:0;rm CHANGELOG.md
    : 1583848896:0;ls
    """)
    history = ZshHistory(hist_file.strpath)

    assert len(history.entries) == 2
    history.remove_duplicates()
    assert len(history.entries) == 2


def test_order_of_entries_after_duplicates_removal(tmpdir):
    """
    Test if we keep the same order of entries after removing the duplicates
    """
    hist_file = tmpdir.join("history.dup")
    hist_file.write(f""": 1583848895:0;rm CHANGELOG.md
    : 1604435793:1;cat tests/.coverage
    : 1604435822:3;pip install coveralls
    : 1604485227:1;cat tests/.coverage
    """)

    history = ZshHistory(hist_file.strpath)
    history.remove_duplicates()
    assert len(history.entries) == 3, "3 entries should be there after removing the duplicate."
    # remove_duplicates should not alter the order of entries
    assert history.entries[0].command == "rm CHANGELOG.md"
    assert history.entries[1].command == "cat tests/.coverage"
    assert history.entries[2].command == "pip install coveralls"
