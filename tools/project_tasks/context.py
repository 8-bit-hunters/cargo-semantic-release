"""Shared plumbing: where the project is, and how its tools are run."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path
from typing import NoReturn, Sequence

# The manifest marks the root; `.git` alone would also match a submodule or a nested checkout.
ROOT_MARKER = "Cargo.toml"


def repo_root(start: Path | None = None) -> Path:
    """The project root, found by walking up from `start` (the working directory by default)."""
    current = (start or Path.cwd()).resolve()
    for candidate in (current, *current.parents):
        if (candidate / ROOT_MARKER).is_file() and (candidate / "plm").is_dir():
            return candidate
    fail(f"not inside the project: no directory above {current} holds {ROOT_MARKER} and plm/")


def step(message: str) -> None:
    """Announce a step of a composite task, so its order is visible while it runs."""
    print(f"\n==> {message}", flush=True)


def note(message: str) -> None:
    print(f"    {message}", flush=True)


def fail(message: str) -> NoReturn:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(
    command: Sequence[str],
    *,
    cwd: Path | None = None,
    allow_failure: bool = False,
    printable: str | None = None,
) -> int:
    """Run a command, showing it first so that it can be run by hand afterwards.

    `printable` gives a friendlier spelling than the one that is executed, for a tool invoked through
    `python -m` whose real command line is an interpreter path nobody would type.
    """
    printable = printable or " ".join(str(part) for part in command)
    print(f"    $ {printable}", flush=True)
    completed = subprocess.run([str(part) for part in command], cwd=cwd)
    if completed.returncode and not allow_failure:
        fail(f"`{printable}` exited with {completed.returncode}")
    return completed.returncode


def doorstop(*arguments: str, cwd: Path | None = None, allow_failure: bool = False) -> int:
    """Run Doorstop's own command line, in this tool's environment."""
    return run(
        [sys.executable, "-m", "doorstop.cli.main", *arguments],
        cwd=cwd,
        allow_failure=allow_failure,
        printable=" ".join(["doorstop", *arguments]),
    )


def require_tool(name: str, hint: str) -> None:
    if shutil.which(name) is None:
        fail(f"{name} is not installed, and this task needs it. {hint}")
