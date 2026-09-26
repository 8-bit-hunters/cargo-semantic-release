"""The ADRs: their status, and the Doorstop items and other ADRs they cite."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from project_tasks.gherkin import repo_files

CODE_SUFFIXES = {".rs", ".py", ".feature"}
ADR_FILE_RE = re.compile(r"^(\d{4}|XXXX)-.+\.md$")
ADR_CITE_RE = re.compile(r"\bADR-(\d{4})\b")
STATUS_RE = re.compile(r"^\*\s*Status:\s*(\w+)", re.MULTILINE)


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
