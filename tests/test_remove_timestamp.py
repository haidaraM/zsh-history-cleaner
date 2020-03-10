from zshhistorycleaner.main import parse_zsh_command


def test_empty_command():
    """
    An empty command should return an empty command
    :return:
    """
    assert parse_zsh_command("") is None


def test_no_timestamp():
    """
    A command without timestamp should not fail
    :return:
    """
    assert parse_zsh_command("ls;") is None


def test_simple_command():
    """

    :return:
    """
    command = ": 1556053755:2;printenv"
    parsed_command = parse_zsh_command(command)
    assert parsed_command is not None
    assert parsed_command.group("command") == "printenv"
    assert parsed_command.group("beginning_time") == "1556053755"
    assert parsed_command.group("elapsed_seconds") == "2"


def test_complex_command():
    """

    :return:
    """
    command = ": 1557138761:0;for d in VWT.*; do l $d; done"
    parsed_command = parse_zsh_command(command)
    assert parsed_command is not None
    assert parsed_command.group("command") == "for d in VWT.*; do l $d; done"
    assert parsed_command.group("beginning_time") == "1557138761"
    assert parsed_command.group("elapsed_seconds") == "0"
