"""Evaluate an MCP server with reproducible question/answer fixtures.

This lightweight harness measures model/tool usability. It is not a substitute
for deterministic MCP protocol tests or Goose end-to-end acceptance tests.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import re
import sys
import time
import xml.etree.ElementTree as ET
from collections.abc import Sequence
from pathlib import Path
from typing import Any

from anthropic import Anthropic
from connections import create_connection

DEFAULT_MODEL = os.environ.get("ANTHROPIC_MODEL")
SECRET_KEY_PATTERN = re.compile(
    r"(?:authorization|api[-_]?key|token|secret|password|credential|cookie)",
    re.IGNORECASE,
)
SECRET_VALUE_PATTERN = re.compile(
    r"(?i)(bearer\s+)[^\s,;]+|((?:api[-_]?key|token|secret|password)\s*[:=]\s*)[^\s,;]+"
)

EVALUATION_PROMPT = """You are evaluating an MCP server with the tools provided.

For the task:
1. Use tools only as needed and honor the task's read-only requirement.
2. Return a concise approach inside <summary>...</summary>. Mention tool names
   and decision points, but never reproduce credentials or full tool payloads.
3. Return actionable tool-design feedback inside <feedback>...</feedback>.
4. Return only the requested value inside <response>...</response>, last.
5. If the task cannot be solved, return <response>NOT_FOUND</response>.
"""


def build_parser() -> argparse.ArgumentParser:
    """Build the CLI parser so published examples can be tested directly."""
    parser = argparse.ArgumentParser(
        description="Evaluate MCP tool usability with XML question/answer fixtures",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Stdio: put the server command after --
  uv run python3 scripts/evaluation.py evaluation.xml \\
    --transport stdio --env API_KEY=value -- \\
    uv run python3 -m my_mcp_server

  # Streamable HTTP; repeat --header for multiple headers
  uv run python3 scripts/evaluation.py evaluation.xml \\
    --transport http --url https://example.com/mcp \\
    --header "Authorization: Bearer token"

  # Legacy SSE compatibility only
  uv run python3 scripts/evaluation.py evaluation.xml \\
    --transport sse --url https://example.com/sse
        """,
    )
    parser.add_argument("eval_file", type=Path, help="Evaluation XML file")
    parser.add_argument(
        "-t",
        "--transport",
        choices=["stdio", "http", "sse"],
        default="stdio",
        help="MCP transport; sse is legacy compatibility only",
    )
    parser.add_argument(
        "-m",
        "--model",
        default=DEFAULT_MODEL,
        help="Anthropic model ID (or set ANTHROPIC_MODEL; required)",
    )
    parser.add_argument(
        "--max-turns",
        type=positive_int,
        default=20,
        help="Maximum model responses per task (default: 20)",
    )
    parser.add_argument(
        "--task-timeout",
        type=positive_float,
        default=300.0,
        help="Per-task wall-clock timeout in seconds (default: 300)",
    )
    parser.add_argument(
        "-e",
        "--env",
        action="append",
        default=[],
        metavar="KEY=VALUE",
        help="stdio environment entry; repeat for multiple values",
    )
    parser.add_argument(
        "-u",
        "--url",
        help="Remote MCP URL for http or legacy sse",
    )
    parser.add_argument(
        "-H",
        "--header",
        action="append",
        default=[],
        dest="headers",
        metavar="KEY: VALUE",
        help="HTTP header; repeat for multiple values",
    )
    parser.add_argument(
        "-c",
        "--command",
        help="Deprecated stdio command form; prefer command after --",
    )
    parser.add_argument(
        "-a",
        "--arg",
        action="append",
        default=[],
        dest="command_args",
        help="Deprecated stdio server argument; repeat as needed",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Write the Markdown report to this file instead of stdout",
    )
    return parser


def parse_cli_args(
    argv: Sequence[str] | None = None,
    parser: argparse.ArgumentParser | None = None,
) -> argparse.Namespace:
    """Parse evaluator options before `--` and preserve the stdio command after it."""
    parser = parser or build_parser()
    values = list(argv) if argv is not None else sys.argv[1:]
    if "--" in values:
        separator_index = values.index("--")
        option_values = values[:separator_index]
        server_command = values[separator_index + 1 :]
    else:
        option_values = values
        server_command = []
    args = parser.parse_args(option_values)
    args.server_command = server_command
    return args


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def positive_float(value: str) -> float:
    parsed = float(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def parse_evaluation_file(file_path: Path) -> list[dict[str, str]]:
    """Parse and validate an XML evaluation file."""
    try:
        root = ET.parse(file_path).getroot()
    except (OSError, ET.ParseError) as exc:
        raise ValueError(f"Cannot parse evaluation file {file_path}: {exc}") from exc

    if root.tag != "evaluation":
        raise ValueError("Evaluation XML root must be <evaluation>")

    evaluations: list[dict[str, str]] = []
    for index, qa_pair in enumerate(root.findall("./qa_pair"), start=1):
        question = (qa_pair.findtext("question") or "").strip()
        answer = (qa_pair.findtext("answer") or "").strip()
        if not question or not answer:
            raise ValueError(f"qa_pair {index} must contain non-empty <question> and <answer>")
        evaluations.append({"question": question, "answer": answer})

    if not evaluations:
        raise ValueError("Evaluation file contains no <qa_pair> tasks")
    return evaluations


def extract_xml_content(text: str | None, tag: str) -> str | None:
    """Extract the final occurrence of a simple tagged response section."""
    if not text:
        return None
    matches = re.findall(rf"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
    return matches[-1].strip() if matches else None


def redact(value: Any) -> Any:
    """Recursively redact common credential fields before model/report use."""
    if isinstance(value, dict):
        return {
            key: "<redacted>" if SECRET_KEY_PATTERN.search(str(key)) else redact(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [redact(item) for item in value]
    if isinstance(value, str):
        return SECRET_VALUE_PATTERN.sub(
            lambda match: f"{match.group(1) or match.group(2)}<redacted>", value
        )
    return value


def serialize_tool_result(result: Any) -> str:
    """Serialize a redacted MCP result for the model."""
    return json.dumps(redact(result), ensure_ascii=False, separators=(",", ":"))


def response_text(response: Any) -> str:
    """Join all text blocks from a provider response."""
    return "\n".join(
        block.text for block in response.content if getattr(block, "type", None) == "text"
    )


async def create_message(
    client: Anthropic,
    *,
    model: str,
    messages: list[dict[str, Any]],
    tools: list[dict[str, Any]],
) -> Any:
    """Call the synchronous Anthropic client without blocking the event loop."""

    def request() -> Any:
        return client.messages.create(
            model=model,
            max_tokens=4096,
            system=EVALUATION_PROMPT,
            messages=messages,  # type: ignore[arg-type]
            tools=tools,  # type: ignore[arg-type]
        )

    return await asyncio.to_thread(request)


async def execute_tool_use(
    connection: Any,
    tool_use: Any,
) -> tuple[dict[str, Any], str, float]:
    """Execute one model-requested MCP call and create its provider result block."""
    started = time.monotonic()
    is_error = False
    try:
        result = await connection.call_tool(tool_use.name, tool_use.input)
        tool_response = serialize_tool_result(result)
        is_error = bool(result.get("isError", False)) if isinstance(result, dict) else False
    except Exception as exc:  # The model receives a safe summary, not a traceback.
        is_error = True
        tool_response = json.dumps(
            {
                "isError": True,
                "content": [
                    {
                        "type": "text",
                        "text": f"Tool execution failed: {type(exc).__name__}: {exc}",
                    }
                ],
            },
            ensure_ascii=False,
        )
    duration = time.monotonic() - started
    block = {
        "type": "tool_result",
        "tool_use_id": tool_use.id,
        "content": tool_response,
        "is_error": is_error,
    }
    return block, tool_use.name, duration


async def agent_loop(
    client: Anthropic,
    model: str,
    question: str,
    tools: list[dict[str, Any]],
    connection: Any,
    max_turns: int,
) -> tuple[str, dict[str, Any]]:
    """Run a bounded model loop and answer every tool use in each response."""
    messages: list[dict[str, Any]] = [{"role": "user", "content": question}]
    tool_metrics: dict[str, dict[str, Any]] = {}

    for _turn in range(max_turns):
        response = await create_message(
            client,
            model=model,
            messages=messages,
            tools=tools,
        )
        messages.append({"role": "assistant", "content": response.content})

        tool_uses = [
            block for block in response.content if getattr(block, "type", None) == "tool_use"
        ]
        if not tool_uses:
            return response_text(response), tool_metrics

        executed = await asyncio.gather(
            *(execute_tool_use(connection, tool_use) for tool_use in tool_uses)
        )
        result_blocks: list[dict[str, Any]] = []
        for result_block, tool_name, duration in executed:
            metrics = tool_metrics.setdefault(tool_name, {"count": 0, "durations": []})
            metrics["count"] += 1
            metrics["durations"].append(duration)
            result_blocks.append(result_block)
        messages.append({"role": "user", "content": result_blocks})

    raise RuntimeError(f"Task exceeded --max-turns={max_turns}")


async def evaluate_single_task(
    client: Anthropic,
    model: str,
    qa_pair: dict[str, str],
    tools: list[dict[str, Any]],
    connection: Any,
    task_index: int,
    max_turns: int,
    task_timeout: float,
) -> dict[str, Any]:
    """Evaluate one fixture within its wall-clock budget."""
    started = time.monotonic()
    print(f"Task {task_index + 1}: {qa_pair['question']}", file=sys.stderr)

    try:
        response, tool_metrics = await asyncio.wait_for(
            agent_loop(
                client,
                model,
                qa_pair["question"],
                tools,
                connection,
                max_turns,
            ),
            timeout=task_timeout,
        )
        error = None
    except Exception as exc:
        response = ""
        tool_metrics = {}
        error = f"{type(exc).__name__}: {exc}"

    response_value = extract_xml_content(response, "response")
    return {
        "question": qa_pair["question"],
        "expected": qa_pair["answer"],
        "actual": response_value,
        "score": int(response_value == qa_pair["answer"]) if response_value else 0,
        "total_duration": time.monotonic() - started,
        "tool_calls": tool_metrics,
        "num_tool_calls": sum(item["count"] for item in tool_metrics.values()),
        "summary": extract_xml_content(response, "summary"),
        "feedback": extract_xml_content(response, "feedback"),
        "error": error,
    }


REPORT_HEADER = """# Evaluation Report

## Run Configuration

- **Model**: `{model}`
- **Transport**: `{transport}`
- **Run date**: `{run_date}`

## Summary

- **Accuracy**: {correct}/{total} ({accuracy:.1f}%)
- **Average Task Duration**: {average_duration_s:.2f}s
- **Average Tool Calls per Task**: {average_tool_calls:.2f}
- **Total Tool Calls**: {total_tool_calls}

> This report measures model/tool usability. Run deterministic protocol tests
> and target-client acceptance tests separately.

---
"""

TASK_TEMPLATE = """## Task {task_num}

**Question:** {question}

- **Expected:** `{expected_answer}`
- **Actual:** `{actual_answer}`
- **Correct:** {correct_indicator}
- **Duration:** {total_duration:.2f}s
- **Tool calls:** `{num_tool_calls}`
- **Error:** {error}

### Summary

{summary}

### Tool feedback

{feedback}

<details>
<summary>Redacted tool metrics</summary>

```json
{tool_calls}
```
</details>

---
"""


async def run_evaluation(
    eval_path: Path,
    connection: Any,
    *,
    model: str,
    transport: str,
    max_turns: int,
    task_timeout: float,
) -> tuple[str, int]:
    """Run every validated fixture and return the report plus failure count."""
    client = Anthropic()
    tools = await connection.list_tools()
    qa_pairs = parse_evaluation_file(eval_path)
    print(
        f"Loaded {len(tools)} tools and {len(qa_pairs)} tasks",
        file=sys.stderr,
    )

    results = []
    for index, qa_pair in enumerate(qa_pairs):
        results.append(
            await evaluate_single_task(
                client,
                model,
                qa_pair,
                tools,
                connection,
                index,
                max_turns,
                task_timeout,
            )
        )

    total = len(results)
    correct = sum(result["score"] for result in results)
    total_tool_calls = sum(result["num_tool_calls"] for result in results)
    report = REPORT_HEADER.format(
        model=model,
        transport=transport,
        run_date=time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        correct=correct,
        total=total,
        accuracy=(correct / total) * 100,
        average_duration_s=sum(result["total_duration"] for result in results) / total,
        average_tool_calls=total_tool_calls / total,
        total_tool_calls=total_tool_calls,
    )
    report += "".join(
        TASK_TEMPLATE.format(
            task_num=index + 1,
            question=qa_pair["question"],
            expected_answer=qa_pair["answer"],
            actual_answer=result["actual"] or "N/A",
            correct_indicator="✅" if result["score"] else "❌",
            total_duration=result["total_duration"],
            num_tool_calls=result["num_tool_calls"],
            error=result["error"] or "None",
            tool_calls=json.dumps(redact(result["tool_calls"]), indent=2),
            summary=result["summary"] or "N/A",
            feedback=result["feedback"] or "N/A",
        )
        for index, (qa_pair, result) in enumerate(zip(qa_pairs, results, strict=True))
    )
    return report, total - correct


def parse_key_values(values: Sequence[str], separator: str, label: str) -> dict[str, str]:
    """Parse repeatable CLI key/value options strictly."""
    parsed: dict[str, str] = {}
    for raw in values:
        if separator not in raw:
            raise ValueError(f"Malformed {label} {raw!r}; expected KEY{separator}VALUE")
        key, value = raw.split(separator, 1)
        key = key.strip()
        if not key:
            raise ValueError(f"Malformed {label} {raw!r}; key cannot be empty")
        parsed[key] = value.strip()
    return parsed


def normalize_server_command(args: argparse.Namespace) -> tuple[str | None, list[str]]:
    """Resolve the preferred remainder command or the deprecated split form."""
    remainder = list(args.server_command)
    if remainder and remainder[0] == "--":
        remainder.pop(0)
    if remainder:
        if args.command or args.command_args:
            raise ValueError("Use either the command after -- or --command/--arg, not both")
        return remainder[0], remainder[1:]
    return args.command, list(args.command_args)


async def async_main(argv: Sequence[str] | None = None) -> int:
    """Parse arguments, connect, evaluate, and return a process exit code."""
    parser = build_parser()
    args = parse_cli_args(argv, parser)

    if not args.eval_file.is_file():
        parser.error(f"evaluation file not found: {args.eval_file}")
    if not args.model:
        parser.error("--model is required (or set ANTHROPIC_MODEL)")

    try:
        qa_pairs = parse_evaluation_file(args.eval_file)
        if not qa_pairs:  # Defensive; parser currently rejects empty files.
            raise ValueError("evaluation file contains no tasks")
        env = parse_key_values(args.env, "=", "environment variable")
        headers = parse_key_values(args.headers, ":", "header")
        command, command_args = normalize_server_command(args)
        if args.transport == "stdio" and args.url:
            raise ValueError("--url cannot be used with stdio")
        if args.transport != "stdio" and (command or command_args or env):
            raise ValueError("server command and --env are valid only with stdio")
        connection = create_connection(
            transport=args.transport,
            command=command,
            args=command_args,
            env=env or None,
            url=args.url,
            headers=headers or None,
        )
    except ValueError as exc:
        parser.error(str(exc))

    print(f"Connecting via {args.transport}...", file=sys.stderr)
    try:
        async with connection:
            report, failures = await run_evaluation(
                args.eval_file,
                connection,
                model=args.model,
                transport=args.transport,
                max_turns=args.max_turns,
                task_timeout=args.task_timeout,
            )
    except Exception as exc:
        print(f"Evaluation failed: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
        print(f"Report saved to {args.output}", file=sys.stderr)
    else:
        print(report)
    return 1 if failures else 0


def main(argv: Sequence[str] | None = None) -> int:
    """Synchronous console entry point."""
    return asyncio.run(async_main(argv))


if __name__ == "__main__":
    raise SystemExit(main())
