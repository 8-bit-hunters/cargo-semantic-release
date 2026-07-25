use crate::repo::prelude::Gitmoji;
use serde::Deserialize;
use std::path::Path;

/// Configuration for how this tool determines the next semantic version.
///
/// Mirrors the shape of [python-semantic-release's `[tool.semantic_release]`
/// config](https://python-semantic-release.readthedocs.io/en/latest/configuration/configuration.html#config)
/// so existing config using its `emoji` commit parser can be reused as-is.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct SemanticReleaseConfig {
    pub tag_format: String,
    pub commit_parser_options: CommitParserOptions,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct CommitParserOptions {
    pub major_tags: Vec<String>,
    pub minor_tags: Vec<String>,
    pub patch_tags: Vec<String>,
}

impl Default for SemanticReleaseConfig {
    fn default() -> Self {
        Self {
            tag_format: "v{version}".to_string(),
            commit_parser_options: CommitParserOptions::default(),
        }
    }
}

impl Default for CommitParserOptions {
    fn default() -> Self {
        fn shortcodes(gitmojis: &[Gitmoji]) -> Vec<String> {
            gitmojis
                .iter()
                .map(|gitmoji| gitmoji.as_shortcode().to_string())
                .collect()
        }

        Self {
            major_tags: shortcodes(&[Gitmoji::Boom]),
            minor_tags: shortcodes(&[
                Gitmoji::Sparkles,
                Gitmoji::ChildrenCrossing,
                Gitmoji::Lipstick,
                Gitmoji::Iphone,
                Gitmoji::Egg,
                Gitmoji::ChartWithUpwardsTrend,
                Gitmoji::HeavyPlusSign,
                Gitmoji::HeavyMinusSign,
                Gitmoji::PassportControl,
            ]),
            patch_tags: shortcodes(&[
                Gitmoji::Art,
                Gitmoji::Ambulance,
                Gitmoji::Lock,
                Gitmoji::Bug,
                Gitmoji::Zap,
                Gitmoji::GoalNet,
                Gitmoji::Alien,
                Gitmoji::Wheelchair,
                Gitmoji::SpeechBalloon,
                Gitmoji::Mag,
                Gitmoji::Fire,
                Gitmoji::WhiteCheckMark,
                Gitmoji::ClosedLockWithKey,
                Gitmoji::RotatingLight,
                Gitmoji::GreenHeart,
                Gitmoji::ArrowDown,
                Gitmoji::ArrowUp,
                Gitmoji::Pushpin,
                Gitmoji::ConstructionWorker,
                Gitmoji::Recycle,
                Gitmoji::Wrench,
                Gitmoji::Hammer,
                Gitmoji::GlobeWithMeridians,
                Gitmoji::Package,
                Gitmoji::Truck,
                Gitmoji::Bento,
                Gitmoji::CardFileBox,
                Gitmoji::LoudSound,
                Gitmoji::Mute,
                Gitmoji::BuildingConstruction,
                Gitmoji::CameraFlash,
                Gitmoji::Label,
                Gitmoji::Seedling,
                Gitmoji::TriangularFlagOnPost,
                Gitmoji::Dizzy,
                Gitmoji::AdhesiveBandage,
                Gitmoji::MonocleFace,
                Gitmoji::Necktie,
                Gitmoji::Stethoscope,
                Gitmoji::Technologist,
                Gitmoji::Thread,
                Gitmoji::SafetyVest,
            ]),
        }
    }
}

impl SemanticReleaseConfig {
    /// Discover the semantic-release config for the project at `repo_root`.
    ///
    /// Looks in order for:
    /// 1. `Cargo.toml`'s `[package.metadata.semantic_release]` table
    /// 2. a standalone `semantic_release.toml` file (bare table, no wrapper)
    ///
    /// Falls back to [`SemanticReleaseConfig::default`] if neither is found.
    pub fn discover(repo_root: &Path) -> Result<Self, String> {
        if let Some(config) = Self::discover_from_cargo_toml(repo_root)? {
            return Ok(config);
        }
        if let Some(config) = Self::discover_from_semantic_release_toml(repo_root)? {
            return Ok(config);
        }
        Ok(Self::default())
    }

    fn discover_from_cargo_toml(repo_root: &Path) -> Result<Option<Self>, String> {
        #[derive(Debug, Deserialize)]
        struct CargoMetadata {
            semantic_release: Option<SemanticReleaseConfig>,
        }

        let cargo_toml_path = repo_root.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(None);
        }

        let manifest =
            cargo_toml::Manifest::<CargoMetadata>::from_path_with_metadata(&cargo_toml_path)
                .map_err(|error| error.to_string())?;

        Ok(manifest
            .package
            .and_then(|package| package.metadata)
            .and_then(|metadata| metadata.semantic_release))
    }

    fn discover_from_semantic_release_toml(repo_root: &Path) -> Result<Option<Self>, String> {
        let path = repo_root.join("semantic_release.toml");
        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        toml::from_str(&contents)
            .map(Some)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod semantic_release_config_tests {
    use super::SemanticReleaseConfig;
    use std::fs;
    use tempfile::TempDir;

    fn temp_repo_with_files(files: &[(&str, &str)]) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        for (name, contents) in files {
            fs::write(temp_dir.path().join(name), contents).unwrap();
        }
        temp_dir
    }

    #[test]
    fn default_matches_the_hardcoded_gitmoji_lists() {
        // Given / When
        let config = SemanticReleaseConfig::default();

        // Then
        assert_eq!(config.tag_format, "v{version}");
        assert_eq!(config.commit_parser_options.major_tags, vec![":boom:"]);
        assert_eq!(config.commit_parser_options.minor_tags.len(), 9);
        assert_eq!(config.commit_parser_options.patch_tags.len(), 42);
        assert!(config
            .commit_parser_options
            .minor_tags
            .contains(&":sparkles:".to_string()));
        assert!(config
            .commit_parser_options
            .patch_tags
            .contains(&":bug:".to_string()));
    }

    #[test]
    fn discovers_config_from_cargo_toml_metadata() {
        // Given
        let temp_dir = temp_repo_with_files(&[(
            "Cargo.toml",
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\nedition = \"2021\"\n\n\
             [package.metadata.semantic_release]\n\
             tag_format = \"release-{version}\"\n\n\
             [package.metadata.semantic_release.commit_parser_options]\n\
             major_tags = [\":test_tube:\"]\n",
        )]);

        // When
        let result = SemanticReleaseConfig::discover(temp_dir.path()).unwrap();

        // Then
        assert_eq!(result.tag_format, "release-{version}");
        assert_eq!(result.commit_parser_options.major_tags, vec![":test_tube:"]);
        assert_eq!(result.commit_parser_options.minor_tags.len(), 9);
    }

    #[test]
    fn discovers_config_from_standalone_semantic_release_toml() {
        // Given
        let temp_dir = temp_repo_with_files(&[(
            "semantic_release.toml",
            "tag_format = \"release-{version}\"\n",
        )]);

        // When
        let result = SemanticReleaseConfig::discover(temp_dir.path()).unwrap();

        // Then
        assert_eq!(result.tag_format, "release-{version}");
    }

    #[test]
    fn cargo_toml_takes_precedence_over_semantic_release_toml() {
        // Given
        let temp_dir = temp_repo_with_files(&[
            (
                "Cargo.toml",
                "[package]\nname = \"foo\"\nversion = \"1.2.3\"\nedition = \"2021\"\n\n\
                 [package.metadata.semantic_release]\n\
                 tag_format = \"from-cargo-toml-{version}\"\n",
            ),
            (
                "semantic_release.toml",
                "tag_format = \"from-standalone-{version}\"\n",
            ),
        ]);

        // When
        let result = SemanticReleaseConfig::discover(temp_dir.path()).unwrap();

        // Then
        assert_eq!(result.tag_format, "from-cargo-toml-{version}");
    }

    #[test]
    fn falls_back_to_default_when_cargo_toml_has_no_semantic_release_metadata() {
        // Given
        let temp_dir = temp_repo_with_files(&[(
            "Cargo.toml",
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\nedition = \"2021\"\n",
        )]);

        // When
        let result = SemanticReleaseConfig::discover(temp_dir.path()).unwrap();

        // Then
        assert_eq!(result, SemanticReleaseConfig::default());
    }

    #[test]
    fn falls_back_to_default_when_no_config_files_exist() {
        // Given
        let temp_dir = TempDir::new().unwrap();

        // When
        let result = SemanticReleaseConfig::discover(temp_dir.path()).unwrap();

        // Then
        assert_eq!(result, SemanticReleaseConfig::default());
    }
}
