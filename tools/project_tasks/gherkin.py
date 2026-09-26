"""Reading the Gherkin feature files, which are the source of truth for the test items."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterator

from gherkin.parser import Parser

SKIP_DIRS = {".git", ".jj", "target", "node_modules", ".venv", "public"}


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
