"""The `trace` commands: the scenarios, the test items and the ADR citations, checked against each other."""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

import cyclopts

from project_tasks.adr import check_adrs, load_adrs
from project_tasks.context import note, step
from project_tasks.items import build_tree, collect, differences

DEFAULT_PREFIX = "TST"
DEFAULT_ADR_DIR = Path("docs/content/development/adr")

Prefix = Annotated[str, cyclopts.Parameter(help="Doorstop document holding the test items.")]
AdrDir = Annotated[Path, cyclopts.Parameter(help="Directory holding the ADRs.")]

app = cyclopts.App(
    name="trace",
    help="Check the scenarios, the test items and the ADR citations against each other.",
)


@app.command
def check(*, prefix: Prefix = DEFAULT_PREFIX, adr_dir: AdrDir = DEFAULT_ADR_DIR) -> int:
    """Verify the scenario mapping, the test items and the ADR citations. Never writes.

    Fails when a scenario has no test tag, a tag has no item, an item has no scenario, a tag sits on
    a `Feature:`, `Rule:` or `Examples:`, an item has drifted from the scenario it mirrors, an ADR
    cites an item that is not active, or anything cites an ADR that does not exist or is not accepted.
    """
    step("Checking the scenarios, the test items and the ADR citations")
    tree, root = build_tree()
    mapping = collect(tree, root, prefix)
    errors = list(mapping.errors)
    errors += check_adrs(tree, root, load_adrs(tree, root, adr_dir))

    for item, what, differing in differences(mapping):
        errors.append(
            f"{item.uid}: {', '.join(sorted(differing))} out of date with the {what}; "
            f"run `tools/project req sync`"
        )

    for error in errors:
        print(f"error: {error}")
    if errors:
        return 1

    note(
        f"{len(mapping.scenarios)} scenario(s) in {len(mapping.features)} feature file(s) mapped "
        f"and in sync, ADR citations valid"
    )
    return 0
