# Use Doorstop for requirements management

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-26

Technical Story: Introduce basic requirement management to the project

## Context and Problem Statement

The project records its architectural decisions as ADRs and tracks the work in progress in
development notes, but it has no record of what the tool is actually required to do. The goals in
the README are informal prose, the behaviour of the commands is documented only as usage text,
and the workspace features are described as implementation phases rather than as requirements.

Without such a record there is nothing to check an implementation against, no way to tell which
behaviour is covered by a test and which is merely assumed, and no place where a new requirement
can be stated before code is written.

How and where should requirements be recorded, so that they live with the code, can be reviewed
like code, and can be traced to the tests that verify them?

## Decision Drivers

* Requirements should live in the repository, in the same review flow as the code
* Traceability between a requirement and the test that covers it should be checkable by a machine
* Low ceremony: the project is small and maintained by few people
* No new runtime dependency for the crate itself
* Text-based and diffable, so a requirement change is visible in a pull request
* Should not duplicate what the ADRs already record

## Considered Options

* [Doorstop](https://doorstop.readthedocs.io/) — requirements management on top of version control
* Hand-written Markdown requirements in `docs/`
* GitHub issues and milestones as the only record
* No formal requirements, relying on the README and the ADRs

## Decision Outcome

Chosen option: "Doorstop", because it is the only option that keeps requirements as reviewable
files in the repository *and* validates traceability mechanically, at a ceremony level the project
can carry.

Two documents are created:

| Document | Location | Item format | Content |
|----------|----------|-------------|---------|
| `REQ` | `plm/req/` | `markdown` | What the tool must do, as prose |
| `TST` | `plm/tst/` | `yaml` | One item per verified behaviour, child of the requirement it covers |

Both documents live under `plm/`, so that the product lifecycle records have one home of their own
and further documents can join them later without crowding the repository root.

A deliberately shallow tree: no separate design or specification document is created, because the
ADRs already record design decisions with their own IDs and their own lifecycle.

The item format is chosen per document, following who writes the items. `REQ` items are prose written
by hand, so they use Doorstop's `markdown` format and read as documents in a diff. Nothing in a `TST`
item is written by hand — its text and its scenario hash are generated from the Gherkin scenario it
mirrors (see [ADR-0008](0008-trace-requirements-to-gherkin-scenarios.md)) — so it uses Doorstop's
default `yaml` format, which is what a generator writes and a reader rarely opens.

The two record sets refer to each other in one direction only: an ADR names the requirements it
serves among its decision drivers, and a requirement says nothing about the decisions taken to
satisfy it. A requirement states what the tool must do and outlives any particular decision about
how; a decision, in contrast, is only understandable against the requirement that motivated it. A
requirement that named its ADRs would also have to be revisited whenever one of them is superseded,
which is exactly the coupling this avoids. The direction is checked rather than trusted: an ADR naming
a UID that is not an active item fails the `Requirements` workflow.

Doorstop is a Python tool, run through `uvx` as the project already runs `pre-commit` and `pyadr`.
Nothing is added to `Cargo.toml`, and contributors need no permanent Python installation. The helper
scripts around this workflow are Python scripts carrying inline
[script metadata](https://packaging.python.org/en/latest/specifications/inline-script-metadata/) and
run with `uv run`, rather than shell scripts, so they declare their own dependencies and behave the
same on every platform. Their command line is built with
[cyclopts](https://cyclopts.readthedocs.io/), which derives the parameters and the help text from the
function's signature and docstring — a script's interface is then described in one place instead of
two.

Validation runs in CI, in a `Requirements` workflow, and not as a pre-commit hook: the project
supports committing with `jj`, where git hooks do not fire, so CI is the only gate that holds for
every contributor.

The workflow reports first and gates second. A plain `doorstop` run prints everything, including the
requirements that no scenario covers yet; then `doorstop -C -e` turns what is left into errors, so an
unreviewed change or a suspect link fails the build while a missing scenario stays advisory. Coverage
is a state of the project and says nothing about whether this change is sound; an unreviewed
fingerprint is a step someone skipped in this change.

### Positive Consequences

* A requirement change is reviewed in the same pull request as the code that implements it
* Requirements that no behaviour covers are visible, both in CI output and in the published
  traceability matrix
* The requirements are published with the rest of the documentation, so they are readable without
  cloning the repository
* Stating the workspace features as requirements gives the unimplemented phases in the
  development notes a definition of done
* An ADR can be superseded without touching any requirement, since nothing in `plm/` points at it

### Negative Consequences

* A second tool, and a second ID space, to learn alongside the ADRs
* Doorstop maintains a review fingerprint per item, so editing a requirement is a two-step job:
  change it, then `doorstop review` it and commit the fingerprint — and because the gate enforces it,
  a forgotten review fails CI rather than passing quietly
* Doorstop rewrites its item files on every validation run and writes them without a trailing
  newline, so `plm/` has to be excluded from the whitespace-fixing pre-commit hooks
* Python tooling becomes part of the contributor workflow, where previously it was optional

## Pros and Cons of the Options

### Doorstop

* Good, because requirements are plain files in the repository, versioned with the code
* Good, because it validates the link graph and external references, so traceability cannot rot
  silently
* Good, because it publishes to HTML and Markdown, which fits the existing Hugo documentation site
* Good, because items carry a review fingerprint, so a changed requirement flags the items that
  depended on it
* Bad, because it adds a Python dependency to a Rust project's workflow
* Bad, because its review and fingerprint mechanics are an ongoing cost per edit

### Hand-written Markdown requirements in `docs/`

* Good, because it needs no tooling at all
* Good, because it publishes to the documentation site with no extra step
* Bad, because nothing checks that a requirement is covered by a test, or that a reference still
  resolves
* Bad, because IDs and cross-references are maintained by hand and rot quietly

### GitHub issues and milestones

* Good, because it is already in use for planning the work
* Good, because it needs no local tooling
* Bad, because an issue describes a change, not a standing requirement, and is closed once the
  change is made
* Bad, because the record lives outside the repository and is not available offline or in a diff

### No formal requirements

* Good, because it is free
* Bad, because there is nothing to verify an implementation against, which is the problem this
  decision addresses

## Links

* Refined by [ADR-0008](0008-trace-requirements-to-gherkin-scenarios.md)
* Refined by [ADR-0009](0009-formulate-requirements-with-the-sophist-master-patterns.md)
