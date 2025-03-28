use git2::Commit;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct GitmojiCommit {
    message: String,
    hash: String,
    intention: Gitmoji,
    scope: String,
}

impl TryFrom<Commit<'_>> for GitmojiCommit {
    type Error = ();

    fn try_from(value: Commit<'_>) -> Result<Self, Self::Error> {
        let message = value.message().expect("Commit don't have message");
        let intention = Gitmoji::try_from(message).expect("Commit don't have intention");
        let message = message
            .replace(intention.as_utf(), "")
            .replace(intention.as_shortcode(), "")
            .trim_start()
            .to_string();
        let hash = value.id().to_string();
        Ok(Self {
            message,
            hash,
            intention,
            scope: "".to_string(),
        })
    }
}

impl Display for GitmojiCommit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let short_hash = self
            .hash
            .get(0..7)
            .unwrap_or("Error: can't show short hash");
        write!(
            f,
            "{} {} ({})",
            self.intention,
            self.message.trim_end(),
            short_hash
        )
    }
}

#[cfg(test)]
mod gitmoji_commit_tests {
    use crate::repo::commit::gitmoji::{Gitmoji, GitmojiCommit};
    use crate::test_util::{repo_init, RepositoryTestExtensions};

    #[test]
    fn create_from_git2_commit() {
        // Given
        let commit_messages = vec![":tada: initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let git2_commit = repository
            .find_commit_by_message(":tada: initial commit")
            .unwrap();

        // When
        let result = GitmojiCommit::try_from(git2_commit.clone()).expect("Failed to parse");

        // Then
        let hash = git2_commit.id().to_string();
        let expected_result = GitmojiCommit {
            message: "initial commit".to_string(),
            hash,
            intention: Gitmoji::Tada,
            scope: "".to_string(),
        };
        assert_eq!(result, expected_result)
    }

    #[test]
    fn display_formatting() {
        // Given
        let commit_messages = vec![":tada: initial commit"];
        let (_temp_dir, repository) = repo_init(Some(commit_messages));
        let git2_commit = repository
            .find_commit_by_message(":tada: initial commit")
            .expect("Could not find commit");
        let commit = GitmojiCommit::try_from(git2_commit.clone()).expect("Failed to parse");

        // When
        let print_out = format!("{}", commit);

        // Then
        assert_eq!(
            print_out,
            format!(
                "{} {} ({})",
                Gitmoji::Tada,
                "initial commit",
                git2_commit
                    .id()
                    .to_string()
                    .get(0..7)
                    .expect("Failed to get short hash")
            )
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, Copy)]
enum Gitmoji {
    Boom,
    Sparkles,
    ChildrenCrossing,
    Lipstick,
    Iphone,
    Egg,
    ChartWithUpwardsTrend,
    HeavyPlusSign,
    HeavyMinusSign,
    PassportControl,
    Art,
    Ambulance,
    Lock,
    Bug,
    Zap,
    GoalNet,
    Alien,
    Wheelchair,
    SpeechBalloon,
    Mag,
    Fire,
    WhiteCheckMark,
    ClosedLockWithKey,
    RotatingLight,
    GreenHeart,
    ArrowDown,
    ArrowUp,
    Pushpin,
    ConstructionWorker,
    Recycle,
    Wrench,
    Hammer,
    GlobeWithMeridians,
    Package,
    Truck,
    Bento,
    CardFileBox,
    LoudSound,
    Mute,
    BuildingConstruction,
    CameraFlash,
    Label,
    Seedling,
    TriangularFlagOnPost,
    Dizzy,
    AdhesiveBandage,
    MonocleFace,
    Necktie,
    Stethoscope,
    Technologist,
    Thread,
    SafetyVest,
    Memo,
    Rocket,
    Tada,
    Bookmark,
    Construction,
    Pencil2,
    Poop,
    Rewind,
    TwistedRightwardsArrows,
    PageFacingUp,
    Bulb,
    Beers,
    BustInSilhouette,
    ClownFace,
    SeeNoEvil,
    Alembic,
    Wastebasket,
    Coffin,
    TestTube,
    Bricks,
    MoneyWithWings,
}

impl Gitmoji {
    fn gitmoji_map() -> &'static HashMap<Gitmoji, Emoji> {
        use once_cell::sync::Lazy;
        use Gitmoji::*;
        static GITMOJIS: Lazy<HashMap<Gitmoji, Emoji>> = Lazy::new(|| {
            HashMap::from([
                (Boom, Emoji::new("💥", ":boom:")),
                (Sparkles, Emoji::new("✨", ":sparkles:")),
                (ChildrenCrossing, Emoji::new("🚸", ":children_crossing:")),
                (Lipstick, Emoji::new("💄", ":lipstick:")),
                (Iphone, Emoji::new("📱", ":iphone:")),
                (Egg, Emoji::new("🥚", ":egg:")),
                (
                    ChartWithUpwardsTrend,
                    Emoji::new("📈", ":chart_with_upwards_trend:"),
                ),
                (HeavyPlusSign, Emoji::new("➕", ":heavy_plus_sign:")),
                (HeavyMinusSign, Emoji::new("➖", ":heavy_minus_sign:")),
                (PassportControl, Emoji::new("🛂", ":passport_control:")),
                (Art, Emoji::new("🎨", ":art:")),
                (Ambulance, Emoji::new("🚑️", ":ambulance:")),
                (Lock, Emoji::new("🔒️", ":lock:")),
                (Bug, Emoji::new("🐛", ":bug:")),
                (Zap, Emoji::new("⚡️", ":zap:")),
                (GoalNet, Emoji::new("🥅", ":goal_net:")),
                (Alien, Emoji::new("👽️", ":alien:")),
                (Wheelchair, Emoji::new("♿️", ":wheelchair:")),
                (SpeechBalloon, Emoji::new("💬", ":speech_balloon:")),
                (Mag, Emoji::new("🔍️", ":mag:")),
                (Fire, Emoji::new("🔥", ":fire:")),
                (WhiteCheckMark, Emoji::new("✅", ":white_check_mark:")),
                (
                    ClosedLockWithKey,
                    Emoji::new("🔐", ":closed_lock_with_key:"),
                ),
                (RotatingLight, Emoji::new("🚨", ":rotating_light:")),
                (GreenHeart, Emoji::new("💚", ":green_heart:")),
                (ArrowDown, Emoji::new("⬇️", ":arrow_down:")),
                (ArrowUp, Emoji::new("⬆️", ":arrow_up:")),
                (Pushpin, Emoji::new("📌", ":pushpin:")),
                (
                    ConstructionWorker,
                    Emoji::new("👷", ":construction_worker:"),
                ),
                (Recycle, Emoji::new("♻️", ":recycle:")),
                (Wrench, Emoji::new("🔧", ":wrench:")),
                (Hammer, Emoji::new("🔨", ":hammer:")),
                (
                    GlobeWithMeridians,
                    Emoji::new("🌐", ":globe_with_meridians:"),
                ),
                (Package, Emoji::new("📦️", ":package:")),
                (Truck, Emoji::new("🚚", ":truck:")),
                (Bento, Emoji::new("🍱", ":bento:")),
                (CardFileBox, Emoji::new("🗃️", ":card_file_box:")),
                (LoudSound, Emoji::new("🔊", ":loud_sound:")),
                (Mute, Emoji::new("🔇", ":mute:")),
                (
                    BuildingConstruction,
                    Emoji::new("🏗️", ":building_construction:"),
                ),
                (CameraFlash, Emoji::new("📸", ":camera_flash:")),
                (Label, Emoji::new("🏷️", ":label:")),
                (Seedling, Emoji::new("🌱", ":seedling:")),
                (
                    TriangularFlagOnPost,
                    Emoji::new("🚩", ":triangular_flag_on_post:"),
                ),
                (Dizzy, Emoji::new("💫", ":dizzy:")),
                (AdhesiveBandage, Emoji::new("🩹", ":adhesive_bandage:")),
                (MonocleFace, Emoji::new("🧐", ":monocle_face:")),
                (Necktie, Emoji::new("👔", ":necktie:")),
                (Stethoscope, Emoji::new("🩺", ":stethoscope:")),
                (Technologist, Emoji::new("🧑", ":technologist:")),
                (Thread, Emoji::new("🧵", ":thread:")),
                (SafetyVest, Emoji::new("🦺", ":safety_vest:")),
                (Memo, Emoji::new("📝", ":memo:")),
                (Rocket, Emoji::new("🚀", ":rocket:")),
                (Tada, Emoji::new("🎉", ":tada:")),
                (Bookmark, Emoji::new("🔖", ":bookmark:")),
                (Construction, Emoji::new("🚧", ":construction:")),
                (Pencil2, Emoji::new("✏️", ":pencil2:")),
                (Poop, Emoji::new("💩", ":poop:")),
                (Rewind, Emoji::new("⏪️", ":rewind:")),
                (
                    TwistedRightwardsArrows,
                    Emoji::new("🔀", ":twisted_rightwards_arrows:"),
                ),
                (PageFacingUp, Emoji::new("📄", ":page_facing_up:")),
                (Bulb, Emoji::new("💡", ":bulb:")),
                (Beers, Emoji::new("🍻", ":beers:")),
                (BustInSilhouette, Emoji::new("👥", ":bust_in_silhouette:")),
                (ClownFace, Emoji::new("🤡", ":clown_face:")),
                (SeeNoEvil, Emoji::new("🙈", ":see_no_evil:")),
                (Alembic, Emoji::new("⚗️", ":alembic:")),
                (Wastebasket, Emoji::new("🗑️", ":wastebasket:")),
                (Coffin, Emoji::new("⚰️", ":coffin:")),
                (TestTube, Emoji::new("🧪", ":test_tube:")),
                (Bricks, Emoji::new("🧱", ":bricks:")),
                (MoneyWithWings, Emoji::new("💸", ":money_with_wings:")),
            ])
        });
        &GITMOJIS
    }

    fn as_utf(&self) -> &str {
        Gitmoji::gitmoji_map().get(self).map_or("❓", |e| e.utf)
    }

    fn as_shortcode(&self) -> &str {
        Gitmoji::gitmoji_map()
            .get(self)
            .map_or("❓", |e| e.shortcode)
    }
}

impl TryFrom<&str> for Gitmoji {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let intention: Vec<Gitmoji> = Gitmoji::gitmoji_map()
            .iter()
            .filter(|(_gitmoji, emoji)| {
                value.contains(emoji.utf) || value.contains(emoji.shortcode)
            })
            .map(|(gitmoji, _emoji)| *gitmoji)
            .collect();
        match intention.first() {
            Some(intention) => Ok(*intention),
            None => Err(()),
        }
    }
}

impl Display for Gitmoji {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let emoji = Gitmoji::gitmoji_map().get(self).map_or("❓", |e| e.utf);
        write!(f, "{emoji}")
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, Copy)]
struct Emoji {
    utf: &'static str,
    shortcode: &'static str,
}

impl Emoji {
    fn new(utf: &'static str, shortcode: &'static str) -> Self {
        Self { utf, shortcode }
    }
}

#[cfg(test)]
mod test_gitmoji {
    use crate::repo::commit::gitmoji::Gitmoji;

    #[test]
    fn display_formatting() {
        // Given
        let gitmojis = vec![
            (Gitmoji::Tada, "🎉"),
            (Gitmoji::Beers, "🍻"),
            (Gitmoji::Boom, "💥"),
        ];

        for (gitmoji, emoji_utf) in gitmojis {
            // When
            let result = format!("{gitmoji}");

            // Then
            assert_eq!(result, emoji_utf.to_string())
        }
    }

    #[test]
    fn as_shortcode() {
        // Given
        let gitmojis = vec![
            (Gitmoji::Tada, ":tada:"),
            (Gitmoji::Beers, ":beers:"),
            (Gitmoji::Boom, ":boom:"),
        ];

        for (gitmoji, emoji_utf) in gitmojis {
            // When
            let result = gitmoji.as_shortcode();

            // Then
            assert_eq!(result, emoji_utf)
        }
    }

    #[test]
    fn as_utf() {
        // Given
        let gitmojis = vec![
            (Gitmoji::Tada, "🎉"),
            (Gitmoji::Beers, "🍻"),
            (Gitmoji::Boom, "💥"),
        ];

        for (gitmoji, emoji_utf) in gitmojis {
            // When
            let result = gitmoji.as_utf();

            // Then
            assert_eq!(result, emoji_utf)
        }
    }

    #[test]
    fn test_from_str() {
        // Given
        let str = "hello :boom:";

        // When
        let result = Gitmoji::try_from(str).expect("Failed to parse");

        // Then
        assert_eq!(result, Gitmoji::Boom);
    }
}
