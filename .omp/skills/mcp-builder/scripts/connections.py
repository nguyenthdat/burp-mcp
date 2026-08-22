"""Connection handling for MCP evaluation clients."""

from __future__ import annotations

import os
from abc import ABC, abstractmethod
from contextlib import AsyncExitStack
from typing import Any

from mcp import ClientSession, StdioServerParameters
from mcp.client.sse import sse_client
from mcp.client.stdio import stdio_client
from mcp.client.streamable_http import streamablehttp_client


class MCPConnection(ABC):
    """Base class for an initialized MCP client connection."""

    def __init__(self) -> None:
        self.session: ClientSession | None = None
        self._stack: AsyncExitStack | None = None

    @abstractmethod
    def _create_context(self) -> Any:
        """Create the transport context for this connection."""

    async def __aenter__(self) -> MCPConnection:
        """Open the transport and initialize the MCP session."""
        self._stack = AsyncExitStack()
        await self._stack.__aenter__()

        try:
            result = await self._stack.enter_async_context(self._create_context())
            if len(result) == 2:
                read, write = result
            elif len(result) == 3:
                read, write, _ = result
            else:
                raise ValueError(f"Transport returned {len(result)} values; expected 2 or 3")

            self.session = await self._stack.enter_async_context(ClientSession(read, write))
            await self.session.initialize()
            return self
        except BaseException:
            await self._stack.aclose()
            self._stack = None
            raise

    async def __aexit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        """Close the MCP session and transport."""
        if self._stack is not None:
            await self._stack.__aexit__(exc_type, exc_val, exc_tb)
        self.session = None
        self._stack = None

    def _require_session(self) -> ClientSession:
        if self.session is None:
            raise RuntimeError("MCP connection is not initialized")
        return self.session

    async def list_tools(self) -> list[dict[str, Any]]:
        """Return all advertised tools, following MCP pagination cursors."""
        session = self._require_session()
        tools: list[dict[str, Any]] = []
        cursor: str | None = None

        while True:
            response = await session.list_tools(cursor=cursor)
            tools.extend(
                {
                    "name": tool.name,
                    "description": tool.description or "",
                    "input_schema": tool.inputSchema,
                }
                for tool in response.tools
            )
            cursor = response.nextCursor
            if not cursor:
                return tools

    async def call_tool(self, tool_name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        """Call a tool and preserve content, structured content, and error state."""
        result = await self._require_session().call_tool(tool_name, arguments=arguments)
        return result.model_dump(mode="json", by_alias=True, exclude_none=True)


class MCPConnectionStdio(MCPConnection):
    """MCP connection over a client-owned stdio process."""

    def __init__(
        self,
        command: str,
        args: list[str] | None = None,
        env: dict[str, str] | None = None,
    ) -> None:
        super().__init__()
        self.command = command
        self.args = args or []
        self.env = env

    def _create_context(self) -> Any:
        return stdio_client(
            StdioServerParameters(
                command=self.command,
                args=self.args,
                env={**os.environ, **self.env} if self.env is not None else None,
            )
        )


class MCPConnectionSSE(MCPConnection):
    """Legacy MCP connection over Server-Sent Events."""

    def __init__(self, url: str, headers: dict[str, str] | None = None) -> None:
        super().__init__()
        self.url = url
        self.headers = headers or {}

    def _create_context(self) -> Any:
        return sse_client(url=self.url, headers=self.headers)


class MCPConnectionHTTP(MCPConnection):
    """MCP connection over Streamable HTTP."""

    def __init__(self, url: str, headers: dict[str, str] | None = None) -> None:
        super().__init__()
        self.url = url
        self.headers = headers or {}

    def _create_context(self) -> Any:
        return streamablehttp_client(url=self.url, headers=self.headers)


def create_connection(
    transport: str,
    command: str | None = None,
    args: list[str] | None = None,
    env: dict[str, str] | None = None,
    url: str | None = None,
    headers: dict[str, str] | None = None,
) -> MCPConnection:
    """Create a validated MCP connection for the requested transport."""
    normalized = transport.lower().replace("-", "_")

    if normalized == "stdio":
        if not command:
            raise ValueError("stdio requires a server command after `--` or via --command")
        return MCPConnectionStdio(command=command, args=args, env=env)

    if normalized == "sse":
        if not url:
            raise ValueError("legacy SSE requires --url")
        return MCPConnectionSSE(url=url, headers=headers)

    if normalized in {"http", "streamable_http"}:
        if not url:
            raise ValueError("Streamable HTTP requires --url")
        return MCPConnectionHTTP(url=url, headers=headers)

    raise ValueError(f"Unsupported transport {transport!r}; use stdio, http, or legacy sse")
