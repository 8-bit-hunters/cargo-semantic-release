#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["cyclopts>=5.0", "doorstop>=3.2", "gherkin-official>=29"]
# ///
"""Keep the Doorstop test items in step with the tagged Gherkin scenarios, and check ADR citations.

Run from the repository root:

    uv run scripts/trace_sync.py check           # CI: verify the mapping, the items and the ADRs
    uv run scripts/trace_sync.py sync            # write what the feature files imply
    uv run scripts/trace_sync.py impact REQ-018  # which ADRs cite a requirement

The feature files are the source of truth. For each one, in file-name order, a heading item and one
item per tagged scenario are expected in `plm/tst`:

- the heading item is non-normative, sits at level `<n>.0`, refers to the feature file without a
  keyword, and carries the `Feature:` name as its header and the description and `Background:` as its
  text;
- a scenario item sits at level `<n>.<m>` in scenario order, refers to the feature file with its own
  `@TST-NNN` tag as the keyword, and carries the scenario's title and the scenario itself as its text.

Each scenario item also stores a `scenario_hash`: a hash of the scenario as the Gherkin parser sees it
— its name, tags, steps, tables, doc strings, examples and the background it inherits. Because
`plm/tst/.doorstop.yml` lists that attribute under `attributes.reviewed`, a changed scenario changes
the item's fingerprint, and Doorstop then asks for the item to be reviewed. The hash covers what the
item's own text cannot: a change to the `Background:` reaches every scenario in the file.

`sync` never creates or deletes items, so that Doorstop keeps assigning UIDs.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Annotated, Any, Iterator

import cyclopts
import doorstop
from doorstop.core import publisher
from gherkin.parser import Parser

HASH_ATTR = "scenario_hash"
SKIP_DIRS = {".git", ".jj", "target", "node_modules", ".venv", "public"}
CODE_SUFFIXES = {".rs", ".py", ".feature"}
ADR_FILE_RE = re.compile(r"^(\d{4}|XXXX)-.+\.md$")
ADR_CITE_RE = re.compile(r"\bADR-(\d{4})\b")
STATUS_RE = re.compile(r"^\*\s*Status:\s*(\w+)", re.MULTILINE)

app = cyclopts.App(
    name="trace_sync",
    help="Keep the Doorstop test items in step with the Gherkin scenarios, and check ADR citations.",
)

Prefix = Annotated[str, cyclopts.Parameter(help="Doorstop document holding the test items.")]
AdrDir = Annotated[Path, cyclopts.Parameter(help="Directory holding the ADRs.")]

DEFAULT_PREFIX = "TST"
DEFAULT_ADR_DIR = Path("docs/content/development/adr")
DEFAULT_CONTENT_DIR = Path("docs/content/development/requirements")
DEFAULT_STATIC_DIR = Path("docs/static/requirements")
# What each document is called in the site's navigation.
DOCUMENT_TITLES = {"REQ": "Requirements", "TST": "Test cases"}


def repo_files(root: Path, suffixes: set[str]) -> Iterator[tuple[Path, Path]]:
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root)
        if path.suffix in suffixes and path.is_file() and not SKIP_DIRS.intersection(rel.parts):
            yield path, rel


def dedent(lines: list[str]) -> str:
    """Strip the common indent from a block of lines, dropping blank lines at either end."""
    lines = list(lines)
    while lines and not lines[0].strip():
        lines.pop(0)
    while lines and not lines[-1].strip():
        lines.pop()
    if not lines:
        return ""
    indent = min(len(line) - len(line.lstrip()) for line in lines if line.strip())
    return "\n".join(line[indent:].rstrip() for line in lines)


def fenced(block: str) -> str:
    return f"```gherkin\n{block}\n```"


# ---------------------------------------------------------------- Gherkin


def canonical_steps(steps: list[dict]) -> list[dict]:
    out = []
    for step in steps:
        canonical: dict[str, Any] = {"keyword": step["keyword"].strip(), "text": step["text"]}
        if table := step.get("dataTable"):
            canonical["table"] = [[c["value"] for c in row["cells"]] for row in table["rows"]]
        if doc := step.get("docString"):
            canonical["doc"] = doc["content"]
        out.append(canonical)
    return out


def canonical_examples(examples: list[dict]) -> list[dict]:
    out = []
    for example in examples:
        header = example.get("tableHeader")
        out.append(
            {
                "name": example.get("name", ""),
                "header": [c["value"] for c in header["cells"]] if header else [],
                "rows": [[c["value"] for c in row["cells"]] for row in example.get("tableBody", [])],
            }
        )
    return out


@dataclass
class Scenario:
    uids: list[str]
    name: str
    line: int
    body: str
    canonical: dict

    def hash(self) -> str:
        blob = json.dumps(
            self.canonical, sort_keys=True, ensure_ascii=False, separators=(",", ":")
        )
        return hashlib.sha256(blob.encode()).hexdigest()


@dataclass
class Feature:
    """One feature file, as the Gherkin parser sees it plus the source lines it came from."""

    path: Path
    rel: Path
    name: str = ""
    description: str = ""
    background: str = ""
    scenarios: list[Scenario] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)

    def where(self, line: int) -> str:
        return f"{self.rel}:{line}"

    def heading_text(self) -> str:
        """The heading item's body: the description and the background, never the name.

        The name is the item's header, which Doorstop renders as the heading; repeating it in the text
        would print it twice.
        """
        parts = [part for part in (self.description, fenced(self.background) if self.background else "") if part]
        parts.append(self.generated_note())
        return "\n\n".join(parts)

    def scenario_text(self, scenario: Scenario) -> str:
        return "\n\n".join([scenario.name, fenced(scenario.body), self.generated_note()])

    def generated_note(self) -> str:
        return f"_Generated from `{self.rel}`; edit the feature file, not this item._"


def parse_feature(path: Path, rel: Path, tag_re: re.Pattern) -> Feature:
    """Parse one feature file: its name, description, background, and its tagged scenarios."""
    feature = Feature(path=path, rel=rel)
    lines = path.read_text(encoding="utf-8").splitlines()
    parsed = Parser().parse(path.read_text(encoding="utf-8")).get("feature")
    if not parsed:
        return feature

    feature.name = parsed.get("name", "")
    feature.description = dedent(parsed.get("description", "").splitlines())

    def uids_in(node: dict) -> list[str]:
        return [m.group(1) for tag in node.get("tags", []) if (m := tag_re.match(tag["name"]))]

    def forbid(node: dict, what: str) -> None:
        for uid in uids_in(node):
            feature.errors.append(
                f"{feature.where(node['location']['line'])}: @{uid} is on a {what}; "
                "put test tags on scenarios only"
            )

    # Every line that starts a block, so that a block's source ends where the next one begins.
    anchors: set[int] = set()

    def note_anchors(node: dict) -> None:
        anchors.add(node["location"]["line"])
        for tag in node.get("tags", []):
            anchors.add(tag["location"]["line"])

    def walk_anchors(children: list[dict]) -> None:
        for child in children:
            for key in ("background", "scenario", "rule"):
                if key in child:
                    note_anchors(child[key])
                    if key == "rule":
                        walk_anchors(child[key].get("children", []))

    walk_anchors(parsed.get("children", []))

    def source_of(line: int) -> str:
        end = min((anchor for anchor in anchors if anchor > line), default=len(lines) + 1)
        return dedent(lines[line - 1 : end - 1])

    forbid(parsed, "Feature")

    def walk(children: list[dict], background_steps: list[dict]) -> None:
        for child in children:
            if "background" in child:
                node = child["background"]
                background_steps = background_steps + canonical_steps(node.get("steps", []))
                feature.background = source_of(node["location"]["line"])
            elif "rule" in child:
                node = child["rule"]
                forbid(node, "Rule")
                walk(node.get("children", []), background_steps)
            elif "scenario" in child:
                node = child["scenario"]
                for example in node.get("examples", []):
                    forbid(example, "Examples block")
                other_tags = sorted(
                    tag["name"] for tag in node.get("tags", []) if not tag_re.match(tag["name"])
                )
                line = node["location"]["line"]
                feature.scenarios.append(
                    Scenario(
                        uids=uids_in(node),
                        name=node["name"],
                        line=line,
                        body=source_of(line),
                        canonical={
                            "name": node["name"],
                            "tags": other_tags,
                            "background": background_steps,
                            "steps": canonical_steps(node.get("steps", [])),
                            "examples": canonical_examples(node.get("examples", [])),
                        },
                    )
                )

    walk(parsed.get("children", []), [])
    return feature


# ---------------------------------------------------------------- Doorstop items


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


# ---------------------------------------------------------------- ADRs


@dataclass
class Adr:
    number: str  # "0002", or "XXXX" while proposed
    rel: Path
    status: str
    cites: list[str]  # Doorstop UIDs named anywhere in the ADR


def load_adrs(tree, root: Path, adr_dir: Path) -> list[Adr]:
    prefixes = [re.escape(document.prefix) for document in tree.documents]
    uid_re = re.compile(rf"\b(?:{'|'.join(prefixes)})-?\d+\b") if prefixes else None
    adrs = []
    directory = root / adr_dir
    for path in sorted(directory.glob("*.md")) if directory.is_dir() else []:
        if not (match := ADR_FILE_RE.match(path.name)):
            continue  # template.md, index.md, ...
        text = path.read_text(encoding="utf-8")
        status = match_status.group(1).lower() if (match_status := STATUS_RE.search(text)) else "unknown"
        cites = sorted(set(uid_re.findall(text))) if uid_re else []
        adrs.append(Adr(match.group(1), path.relative_to(root), status, cites))
    return adrs


def check_adrs(tree, root: Path, adrs: list[Adr]) -> list[str]:
    errors = []
    active = {str(item.uid) for document in tree.documents for item in document.items if item.active}
    for adr in adrs:
        for uid in adr.cites:
            if uid not in active:
                errors.append(f"{adr.rel}: cites {uid}, which is not an active Doorstop item")

    by_number = {adr.number: adr for adr in adrs if adr.number != "XXXX"}
    for path, rel in repo_files(root, CODE_SUFFIXES):
        for number, line in (
            (number, lineno)
            for lineno, text in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1)
            for number in ADR_CITE_RE.findall(text)
        ):
            adr = by_number.get(number)
            if adr is None:
                errors.append(f"{rel}:{line}: cites ADR-{number}, which does not exist")
            elif adr.status != "accepted":
                errors.append(
                    f"{rel}:{line}: cites ADR-{number}, which is {adr.status} ({adr.rel})"
                )
    return errors


# ---------------------------------------------------------------- commands


def build_tree() -> tuple[Any, Path]:
    tree = doorstop.build()
    return tree, Path(getattr(tree, "root", None) or Path.cwd())


@app.command
def check(
    *,
    prefix: Prefix = DEFAULT_PREFIX,
    adr_dir: AdrDir = DEFAULT_ADR_DIR,
) -> int:
    """Verify the scenario mapping, the items and the ADR citations. Never writes."""
    tree, root = build_tree()
    mapping = collect(tree, root, prefix)
    errors = list(mapping.errors)
    errors += check_adrs(tree, root, load_adrs(tree, root, adr_dir))

    for item, what, differing in differences(mapping):
        errors.append(
            f"{item.uid}: {', '.join(sorted(differing))} out of date with the {what}; run "
            f"`uv run scripts/trace_sync.py sync`"
        )

    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if errors:
        return 1

    print(
        f"ok: {len(mapping.scenarios)} scenario(s) in {len(mapping.features)} feature file(s) "
        f"mapped and in sync, ADR citations valid"
    )
    return 0


@app.command
def sync(*, prefix: Prefix = DEFAULT_PREFIX) -> int:
    """Write the levels, headers, text and scenario hashes the feature files imply."""
    tree, root = build_tree()
    mapping = collect(tree, root, prefix)
    if mapping.errors:
        for error in mapping.errors:
            print(f"error: {error}", file=sys.stderr)
        print("nothing written: fix the mapping errors first", file=sys.stderr)
        return 1

    changed = []
    for item, what, differing in differences(mapping):
        for name, value in differing.items():
            apply(item, name, value)
        item.save()
        changed.append(item)
        print(f"updated {item.uid} ({', '.join(sorted(differing))}) from the {what}")

    if not changed:
        print(f"ok: {len(mapping.scenarios)} scenario(s) already in sync")
        return 0

    print("\nreview the changed items:")
    print("  uvx --from doorstop doorstop review " + " ".join(str(item.uid) for item in changed))
    return 0


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


@app.command
def publish(
    *,
    content: Annotated[Path, cyclopts.Parameter(help="Section directory for the document pages.")] = DEFAULT_CONTENT_DIR,
    static: Annotated[Path, cyclopts.Parameter(help="Directory for the standalone Doorstop site.")] = DEFAULT_STATIC_DIR,
) -> int:
    """Publish the Doorstop documents into the documentation site.

    Writes one Markdown page per document into the site's requirements section, so that the
    requirements are browsable and searchable like any other page, and the standalone Doorstop site —
    which carries the traceability matrix — into the site's static directory.
    """
    tree, root = build_tree()
    content_dir = root / content
    content_dir.mkdir(parents=True, exist_ok=True)

    # Parents before children, so that the navigation lists the requirements above the test cases.
    documents = sorted(tree.documents, key=lambda document: str(document.parent or ""))
    for weight, document in enumerate(documents, start=1):
        prefix = str(document.prefix)
        target = content_dir / f"{prefix}.md"
        # No `toc`: the theme builds "On this page" from the headings, so Doorstop's own table
        # of contents would only repeat it.
        publisher.publish(document, str(target), ".md", linkify=True, toc=False)
        title = DOCUMENT_TITLES.get(prefix, prefix)
        # Doorstop writes no front matter; without one the page would be called after its first
        # heading, which is "Table of Contents" for every document.
        body = demote_headings(hugo_anchors(target.read_text(), [str(d.prefix) for d in documents]))
        target.write_text(f"---\ntitle: {title}\nweight: {weight}\n---\n\n" + body)
        print(f"published {prefix} to {target.relative_to(root)}")

    publisher.publish(tree, str(root / static), ".html", index=True, matrix=True)
    print(f"published the Doorstop site to {static}")
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


if __name__ == "__main__":
    sys.exit(app())
