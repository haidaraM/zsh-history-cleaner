from zshhistorycleaner.history import ZshHistory


def test_empty_history(tmpdir):
    """
    Test reading an empty history file
    """
    hist_file = tmpdir.join("empty.history")
    hist_file.write("")
    zsh_history = ZshHistory(hist_file.strpath)

    assert len(zsh_history.entries) == 0


def test_history_with_entries(tmpdir):
    """
    Test reading entries from a history file
    """
    hist_file = tmpdir.join("history")
    hist_file.write(""": 1604529234:0;l src/github.com
    : 1604529234:0;whoami
    """)

    history = ZshHistory(hist_file.strpath)
    assert len(history.entries) == 2


def test_history_with_invalid_entries(tmpdir, caplog):
    """
    Test reading a history file with invalid entries
    """
    hist_file = tmpdir.join("history")
    hist_file.write(""": 1604529234:0;echo hello
    : invalid_line
    : 1604529234:0;true
    second invalid_line
    """)

    history = ZshHistory(hist_file.strpath)
    assert len(history.entries) == 2

    # check warning in the log
    assert "line 2" in caplog.text, "There should a message regarding line 2 which is invalid"
    assert "line 4" in caplog.text, "There should a message regarding line 4 which is invalid"
