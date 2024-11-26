import os

from zshhistorycleaner.history import ZshHistory


def test_empty_history(tmpdir):
    """
    Test reading an empty history file
    """
    hist_file = tmpdir.join("empty.history")
    hist_file.write("")
    zsh_history = ZshHistory(hist_file.strpath)

    assert len(zsh_history.entries) == 0


def test_history_save(tmpdir):
    """
    Test saving history file
    """
    hist_file = tmpdir.join("history")
    hist_output_path = f"{hist_file.strpath}_output"

    hist_file.write(""": 1604529234:0;l src/github.com
    : 1604529235:0;whoami
    """)

    history = ZshHistory(hist_file.strpath)
    history.save(output_file_path=hist_output_path, backup=False)

    assert os.path.isfile(hist_output_path), f"The history should have been saved to {hist_output_path}"
    saved_history = ZshHistory(hist_output_path)
    assert len(saved_history.entries) == 2, "The saved history should have 2 entries"


def test_history_backup_save(tmpdir):
    """
    Test saving history file with a backup
    """
    hist_file = tmpdir.join("history")
    hist_output_path = f"{hist_file.strpath}_output"

    hist_file.write(""": 1604529234:0;l src/github.com
    : 1604529235:0;whoami
    """)

    history = ZshHistory(hist_file.strpath)
    backup_file_path = history.save(output_file_path=hist_output_path, backup=True)

    assert os.path.isfile(backup_file_path), f"The backup should have been save to {backup_file_path}"
