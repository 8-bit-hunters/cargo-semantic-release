use cargo_semantic_release::test_util::repo_init;
pub use cargo_semantic_release::test_util::RepositoryTestExtensions;
use cargo_semantic_release::{Changes, SemanticVersionAction};

#[test]
fn empty_repo_raises_error() {
    // Given
    let (_temp_dir, repository) = repo_init(None);

    // When
    let result = Changes::from_repo(&repository);

    // Then
    assert!(result.is_err(), "Expected Error, but got Ok");
}

#[test]
fn major_change_increments_major_semantic_version() {
    // Given
    let commit_messages = vec![
        "💥 introduce breaking changes",
        ":sparkles: introduce new feature",
        ":green_heart: fix CI build",
        ":memo: add or update documentation",
    ];
    let (_temp_dir, repository) = repo_init(Some(commit_messages));

    // When
    let result = Changes::from_repo(&repository)
        .unwrap()
        .define_action_for_semantic_version();

    // Then
    assert_eq!(result, SemanticVersionAction::IncrementMajor);
}

#[test]
fn minor_change_increments_minor_semantic_version() {
    // Given
    let commit_messages = vec![
        ":sparkles: introduce new feature",
        ":green_heart: fix CI build",
        ":memo: add or update documentation",
    ];
    let (_temp_dir, repository) = repo_init(Some(commit_messages));

    // When
    let result = Changes::from_repo(&repository)
        .unwrap()
        .define_action_for_semantic_version();

    // Then
    assert_eq!(result, SemanticVersionAction::IncrementMinor);
}

#[test]
fn patch_change_increments_patch_semantic_version() {
    // Given
    let commit_messages = vec![
        ":green_heart: fix CI build",
        ":memo: add or update documentation",
    ];
    let (_temp_dir, repository) = repo_init(Some(commit_messages));

    // When
    let result = Changes::from_repo(&repository)
        .unwrap()
        .define_action_for_semantic_version();

    // Then
    assert_eq!(result, SemanticVersionAction::IncrementPatch);
}

#[test]
fn other_change_keeps_semantic_version() {
    // Given
    let commit_messages = vec![":memo: add or update documentation"];
    let (_temp_dir, repository) = repo_init(Some(commit_messages));

    // When
    let result = Changes::from_repo(&repository)
        .unwrap()
        .define_action_for_semantic_version();

    // Then
    assert_eq!(result, SemanticVersionAction::Keep);
}
