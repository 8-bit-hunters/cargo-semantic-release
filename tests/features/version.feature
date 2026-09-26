Feature: Deciding the version of the next publication

  As the maintainer of a published work
  I want the number of its next publication decided from the record of changes made since the last one
  So that the number alone tells a reader what kind of change to expect

  A clerk could do this by hand: read what each recorded change did, and number the next publication
  accordingly.

  Background:
    Given the work was last published as version "1.0.0"

  @TST-001
  Scenario: Proposing a version after a change that breaks how the work is used
    Given the following changes were recorded since then
      | change                                       |
      | a change that breaks how the work is used    |
      | an addition that leaves existing use working |
    When a version is proposed for the next publication
    Then the proposed version is "2.0.0"

  @TST-002
  Scenario: Proposing a version after an addition that leaves existing use working
    Given the following changes were recorded since then
      | change                                       |
      | an addition that leaves existing use working |
      | a correction                                 |
    When a version is proposed for the next publication
    Then the proposed version is "1.1.0"

  @TST-003
  Scenario: Proposing a version after nothing but a correction
    Given the following changes were recorded since then
      | change       |
      | a correction |
    When a version is proposed for the next publication
    Then the proposed version is "1.0.1"

  @TST-004
  Scenario: Proposing a version after changes that leave the work itself untouched
    Given the following changes were recorded since then
      | change                |
      | a note about the work |
    When a version is proposed for the next publication
    Then the proposed version is "1.0.0"

  @TST-005
  Scenario: Demanding a major version after nothing but a correction
    Given the following changes were recorded since then
      | change       |
      | a correction |
    When a major version is demanded for the next publication
    Then the proposed version is "2.0.0"

  @TST-006
  Scenario: Proposing a version without publishing it
    Given the following changes were recorded since then
      | change                                       |
      | an addition that leaves existing use working |
    When a version is proposed for the next publication
    Then the work still carries version "1.0.0"
    And no publication is recorded
