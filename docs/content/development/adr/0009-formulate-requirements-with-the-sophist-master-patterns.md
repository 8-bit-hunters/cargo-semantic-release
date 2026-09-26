# Formulate requirements with the SOPHIST MASTeR patterns

* Status: accepted
* Deciders: Kristof Kovacs
* Date: 2026-09-26

Technical Story: Introduce basic requirement management to the project

## Context and Problem Statement

[ADR-0007](0007-use-doorstop-for-requirements-management.md) establishes where requirements live, and
[ADR-0008](0008-trace-requirements-to-gherkin-scenarios.md) how they are verified. Neither says how a
requirement sentence is written.

The first set of requirements was written as free prose: readable, but inconsistent about who acts,
which obligations are binding, and how strongly. Several items quietly bundled four behaviours into
one sentence — "shall write the version, commit the change, tag the commit, and push both to origin"
— which cannot be verified as a unit, and cannot be traced to a single scenario.

Which sentence structure should a requirement follow?

## Decision Drivers

* A requirement must be verifiable as one statement, so that a scenario can cover it exactly
* The actor must be named: it should never be in doubt who does the thing
* The strength of an obligation must be explicit and consistent
* Conditional behaviour must state its negative case, so that gaps are visible rather than assumed
* The structure must be learnable in minutes by a contributor who has never met it

## Considered Options

* SOPHIST MASTeR sentence patterns with the SOPHIST review rules
* User stories: "As a …, I want …, so that …"
* EARS (Easy Approach to Requirements Syntax)
* Free prose with a `shall` convention

## Decision Outcome

Chosen option: "SOPHIST MASTeR sentence patterns with the SOPHIST review rules".

Every normative requirement follows:

```
[<condition>] <system> <obligation> [<type of functionality>] <process verb> <object>
```

with `cargo-semantic-release` as the system, and one of three functionality types:

| Type | Form | Used when |
|------|------|-----------|
| Autonomous | `shall <verb>` | the tool acts by itself |
| User interaction | `shall provide <whom> with the ability to <verb>` | a person drives the action |
| Interface | `shall be able to <verb>` | something outside the tool drives it |

A requirement that is not functional takes the matching MASTeR template instead: **PropertyMASTeR**
for a measurable quality, **EnvironmentMASTeR** (`shall be designed so that it can be …`) for an
operating or toolchain constraint, and **ProcessMASTeR** for an activity or a deliverable, whose
subject is a person or an organization rather than the tool.

Obligations are **SHALL** (mandatory), **SHOULD** (wish), and **WILL** (future purpose, to be
considered but not tested). Conditions are written `IF <logical expression>`, `AS SOON AS <event>`, or
`AS LONG AS <time period>`. Two actors appear: *the maintainer*, who releases a crate, and *the
contributor*, who writes the commit messages a release is derived from.

Every sentence is then checked against the ten SOPHIST review rules, of which three shaped the set
most: one main verb per sentence (#2), every conditional behaviour also needs its negative case (#6),
and exceptional behaviour is specified as well as the standard path (#10).

User stories were considered and rejected for the requirement items themselves. A story is a
placeholder for a conversation, negotiable and sized for an iteration; it is not verifiable, and
several of these requirements have no persona that is not invented — nobody "wants" the baseline to be
the latest release tag. The motivation a story carries in its *so that* clause is kept instead on the
non-normative heading item of each group, where the reader meets it before the requirements it
explains, and where Doorstop's `normative: false` already says "this is context, not a testable
statement".

### Positive Consequences

* Each requirement is one verifiable statement, so a scenario can trace to exactly one of them
* Applying the rules surfaced behaviour nobody had written down: that the next version is reported at
  all, that skipping the commit also skips the release tag, that a failed push is reported, that an
  undo can be previewed
* Reviewing a requirement becomes mechanical — the rules name the defect and its signal words
* The published document reads uniformly, rather than in as many voices as it had authors

### Negative Consequences

* The set grew from 14 prose items to 56 normative ones; the outline, not the item count, is now what
  makes it readable
* Most of those have no scenario yet, so Doorstop's coverage warnings grew with them
* MASTeR prose is deliberately stilted; "shall provide the maintainer with the ability to" is precise
  and not elegant
* A contributor has to learn the pattern before adding a requirement

## Pros and Cons of the Options

### SOPHIST MASTeR with the review rules

* Good, because the pattern makes the actor, the obligation and the condition impossible to omit
* Good, because the review rules turn "is this a good requirement?" into a checklist
* Good, because atomic sentences map one-to-one onto scenarios, which is what the `TST` document needs
* Bad, because the prose is stilted and the item count is high
* Bad, because it is a method to learn, with its own vocabulary

### User stories

* Good, because they carry the motivation and the beneficiary in the sentence itself
* Good, because they are familiar to anyone who has worked from a backlog
* Bad, because a story is not verifiable: "I want X" has no pass or fail
* Bad, because internal rules have no honest persona, so the actor gets invented
* Bad, because stories are sized for an iteration, while a requirement outlives one

### EARS

* Good, because its five patterns are simpler to learn than MASTeR
* Good, because it handles conditional and optional behaviour explicitly
* Bad, because it has no counterpart to MASTeR's functionality types, so the distinction between what
  the tool does by itself and what it offers a person to do would be lost
* Bad, because it comes without a review ruleset

### Free prose with a `shall` convention

* Good, because it needs nothing to learn and reads naturally
* Bad, because it is what the first set already was: inconsistent actors, bundled verbs, missing
  negative cases

## Links

* Refines [ADR-0007](0007-use-doorstop-for-requirements-management.md)
