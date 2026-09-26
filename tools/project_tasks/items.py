"""Reconciling the Doorstop test items with the scenarios, and adapting documents for the site."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import doorstop

from project_tasks.gherkin import Feature, Scenario, parse_feature, repo_files

HASH_ATTR = "scenario_hash"
# What each document is called in the site's navigation.
DOCUMENT_TITLES = {"REQ": "Requirements", "TST": "Test cases"}


def build_tree() -> tuple[Any, Path]:
    tree = doorstop.build()
    return tree, Path(getattr(tree, "root", None) or Path.cwd())


@dataclass
class Mapping:
    """What the feature files and the test document say, reconciled."""

    features: list[Feature]
    errors: list[str]
    headings: dict[str, Any]  # feature path -> item
    scenarios: dict[str, tuple[Any, Feature, Scenario]]  # uid -> (item, feature, scenario)


def collect(tree, root: Path, prefix: str) -> Mapping:
    try:
        document = tree.find_document(prefix)
    except doorstop.DoorstopError as error:
        sys.exit(f"error: {error}")

    tag_re = re.compile(rf"^@({re.escape(prefix)}\S+)$")
    features = [parse_feature(path, rel, tag_re) for path, rel in repo_files(root, {".feature"})]
    errors = [error for feature in features for error in feature.errors]

    by_uid: dict[str, list[tuple[Feature, Scenario]]] = {}
    for feature in features:
        for scenario in feature.scenarios:
            if not scenario.uids:
                errors.append(
                    f"{feature.where(scenario.line)}: scenario '{scenario.name}' has no "
                    f"@{prefix} tag"
                )
            elif len(scenario.uids) > 1:
                errors.append(
                    f"{feature.where(scenario.line)}: scenario '{scenario.name}' has several "
                    f"{prefix} tags: {', '.join(scenario.uids)}"
                )
            else:
                by_uid.setdefault(scenario.uids[0], []).append((feature, scenario))

    for uid, found in sorted(by_uid.items()):
        if len(found) > 1:
            errors.append(
                f"@{uid} is used on {len(found)} scenarios: "
                + ", ".join(feature.where(scenario.line) for feature, scenario in found)
            )

    items = {str(item.uid): item for item in document.items if item.active}
    scenario_items = {uid: item for uid, item in items.items() if item.normative}
    heading_items = {uid: item for uid, item in items.items() if not item.normative}

    for uid in sorted(by_uid.keys() - scenario_items.keys()):
        feature, scenario = by_uid[uid][0]
        errors.append(f"{feature.where(scenario.line)}: @{uid} has no item in {prefix}")
    for uid in sorted(scenario_items.keys() - by_uid.keys()):
        errors.append(f"{uid}: no scenario is tagged @{uid}")

    def referenced_paths(item) -> list[str]:
        return [str(reference.get("path", "")) for reference in item.references or []]

    headings: dict[str, Any] = {}
    for feature in features:
        found = [
            item for item in heading_items.values() if str(feature.rel) in referenced_paths(item)
        ]
        if not found:
            errors.append(
                f"{feature.rel}: no heading item refers to this feature file; add one with "
                f"`uvx --from doorstop doorstop add {prefix}`, set `normative: false` and let it "
                f"refer to the file"
            )
        elif len(found) > 1:
            errors.append(
                f"{feature.rel}: {len(found)} heading items refer to this feature file: "
                + ", ".join(sorted(str(item.uid) for item in found))
            )
        else:
            headings[str(feature.rel)] = found[0]

    claimed = {str(item.uid) for item in headings.values()}
    for uid in sorted(heading_items.keys() - claimed):
        errors.append(f"{uid}: refers to no feature file, but is a heading in {prefix}")

    scenarios = {}
    for uid, found in by_uid.items():
        if len(found) == 1 and uid in scenario_items:
            feature, scenario = found[0]
            item = scenario_items[uid]
            if str(feature.rel) not in referenced_paths(item):
                errors.append(
                    f"{uid}: refers to {referenced_paths(item) or 'nothing'}, but its scenario is "
                    f"in {feature.rel}"
                )
            scenarios[uid] = (item, feature, scenario)

    return Mapping(features=features, errors=errors, headings=headings, scenarios=scenarios)


def differences(mapping: Mapping) -> list[tuple[Any, str, dict[str, Any]]]:
    """What each item should hold: (item, what it is, {attribute: wanted value})."""
    wanted: list[tuple[Any, str, dict[str, Any]]] = []
    for index, feature in enumerate(mapping.features, start=1):
        heading = mapping.headings.get(str(feature.rel))
        if heading is not None:
            wanted.append(
                (
                    heading,
                    f"heading of {feature.rel}",
                    {
                        "level": f"{index}.0",
                        "header": feature.name,
                        "text": feature.heading_text(),
                    },
                )
            )
        for position, scenario in enumerate(feature.scenarios, start=1):
            entry = mapping.scenarios.get(scenario.uids[0] if scenario.uids else "")
            if entry is None:
                continue
            item, _, _ = entry
            wanted.append(
                (
                    item,
                    f"scenario at {feature.where(scenario.line)}",
                    {
                        "level": f"{index}.{position}",
                        "text": feature.scenario_text(scenario),
                        HASH_ATTR: scenario.hash(),
                    },
                )
            )

    stale = []
    for item, what, values in wanted:
        differing = {
            name: value for name, value in values.items() if current(item, name) != value
        }
        if differing:
            stale.append((item, what, differing))
    return stale


def current(item, name: str) -> Any:
    if name == "level":
        return str(item.level)
    if name == "text":
        return str(item.text)
    if name == "header":
        return str(item.header or "")
    return item.get(name) or ""


def apply(item, name: str, value: Any) -> None:
    if name == "level":
        item.level = value
    elif name == "text":
        item.text = value
    elif name == "header":
        item.header = value
    else:
        item.set(name, value)


HEADING_ANCHOR = re.compile(r"\{#([A-Za-z]+-?\d+)\}")
LINK_FRAGMENT = re.compile(r"\]\(([^)#]*)#([^)]+)\)")


FENCE = re.compile(r"^\s*```")
ATX_HEADING = re.compile(r"^(#{1,5}) ")


def demote_headings(markdown: str) -> str:
    """Push every heading down one level, leaving fenced blocks alone.

    The page's own `h1` is its front matter title, so a document that starts at `h1` gives the page
    two of them. It also matters for navigation: the theme only anchors `h2` and below, so a group
    heading left at `h1` cannot be linked to at all.
    """
    lines, fenced = [], False
    for line in markdown.splitlines():
        if FENCE.match(line):
            fenced = not fenced
        elif not fenced and ATX_HEADING.match(line):
            line = "#" + line
        lines.append(line)
    return "\n".join(lines) + "\n"


def hugo_anchors(markdown: str, prefixes: list[str]) -> str:
    """Rewrite Doorstop's anchors into the ones the site actually renders.

    Doorstop writes its links for GitHub, whose heading slugs are built from the whole heading text:
    `#12-req-018-req-018`. Hugo instead uses the explicit `{#REQ-018}` attribute, and the theme
    lowercases it when it builds the "On this page" list — so out of the box the document's own table
    of contents, the page sidebar, and the links between the documents all point at anchors that do
    not exist. Lowercasing the anchor and addressing every link by UID makes the three agree.
    """
    uid_pattern = re.compile(rf"({'|'.join(prefixes)})-?(\d+)$", re.IGNORECASE)

    def uid_anchor(fragment: str) -> str | None:
        match = uid_pattern.search(fragment)
        return f"{match.group(1).lower()}-{match.group(2)}" if match else None

    def heading(match: re.Match) -> str:
        return f"{{#{match.group(1).lower()}}}"

    def link(match: re.Match) -> str:
        path, fragment = match.group(1), match.group(2)
        anchor = uid_anchor(fragment)
        return f"]({path}#{anchor or fragment})"

    return LINK_FRAGMENT.sub(link, HEADING_ANCHOR.sub(heading, markdown))


def publish_documents(root: Path, content: Path, static: Path) -> None:
    """Render the Doorstop documents into the site: a page per document, plus the standalone site."""
    from doorstop.core import publisher

    from project_tasks.context import note

    tree, _ = build_tree()
    content_dir = root / content
    content_dir.mkdir(parents=True, exist_ok=True)

    # Parents before children, so that the navigation lists the requirements above the test cases.
    documents = sorted(tree.documents, key=lambda document: str(document.parent or ""))
    prefixes = [str(document.prefix) for document in documents]
    for weight, document in enumerate(documents, start=1):
        prefix = str(document.prefix)
        target = content_dir / f"{prefix}.md"
        # No `toc`: the theme builds "On this page" from the headings, so Doorstop's own table of
        # contents would only repeat it.
        publisher.publish(document, str(target), ".md", linkify=True, toc=False)
        title = DOCUMENT_TITLES.get(prefix, prefix)
        # Doorstop writes no front matter; without one the page would be called after its first
        # heading.
        body = demote_headings(hugo_anchors(target.read_text(), prefixes))
        target.write_text(f"---\ntitle: {title}\nweight: {weight}\n---\n\n" + body)
        note(f"{prefix} -> {target.relative_to(root)}")

    publisher.publish(tree, str(root / static), ".html", index=True, matrix=True)
    note(f"Doorstop site (with the traceability matrix) -> {static}")
