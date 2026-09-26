 # Trace requirements to Gherkin scenarios

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-26

Technical Story: Introduce basic requirement management to the project

## Context and Problem Statement

[ADR-0007](0007-use-doorstop-for-requirements-management.md) introduces a `TST` document whose
items are the behaviours that verify the requirements in `REQ`. That leaves the question of what a
test item actually contains, and how it connects to something that runs.

The behaviour of a command — given a repository in some state, when the command runs, then this
happens — is naturally expressed as a Gherkin scenario, and Gherkin scenarios can be executed
directly in Rust with [cucumber-rs](https://cucumber-rs.github.io/cucumber/current/). So the same
text can serve as the specification of a behaviour and as its test, provided there is exactly one
copy of it.

Where does that text live, and how is a Doorstop item tied to it?

## Decision Drivers

* One source of truth: a behaviour should not be stated twice and be allowed to disagree with
  itself
* The specification should be executable, so that a behaviour claimed to be covered really is
* Drift between an item and its scenario should be caught by a machine, in both directions
* Should stay readable to someone who has never used Doorstop

## Considered Options

* The scenario text lives only in the feature file; the item references it
* The scenario text lives in the item and the feature file is generated from it
* The scenario text is copied into both, kept in step by a comparison check
* Test items are plain prose and the Gherkin scenarios are unrelated to them

## Decision Outcome

Chosen option: "The scenario text lives only in the feature file; the item references it".

Each scenario in `tests/features/` is tagged with the UID of its `TST` item:

```gherkin
  @TST-003
  Scenario: Proposing a version after nothing but a correction
    Given the following changes were recorded since then
      | change       |
      | a correction |
    When a version is proposed for the next publication
    Then the proposed version is "1.0.1"
```

The item links to the requirement it verifies, refers to the feature file with the tag as the search
keyword, and holds the scenario's title and the scenario itself as its text. All of that text is
**generated** from the feature file by `scripts/trace_sync.py`; the item is stored in
Doorstop's default `yaml` format, since nothing in it is written by hand:

```yaml
level: '1.3'
links:
- REQ-020: 0LY7VhXydAhnYaj8oKVi6cVZyJ195MH-OY3aa5GbA7c=
references:
- keyword: '@TST-003'
  path: tests/features/version.feature
  type: file
text: |
  Proposing a version after nothing but a correction

  ```gherkin
  Scenario: Proposing a version after nothing but a correction
    Given the following changes were recorded since then
      | change       |
      | a correction |
    When a version is proposed for the next publication
    Then the proposed version is "1.0.1"
  ```

  _Generated from `tests/features/version.feature`; edit the feature file, not this item._
```

Carrying the scenario in the item is what makes the published `TST` document readable on its own:
Doorstop renders a reference as a location — `` `tests/features/version.feature` (line 32) `` — never as
content, so an item that held only a title would send every reader into the repository.

It also puts the behaviour inside the item's fingerprint. When the text was only a title, editing a
scenario changed what verifies a requirement while the item's `reviewed` hash stayed valid, and the
change passed unreviewed. Now Doorstop reports `unreviewed changes` on the item whose scenario moved,
which is the correct amount of friction: the behaviour behind a requirement changed and wants a look.

### How the test document is structured

The `TST` outline mirrors the feature files, not the requirements: each feature file is a
non-normative heading item at level `<n>.0`, carrying the `Feature:` name and referring to the file
itself, and its scenarios follow at `<n>.<m>` in the order they appear in the file.

Mirroring the `REQ` outline instead was considered and rejected. The relationship to the requirements
is already carried by `links`, and shown in the published traceability matrix; encoding it a second
time in `level` would leave two structures to keep in agreement with nothing checking that they
agree — the coupling this ADR set avoids elsewhere.

Because the feature files decide the levels, the titles and the scenario text, everything in an item
but its links and references is derived by `scripts/trace_sync.py`: `check` reports any difference and
fails, `sync` writes what the feature files imply. It deliberately never creates or deletes items, so
that Doorstop keeps assigning UIDs. It reads the feature files with the Gherkin parser rather than by
pattern matching, so `Rule:`, `Scenario Outline:`, doc strings and comments are understood rather than
guessed at, and it writes through Doorstop's own API rather than editing the item files as text.

Each scenario item also stores a `scenario_hash`: a hash of the scenario as the parser sees it — name,
tags, steps, tables, doc strings, examples, and the `Background:` it inherits. `plm/tst/.doorstop.yml`
lists that attribute under `attributes.reviewed`, so it is part of the item's fingerprint. It covers
what the item's own text cannot: the background lives on the heading item, so without the hash a change
to it would leave every scenario that inherits it looking untouched.

The heading item carries the `Feature:` name as its `header` — which Doorstop renders as the document
heading, so the body does not repeat it — and the feature's description and `Background:` as its text,
so the preconditions every scenario in the file shares are published once, where they belong.

### How a scenario is written

Two rules follow from making the scenario the specification, and both are about restraint.

**A title names the precondition and the action, never the outcome.** "Proposing a version after
nothing but a correction" says which situation is examined and what is asked for; whether the answer
is `1.0.1` is the body's business. A title that gave the outcome away would have to be rewritten
whenever the rule changed, and would read as a claim rather than as a case.

**A feature's description is a user story, and nothing more.** The `Feature:` block carries

```gherkin
  As the maintainer of a published work
  I want the number of its next publication decided from the record of changes made since the last one
  So that the number alone tells a reader what kind of change to expect
```

and, where it helps a newcomer, a sentence on what doing the job by hand looks like — never the rules
themselves. A description that restated the rules ("a break demands a new major version, an addition a
new minor one") would be duplicating REQ-018 … REQ-021 in a place nothing verifies: change a rule, and
the scenarios and the requirements get updated while the description quietly starts lying. A story
states the need instead of the mechanism, so it cannot go stale that way.

This is the one place in the requirement set where a user story belongs. [ADR-0009](0009-formulate-requirements-with-the-sophist-master-patterns.md)
rejects the form for requirement items because a story cannot be verified — and a feature description
is not verified by anything either, which is precisely why the form fits here.

**A scenario is written in the language of the work being published, not of the tool.** Almost
everything this tool does, a clerk could once have done by hand: read the record of changes made
since a work was last published, and number the next publication accordingly. So the scenarios are
written as that job — changes are *recorded*, a version is *proposed*, a publication is *marked* —
with no mention of git, Cargo, commit messages, emoji, files, flags, or output. Those appear only in
the step definitions, which translate a phrase such as "a change that breaks how the work is used"
into the Gitmoji commit that expresses it.

That restraint is what keeps the scenarios usable as requirements: they survive a change of
interface, they can be read by someone who does not know the tool, and a behaviour that cannot be
stated without naming a flag is a sign that it has not been understood yet.

That closes the loop in both directions. Doorstop reports an error when an item's keyword is no
longer found in the referenced file, which covers a renamed, retagged, or deleted scenario. The
opposite case — a scenario tagged with a UID that has no item — is covered by
`scripts/trace_sync.py`, which runs alongside Doorstop in the `Requirements` workflow.

The scenarios are executed by cucumber-rs as the `features` test target, driving the real binary
in a throwaway git repository built with the existing `test_util` helpers. The unit tests and
`tests/integration.rs` are unaffected: the executable specification covers behaviour at the
command level, not the internals.

### Positive Consequences

* A behaviour is written once, in the form that both documents it and tests it
* A requirement is only claimed as covered if a scenario actually runs and passes
* Scenario names read as requirements in the published traceability matrix
* The published test document carries the scenarios themselves, so it can be read without the
  repository at hand
* The Given/When/Then steps are reusable, so covering a further behaviour is often a matter of
  writing a scenario with no new Rust code
* Scenarios written in the language of the work outlive changes to the command line, and read as
  requirements to someone who has never used the tool

### Negative Consequences

* UIDs are baked into the feature files as tags, so items cannot be renumbered freely
* Item files are generated, not written: editing one by hand is undone by the next `sync`, which the
  generated note in each item says out loud
* A scenario edit is a three-step job: change the feature file, run `sync`, run `doorstop review`
* Keeping technology out of the scenarios puts a translation layer in the step definitions, so a new
  vocabulary word costs a step definition change rather than only a line of Gherkin
* A new dependency on `cucumber` and `futures` as dev-dependencies, and a test target with
  `harness = false`

## Pros and Cons of the Options

### The scenario text lives only in the feature file (chosen)

* Good, because there is exactly one copy of the behaviour, and it is the one that runs
* Good, because both directions of drift are caught mechanically
* Bad, because the published test document is thin on its own

### The scenario text lives in the item, the feature file is generated

* Good, because the published document is self-contained
* Good, because there is a single source of truth
* Bad, because it needs a generator and a check that the generated files are up to date
* Bad, because generated files in `tests/` are confusing to edit and easy to edit by mistake

### The scenario text is copied into both, by hand

* Good, because the published document is self-contained with no generator
* Bad, because the two copies can disagree, and only a text-comparison check keeps them honest
* Bad, because every edit has to be made twice

### The scenario text is generated into the item from the feature file (chosen, as a refinement)

This was originally rejected along with the hand-made copy above, on the grounds that a copy has to be
maintained twice. Generation removes that objection: the edit is made once, in the feature file, and
`scripts/trace_sync.py sync` writes the item. The check runs in CI, so a difference cannot
survive a pull request.

* Good, because the published document carries the behaviour, which Doorstop's references cannot
* Good, because an edit is still made in exactly one place
* Good, because the scenario is part of the item's fingerprint, so a changed behaviour asks to be
  reviewed
* Bad, because the item files are no longer hand-editable, which has to be said in them
* Bad, because a scenario edit now costs a `sync` and a `doorstop review`

### Test items are plain prose, unrelated to the scenarios

* Good, because it needs no tooling beyond Doorstop
* Bad, because "covered by a test" becomes a claim nothing verifies, which defeats the purpose of
  the `TST` document

## Links

* Refines [ADR-0007](0007-use-doorstop-for-requirements-management.md)
