"""The `req` commands: creating, validating, syncing and publishing the requirements."""

from __future__ import annotations

from pathlib import Path
from typing import Annotated, Literal

import cyclopts

from project_tasks.adr import load_adrs
from project_tasks.context import doorstop, note, step
from project_tasks.items import (
    apply,
    build_tree,
    collect,
    differences,
    publish_documents,
)

DEFAULT_PREFIX = "TST"
DEFAULT_ADR_DIR = Path("docs/content/development/adr")
DEFAULT_CONTENT_DIR = Path("docs/content/development/requirements")
DEFAULT_STATIC_DIR = Path("docs/static/requirements")

Policy = Literal["warn", "error"]
Prefix = Annotated[str, cyclopts.Parameter(help="Doorstop document holding the test items.")]
AdrDir = Annotated[Path, cyclopts.Parameter(help="Directory holding the ADRs.")]

app = cyclopts.App(name="req", help="Create, validate, sync and publish the requirements.")


def validation_flags(unlinked: Policy, unreviewed: Policy) -> list[str]:
    """Doorstop's flags for a given policy.

    Doorstop reports both a requirement that no test item covers and an item whose fingerprint is
    stale as warnings, and warnings do not fail its run. `-e` turns warnings into errors, `-C` drops
    the coverage warnings and `-W` drops the review ones, so the four combinations of the two
    policies are expressible.
    """
    return {
        ("warn", "warn"): [],
        ("warn", "error"): ["-C", "-e"],
        ("error", "warn"): ["-W", "-e"],
        ("error", "error"): ["-e"],
    }[(unlinked, unreviewed)]


@app.command
def check(*, unlinked: Policy = "warn", unreviewed: Policy = "error") -> int:
    """Validate the requirement tree with Doorstop.

    Parameters
    ----------
    unlinked
        What a requirement that no test item covers counts as. Most of the workspace requirements
        have no scenario yet, so this is a warning by default.
    unreviewed
        What an item whose fingerprint no longer matches its content counts as — a change somebody
        has not reviewed. An error by default: it is a step missing from the change at hand, not a
        standing property of the project.
    """
    flags = validation_flags(unlinked, unreviewed)
    if flags:
        step("Reporting the requirement tree (nothing here fails the run)")
        doorstop(allow_failure=True)
    step(f"Validating the tree (unlinked requirements {unlinked}, unreviewed items {unreviewed})")
    doorstop(*flags)
    return 0


@app.command
def new(
    *,
    document: Annotated[str, cyclopts.Parameter(help="Document prefix, e.g. REQ or TST.")] = "REQ",
    level: Annotated[str | None, cyclopts.Parameter(help="Where in the outline, e.g. 3.7.")] = None,
    count: Annotated[int, cyclopts.Parameter(help="How many items to add.")] = 1,
) -> int:
    """Add an item, letting Doorstop assign the next UID.

    A level whose last segment ends in a zero has to be quoted in the item file; Doorstop writes it
    unquoted, so YAML reads `1.10` back as `1.1`. Check the file after adding one of those.
    """
    arguments = ["add", document, "-c", str(count)]
    if level:
        arguments += ["-l", level]
    doorstop(*arguments)
    note("Fill in the item, then run `tools/project req check`.")
    return 0


@app.command
def sync(*, prefix: Prefix = DEFAULT_PREFIX) -> int:
    """Write into the test items what the feature files imply, then say what needs reviewing."""
    tree, root = build_tree()
    mapping = collect(tree, root, prefix)
    if mapping.errors:
        for error in mapping.errors:
            print(f"error: {error}")
        print("nothing written: fix the mapping errors first")
        return 1

    changed = []
    for item, what, differing in differences(mapping):
        for name, value in differing.items():
            apply(item, name, value)
        item.save()
        changed.append(item)
        note(f"updated {item.uid} ({', '.join(sorted(differing))}) from the {what}")

    if not changed:
        note(f"{len(mapping.scenarios)} scenario(s) already in sync")
        return 0

    step("Review the changed items")
    note("tools/project req review " + " ".join(str(item.uid) for item in changed))
    return 0


@app.command
def review(
    uids: Annotated[list[str] | None, cyclopts.Parameter(help="Items to review; all if omitted.")] = None,
) -> int:
    """Accept the current content of items, stamping their fingerprints."""
    doorstop("review", *(uids or ["all"]))
    return 0


@app.command
def publish(
    *,
    content: Annotated[Path, cyclopts.Parameter(help="Section directory for the document pages.")] = DEFAULT_CONTENT_DIR,
    static: Annotated[Path, cyclopts.Parameter(help="Directory for the standalone Doorstop site.")] = DEFAULT_STATIC_DIR,
) -> int:
    """Render the documents into the documentation site.

    One Markdown page per document goes into the site's requirements section, so the requirements are
    browsable and searchable like any other page, and the standalone Doorstop site — which carries the
    traceability matrix — goes into the site's static directory.
    """
    _, root = build_tree()
    step("Publishing the requirement documents into the site")
    publish_documents(root, content, static)
    return 0


@app.command
def impact(uid: str, *, adr_dir: AdrDir = DEFAULT_ADR_DIR) -> int:
    """List the ADRs that cite a Doorstop item.

    Parameters
    ----------
    uid
        The item to look for, e.g. REQ-018.
    """
    tree, root = build_tree()
    citing = [adr for adr in load_adrs(tree, root, adr_dir) if uid in adr.cites]
    if not citing:
        print(f"no ADR cites {uid}")
        return 0
    for adr in citing:
        print(f"{adr.rel} ({adr.status})")
    return 0
