//! Commit scope parsing for Gitmoji/Conventional commit message format.
//!
//! This module provides functionality to extract package scopes from commit messages.
//! Scopes are used to associate commits with specific packages in a workspace,
//! enabling per-package version calculation and changelog generation.
//!
//! The parser extracts scopes from the first set of parentheses in the message,
//! splitting on commas and trimming whitespace from each scope name.
//!
//! # Message Format
//!
//! Scopes are extracted from parentheses in the commit message body (after the gitmoji
//! has been removed):
//!
//! - `(my-crate)` - single scope
//! - `(pkg1,pkg2)` - multiple scopes, no spaces
//! - `(pkg1, pkg2, pkg3)` - multiple scopes with spaces
//! - `(  my-crate  )` - whitespace is trimmed
//!
//! # Examples
//!
//! This is an internal module. Within the crate, import using:
//! `use super::scope::{CommitScopeParser, GitmojiCommitScopeParser};`

/// Trait for parsing scopes from commit messages.
///
/// Implementations can support different commit message conventions:
/// - Gitmoji: `:emoji: description (scope)`
/// - Conventional Commits: `type(scope): description` (future)
#[allow(dead_code)]
pub trait CommitScopeParser {
    /// Extracts package scopes from a commit message.
    ///
    /// Parses the first set of parentheses in the message, splitting the content
    /// on commas to extract individual scopes. Whitespace around scope names
    /// is automatically trimmed.
    ///
    /// # Message Format
    ///
    /// - `(scope)` - single scope
    /// - `(scope1,scope2)` - multiple scopes without spaces
    /// - `(scope1, scope2)` - multiple scopes with spaces
    /// - Messages without parentheses return an empty vector
    ///
    /// # Examples
    ///
    /// - `"Add feature (my-crate)"` returns `vec!["my-crate"]`
    /// - `"Fix (pkg1,pkg2)"` returns `vec!["pkg1", "pkg2"]`
    /// - `"Update README"` returns `vec![]`
    fn parse_scopes(&self, message: &str) -> Vec<String>;
}

/// Default parser implementation for Gitmoji-style commit messages.
///
/// This parser extracts scopes from the first set of parentheses in the commit
/// message body, after the gitmoji has been stripped. It is designed to work
/// with the Gitmoji commit convention where scopes are optionally specified
/// in parentheses.
///
/// # Usage
///
/// This parser is used internally by `GitmojiCommit::parse` to extract scopes
/// from commit messages. It can also be used directly for testing or custom
/// parsing needs.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct GitmojiCommitScopeParser;

impl CommitScopeParser for GitmojiCommitScopeParser {
    /// Extracts scopes from the first set of parentheses in the message.
    ///
    /// # Algorithm
    ///
    /// 1. Find the first opening parenthesis `(`
    /// 2. Find the first closing parenthesis `)` after it
    /// 3. Extract the content between them
    /// 4. Split on commas
    /// 5. Trim whitespace from each scope
    /// 6. Filter out empty strings
    /// 7. Return the collected scopes
    ///
    /// # Behavior
    ///
    /// - Returns empty `Vec` if no parentheses found
    /// - Returns empty `Vec` if parentheses are empty or contain only whitespace
    /// - Only the **first** parenthesis group is captured; subsequent groups are ignored
    /// - Handles nested parentheses by matching the first closing `)` after the first `(`
    ///
    /// # Examples
    ///
    /// - `"Fix (my-crate)"` returns `vec!["my-crate"]`
    /// - `"Fix (a,b,c)"` returns `vec!["a", "b", "c"]`
    /// - `"Fix (first) and (second)"` returns `vec!["first"]` (only first group)
    fn parse_scopes(&self, message: &str) -> Vec<String> {
        let Some(start_idx) = message.find('(') else {
            return Vec::new();
        };
        let rest = &message[start_idx + 1..];
        let Some(end_idx) = rest.find(')') else {
            return Vec::new();
        };
        let content = &rest[..end_idx];

        content
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_message_is_empty() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            Vec::<String>::new(),
            "an empty message should return an empty scope list"
        );
    }

    #[test]
    fn when_message_has_no_scopes() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Add feature";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            Vec::<String>::new(),
            "a message without parentheses should return an empty scope list"
        );
    }

    #[test]
    fn when_message_has_single_scope() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Add feature (my-crate)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["my-crate"],
            "a single scope should be extracted"
        );
    }

    #[test]
    fn when_message_has_multiple_scopes_comma_separated() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix bug (pkg1,pkg2)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["pkg1", "pkg2"],
            "multiple comma-separated scopes should each be extracted"
        );
    }

    #[test]
    fn when_message_has_multiple_scopes_with_spaces() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix bug (pkg1, pkg2, pkg3)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["pkg1", "pkg2", "pkg3"],
            "spaces after commas should be trimmed from each scope"
        );
    }

    #[test]
    fn when_scope_has_internal_whitespace() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix (  pkg1  ,  pkg2  )";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["pkg1", "pkg2"],
            "whitespace around scope names should be trimmed"
        );
    }

    #[test]
    fn when_scope_is_at_start_of_message() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "(scope) description";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["scope"],
            "scope at the start should be extracted"
        );
    }

    #[test]
    fn when_scope_is_at_end_of_message() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Description (scope)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["scope"],
            "scope at the end should be extracted"
        );
    }

    #[test]
    fn when_message_is_only_scope() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "(scope)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["scope"],
            "a message consisting only of a scope should be extracted"
        );
    }

    #[test]
    fn when_parentheses_are_empty() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix ()";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            Vec::<String>::new(),
            "empty parentheses should return an empty scope list"
        );
    }

    #[test]
    fn when_parentheses_contain_only_whitespace() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix (   )";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            Vec::<String>::new(),
            "parentheses with only whitespace should return an empty scope list"
        );
    }

    #[test]
    fn when_message_has_multiple_paren_groups() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix (first) and (second)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["first"],
            "only the first parenthesis group should be captured"
        );
    }

    #[test]
    fn when_scope_contains_hyphen() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Add feature (my-crate)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["my-crate"],
            "hyphenated package names should be preserved"
        );
    }

    #[test]
    fn when_scope_contains_path_separators() {
        // Given
        let parser = GitmojiCommitScopeParser;
        let message = "Fix (path/to/crate)";

        // When
        let result = parser.parse_scopes(message);

        // Then
        assert_eq!(
            result,
            vec!["path/to/crate"],
            "path-like package names with slashes should be preserved"
        );
    }
}
