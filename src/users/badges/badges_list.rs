use crate::list::{bounty::BountyType, List};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use strum::IntoEnumIterator as _;
use strum_macros::{AsRefStr, Display, EnumIter};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum BadgeRule {
    HardestLevel {
        list: List,
        threshold: i32,
        cutoff: Option<DateTime<Utc>>,
    },
    LevelCompletion(usize),
    LeaderboardRank(i32),
    PackCompletion(usize),
    BountyBoard {
        bounty_type: BountyType,
        threshold: usize,
    },
    HardestPackTier(PackTier),
    PublisherLevels(usize),
    LevelTags {
        alias: &'static str,
        tags: &'static [&'static str],
        mode: TagBadgeMode,
        threshold: usize,
    },
    AllNlw,
    EdelHigh,
    EdelLow,
    #[strum(serialize = "2p_and_solo")]
    TwoPlayerAndSolo,
    Alphabet,
    FirstVictor,
    Creator,
    Verifier,
    FastestTime,
}

impl BadgeRule {
    /// wether this rule is only for one list statistics or for both (none)
    pub fn list(self) -> Option<List> {
        match self {
            Self::HardestLevel { list, .. } => Some(list),
            Self::LeaderboardRank(_) => Some(List::Classic),
            Self::FastestTime => Some(List::Platformer),
            Self::LevelCompletion(_)
            | Self::PackCompletion(_)
            | Self::BountyBoard { .. }
            | Self::HardestPackTier(_)
            | Self::PublisherLevels(_)
            | Self::LevelTags { .. }
            | Self::AllNlw
            | Self::EdelHigh
            | Self::EdelLow
            | Self::TwoPlayerAndSolo
            | Self::Alphabet
            | Self::FirstVictor
            | Self::Creator
            | Self::Verifier => None,
        }
    }
}

impl fmt::Display for BadgeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = self.list().map_or_else(
            || "global".to_owned(),
            |list| list.to_string().to_ascii_lowercase(),
        );
        write!(f, "{scope}.{}", self.as_ref())?;

        let threshold: Option<&dyn fmt::Display> = match self {
            Self::HardestLevel { threshold, .. } | Self::LeaderboardRank(threshold) => {
                Some(threshold)
            }
            Self::LevelCompletion(threshold)
            | Self::PackCompletion(threshold)
            | Self::PublisherLevels(threshold) => Some(threshold),
            Self::BountyBoard {
                bounty_type,
                threshold,
            } => {
                write!(f, ".{}", bounty_type.as_ref())?;
                Some(threshold)
            }
            Self::HardestPackTier(tier) => {
                write!(f, ".{}", tier.code())?;
                None
            }
            Self::LevelTags {
                alias, threshold, ..
            } => {
                write!(f, ".{alias}")?;
                Some(threshold)
            }
            Self::AllNlw
            | Self::EdelHigh
            | Self::EdelLow
            | Self::TwoPlayerAndSolo
            | Self::Alphabet
            | Self::FirstVictor
            | Self::Creator
            | Self::Verifier
            | Self::FastestTime => None,
        };

        if let Some(threshold) = threshold {
            write!(f, ".{threshold}")?;
        }

        Ok(())
    }
}

// hardcoded here to not bother with dynamic fetching, they never change
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumIter,
    Display,
    Serialize,
    Deserialize,
    ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum PackTier {
    Iron,
    Gold,
    Ruby,
    Sapphire,
    Pearl,
    Diamond,
}

impl PackTier {
    fn code(self) -> String {
        self.to_string().to_ascii_lowercase()
    }

    pub fn name(self) -> String {
        format!("{self} Tier")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TagBadgeMode {
    And,
    Or,
}

pub const NLW_TIERS: &[&str] = &[
    "Beginner",
    "Easy",
    "Medium",
    "Hard",
    "Very Hard",
    "Insane",
    "Extreme",
    "Remorseless",
    "Relentless",
    "Terrifying",
    "Catastrophic",
    "Inexorable",
    "Excruciating",
    "Merciless",
    "Fuck",
];

pub struct AvailableBadges;

impl AvailableBadges {
    pub fn get_all() -> Vec<BadgeRule> {
        let mut badges = vec![
            BadgeRule::AllNlw,
            BadgeRule::EdelHigh,
            BadgeRule::EdelLow,
            BadgeRule::TwoPlayerAndSolo,
            BadgeRule::Alphabet,
            BadgeRule::FirstVictor,
            BadgeRule::Creator,
            BadgeRule::Verifier,
            BadgeRule::FastestTime,
        ];

        // used to require having beaten a level after a certain date for it to count for a hardest rank badge (e.g. to avoid getting the top 500 badge for beating Acu when it was top 400)
        //(badge scope, hardest level rank, date after which the record can be counted for this badge")
        badges.extend(
            [
                (1000, Some("2025-01-01T00:00:00Z")),
                (750, Some("2024-01-01T00:00:00Z")),
                (500, Some("2023-01-01T00:00:00Z")),
                (250, Some("2022-01-01T00:00:00Z")),
                (150, None),
                (100, None),
                (75, None),
                (50, None),
                (25, None),
                (10, None),
            ]
            .map(|(threshold, cutoff)| {
                // for the hardest level badge, gets the defined cutoff date for a given tier
                let cutoff = cutoff.map(|cutoff| {
                    DateTime::parse_from_rfc3339(cutoff)
                        .expect("Cutoffs must be a valid date")
                        .with_timezone(&Utc)
                });
                BadgeRule::HardestLevel {
                    list: List::Classic,
                    threshold,
                    cutoff,
                }
            }),
        );
        badges.extend(
            [150, 125, 100, 75, 50, 25, 10].map(|threshold| BadgeRule::HardestLevel {
                list: List::Platformer,
                threshold,
                cutoff: None,
            }),
        );
        badges.extend([5, 10, 25, 50, 75, 100, 150, 200, 250].map(BadgeRule::LevelCompletion));
        badges.extend([2000, 1000, 500, 250, 100, 50, 20].map(BadgeRule::LeaderboardRank));
        badges.extend([3, 5, 10, 15].map(BadgeRule::PackCompletion));
        for (bounty_type, thresholds) in [
            (BountyType::Bounty, &[5, 10, 15, 20, 25][..]),
            (BountyType::Weekly, &[3, 7, 12, 18, 25][..]),
            (BountyType::Monthly, &[1, 2, 4, 6, 8][..]),
            (BountyType::Event, &[1, 3, 5][..]),
        ] {
            badges.extend(thresholds.iter().map(|&threshold| BadgeRule::BountyBoard {
                bounty_type,
                threshold,
            }));
        }
        badges.extend(PackTier::iter().map(BadgeRule::HardestPackTier));
        badges.extend([4, 8].map(BadgeRule::PublisherLevels));
        // (badge code, corresponding level tags, whether to require all tags individually or sum them all)
        for (thresholds, tag_badges) in [
            (
                &[5, 10, 25, 35, 50][..],
                &[
                    ("timings", &["Timings"][..]),
                    ("overall", &["Overall"][..]),
                    ("chokepoints", &["Chokepoints"][..]),
                    ("fastpaced", &["Fast-Paced"][..]),
                    ("learny", &["Learny"][..]),
                    ("memory", &["Memory"][..]),
                    ("wave", &["Wave"][..]),
                    ("22", &["2.2"][..]),
                    ("ship", &["Ship"][..]),
                    ("nervecontrol", &["Nerve Control"][..]),
                    ("xl", &["XL"][..]),
                    ("clicksync", &["Clicksync"][..]),
                    ("highcps", &["High CPS"][..]),
                    ("flow", &["Flow"][..]),
                    ("slowpaced", &["Slow-Paced"][..]),
                    ("xxl", &["XXL"][..]),
                ][..],
            ),
            (
                &[5, 10, 15, 20, 25][..],
                &[
                    ("duals", &["Duals"][..]),
                    ("nong", &["NONG"][..]),
                    ("cube", &["Cube"][..]),
                    ("gimmicky", &["Gimmicky"][..]),
                ][..],
            ),
            (
                &[5, 10, 15, 20, 30][..],
                &[
                    ("19", &["1.9", "1.9PS"][..]),
                    ("medium", &["Medium"][..]),
                    ("20", &["2.0"][..]),
                    ("circles", &["Circles"][..]),
                ][..],
            ),
            (
                &[3, 5, 10, 15, 20][..],
                &[
                    ("2p", &["2P"][..]),
                    ("ufo", &["UFO"][..]),
                    ("ball", &["Ball"][..]),
                    ("robot", &["Robot"][..]),
                    ("spider", &["Spider"][..]),
                    ("bossfight", &["Bossfight"][..]),
                    ("mirror", &["Mirror"][..]),
                    ("xxlplus", &["XXL+"][..]),
                ][..],
            ),
            (&[5, 10, 15][..], &[("oldswing", &["Old Swing"][..])][..]),
            (&[3, 5, 10][..], &[("newswing", &["New Swing"][..])][..]),
        ] {
            badges.extend(tag_badges.iter().flat_map(|&(alias, tags)| {
                thresholds
                    .iter()
                    .map(move |&threshold| BadgeRule::LevelTags {
                        alias,
                        tags,
                        mode: TagBadgeMode::Or,
                        threshold,
                    })
            }));
        }
        badges.push(BadgeRule::LevelTags {
            alias: "alltags",
            tags: &[
                "2P",
                "Circles",
                "Clicksync",
                "Fast-Paced",
                "Timings",
                "Chokepoints",
                "Learny",
                "Memory",
                "High CPS",
                "Gimmicky",
                "Flow",
                "Slow-Paced",
                "Bossfight",
                "Mirror",
                "Nerve Control",
                "Cube",
                "Ship",
                "Ball",
                "UFO",
                "Wave",
                "Robot",
                "Spider",
                "Old Swing",
                "New Swing",
                "Duals",
                "Overall",
            ],
            mode: TagBadgeMode::And,
            threshold: 1,
        });

        badges
    }
}
