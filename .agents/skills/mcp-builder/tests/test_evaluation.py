"""Unit tests for the MCP evaluation harness."""

from __future__ import annotations

import sys
from pathlib import Path
from types import SimpleNamespace

import pytest

sys.path.insert(0, str(Path(__file__).parents[1] / "scripts"))

from evaluation import (
    normalize_server_command,
    parse_cli_args,
    parse_evaluation_file,
    parse_key_values,
    redact,
)


def test_stdio_command_after_separator_is_not_greedy() -> None:
    args = parse_cli_args(
        [
            "fixture.xml",
            "--transport",
            "stdio",
            "--env",
            "API_KEY=one",
            "--env",
            "DEBUG=true",
            "--",
            "uv",
            "run",
            "python",
            "-m",
            "server",
        ]
    )

    command, command_args = normalize_server_command(args)
    assert args.eval_file == Path("fixture.xml")
    assert parse_key_values(args.env, "=", "environment variable") == {
        "API_KEY": "one",
        "DEBUG": "true",
    }
    assert command == "uv"
    assert command_args == ["run", "python", "-m", "server"]


def test_headers_are_repeatable() -> None:
    args = parse_cli_args(
        [
            "fixture.xml",
            "--transport",
            "http",
            "--url",
            "https://example.test/mcp",
            "--header",
            "Authorization: Bearer secret",
            "--header",
            "X-Request-ID: abc",
        ]
    )
    assert parse_key_values(args.headers, ":", "header") == {
        "Authorization": "Bearer secret",
        "X-Request-ID": "abc",
    }


def test_command_forms_cannot_be_mixed() -> None:
    args = SimpleNamespace(
        server_command=["--", "uv", "run", "server.py"],
        command="python3",
        command_args=[],
    )
    with pytest.raises(ValueError, match="either"):
        normalize_server_command(args)


def test_malformed_key_value_is_rejected() -> None:
    with pytest.raises(ValueError, match="Malformed header"):
        parse_key_values(["Authorization"], ":", "header")


def test_parse_evaluation_rejects_empty_document(tmp_path: Path) -> None:
    fixture = tmp_path / "empty.xml"
    fixture.write_text("<evaluation />", encoding="utf-8")
    with pytest.raises(ValueError, match="no <qa_pair>"):
        parse_evaluation_file(fixture)


def test_parse_evaluation_rejects_missing_answer(tmp_path: Path) -> None:
    fixture = tmp_path / "invalid.xml"
    fixture.write_text(
        "<evaluation><qa_pair><question>Q?</question></qa_pair></evaluation>",
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="non-empty"):
        parse_evaluation_file(fixture)


def test_parse_evaluation_accepts_valid_fixture(tmp_path: Path) -> None:
    fixture = tmp_path / "valid.xml"
    fixture.write_text(
        """<evaluation>
        <qa_pair><question>Q?</question><answer>A</answer></qa_pair>
        </evaluation>""",
        encoding="utf-8",
    )
    assert parse_evaluation_file(fixture) == [{"question": "Q?", "answer": "A"}]


def test_redact_removes_secret_fields_and_inline_values() -> None:
    value = {
        "Authorization": "Bearer abc",
        "nested": {"api_key": "secret"},
        "message": "token=visible normal=value",
    }
    assert redact(value) == {
        "Authorization": "<redacted>",
        "nested": {"api_key": "<redacted>"},
        "message": "token=<redacted> normal=value",
    }
