"""The `docs` commands: building and serving the documentation site, in the order that works."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Annotated

import cyclopts

from project_tasks.context import note, repo_root, require_tool, run, step
from project_tasks.items import publish_documents

DEFAULT_CONTENT_DIR = Path("docs/content/development/requirements")
DEFAULT_STATIC_DIR = Path("docs/static/requirements")
SITE_DIR = Path("docs")
# Hugo's output and its asset cache. Left in place, `hugo server` serves the stale files from them
# instead of what it has just built, which makes a change look as if it had no effect.
GENERATED = (Path("docs/public"), Path("docs/resources"))

app = cyclopts.App(name="docs", help="Build or serve the documentation site.")

Port = Annotated[int, cyclopts.Parameter(help="Port for the development server.")]
BaseUrl = Annotated[str | None, cyclopts.Parameter(help="Site URL, e.g. for a project page.")]


def prepare(root: Path) -> None:
    """Everything that has to happen before Hugo runs, in order.

    The requirements pages are generated, so they have to exist before Hugo reads the content tree,
    and Hugo's previous output has to be gone before it serves anything.
    """
    require_tool("hugo", "Install the extended build of Hugo, and Go for the theme module.")
    require_tool("go", "The Hextra theme is a Hugo module, which needs Go to fetch.")

    step("Publishing the requirement documents into the site")
    publish_documents(root, DEFAULT_CONTENT_DIR, DEFAULT_STATIC_DIR)

    step("Clearing Hugo's previous output")
    for path in GENERATED:
        target = root / path
        if target.exists():
            shutil.rmtree(target)
            note(f"removed {path}")
        else:
            note(f"{path} was already gone")


@app.command
def build(*, base_url: BaseUrl = None) -> int:
    """Publish the requirements, then build the site into `docs/public`."""
    root = repo_root()
    prepare(root)
    step("Building the site")
    command = ["hugo", "--gc", "--minify"]
    if base_url:
        command += ["--baseURL", base_url]
    run(command, cwd=root / SITE_DIR)
    note(f"built into {SITE_DIR / 'public'}")
    return 0


@app.command
def serve(*, port: Port = 1313) -> int:
    """Publish the requirements, then serve the site with live reload."""
    root = repo_root()
    prepare(root)
    step(f"Serving the site on http://localhost:{port}/")
    note("The requirements pages are a snapshot: re-run this after changing an item.")
    run(["hugo", "server", "--port", str(port)], cwd=root / SITE_DIR)
    return 0
