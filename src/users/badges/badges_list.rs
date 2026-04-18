pub const TIERED_BADGES: &[(&str, &[&str])] = &[
    (
        "classic.hardest_level",
        &[
            "1000", "750", "500", "250", "150", "100", "75", "50", "25", "10",
        ],
    ),
    (
        "platformer.hardest_level",
        &["150", "125", "100", "75", "50", "25", "10"],
    ),
    (
        "global.level_completion",
        &["5", "10", "25", "50", "75", "100", "150", "200", "250"],
    ),
    (
        "classic.leaderboard_rank",
        &["2000", "1000", "500", "250", "100", "50", "20"],
    ),
    ("global.pack_completion", &["3", "5", "10", "15"]),
    (
        "global.hardest_pack_tier",
        &["iron", "gold", "ruby", "sapphire", "pearl", "diamond"],
    ),
    ("global.publisher_levels", &["4", "8"]),
    ("global.level_tags.timings", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.overall", &["5", "10", "25", "35", "50"]),
    (
        "global.level_tags.chokepoints",
        &["5", "10", "25", "35", "50"],
    ),
    (
        "global.level_tags.fastpaced",
        &["5", "10", "25", "35", "50"],
    ),
    ("global.level_tags.learny", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.memory", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.wave", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.22", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.ship", &["5", "10", "25", "35", "50"]),
    (
        "global.level_tags.nervecontrol",
        &["5", "10", "25", "35", "50"],
    ),
    ("global.level_tags.xl", &["5", "10", "25", "35", "50"]),
    (
        "global.level_tags.clicksync",
        &["5", "10", "25", "35", "50"],
    ),
    ("global.level_tags.highcps", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.duals", &["5", "10", "15", "20", "25"]),
    ("global.level_tags.nong", &["5", "10", "15", "20", "25"]),
    ("global.level_tags.cube", &["5", "10", "15", "20", "25"]),
    ("global.level_tags.gimmicky", &["5", "10", "15", "20", "25"]),
    ("global.level_tags.flow", &["5", "10", "25", "35", "50"]),
    (
        "global.level_tags.slowpaced",
        &["5", "10", "25", "35", "50"],
    ),
    (
        "global.level_tags.precision",
        &["5", "10", "25", "35", "50"],
    ),
    ("global.level_tags.xxl", &["5", "10", "25", "35", "50"]),
    ("global.level_tags.19", &["5", "10", "15", "20", "30"]),
    ("global.level_tags.medium", &["5", "10", "15", "20", "30"]),
    ("global.level_tags.20", &["5", "10", "15", "20", "30"]),
    ("global.level_tags.circles", &["5", "10", "15", "20", "30"]),
    ("global.level_tags.2p", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.ufo", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.ball", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.robot", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.spider", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.bossfight", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.mirror", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.xxlplus", &["3", "5", "10", "15", "20"]),
    ("global.level_tags.oldswing", &["5", "10", "15"]),
    ("global.level_tags.newswing", &["3", "5", "10"]),
];

pub const SINGLE_BADGES: &[&str] = &[
    "global.level_tags.alltags.1",
    "global.all_nlw",
    "global.edel_high",
    "global.edel_low",
    "global.2p_and_solo",
    "global.alphabet",
    "global.first_victor",
    "global.creator",
    "global.verifier",
    "platformer.fastest_time",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagBadgeMode {
    And,
    Or,
}

// (badge code, corresponding level tags, whether to require all tags individually or sum them all)
pub const LEVEL_TAG_BADGES: &[(&str, &[&str], TagBadgeMode)] = &[
    ("timings", &["Timings"], TagBadgeMode::Or),
    ("overall", &["Overall"], TagBadgeMode::Or),
    ("chokepoints", &["Chokepoints"], TagBadgeMode::Or),
    ("fastpaced", &["Fast-Paced"], TagBadgeMode::Or),
    ("learny", &["Learny"], TagBadgeMode::Or),
    ("memory", &["Memory"], TagBadgeMode::Or),
    ("wave", &["Wave"], TagBadgeMode::Or),
    ("22", &["2.2"], TagBadgeMode::Or),
    ("ship", &["Ship"], TagBadgeMode::Or),
    ("nervecontrol", &["Nerve Control"], TagBadgeMode::Or),
    ("xl", &["XL"], TagBadgeMode::Or),
    ("clicksync", &["Clicksync"], TagBadgeMode::Or),
    ("highcps", &["High CPS"], TagBadgeMode::Or),
    ("duals", &["Duals"], TagBadgeMode::Or),
    ("nong", &["NONG"], TagBadgeMode::Or),
    ("cube", &["Cube"], TagBadgeMode::Or),
    ("gimmicky", &["Gimmicky"], TagBadgeMode::Or),
    ("flow", &["Flow"], TagBadgeMode::Or),
    ("slowpaced", &["Slow-Paced"], TagBadgeMode::Or),
    ("precision", &["Precision"], TagBadgeMode::Or),
    ("xxl", &["XXL"], TagBadgeMode::Or),
    ("19", &["1.9", "1.9PS"], TagBadgeMode::Or),
    ("medium", &["Medium"], TagBadgeMode::Or),
    ("20", &["2.0"], TagBadgeMode::Or),
    ("circles", &["Circles"], TagBadgeMode::Or),
    ("2p", &["2P"], TagBadgeMode::Or),
    ("ufo", &["UFO"], TagBadgeMode::Or),
    ("ball", &["Ball"], TagBadgeMode::Or),
    ("robot", &["Robot"], TagBadgeMode::Or),
    ("spider", &["Spider"], TagBadgeMode::Or),
    ("bossfight", &["Bossfight"], TagBadgeMode::Or),
    ("mirror", &["Mirror"], TagBadgeMode::Or),
    ("xxlplus", &["XXL+"], TagBadgeMode::Or),
    ("oldswing", &["Old Swing"], TagBadgeMode::Or),
    ("newswing", &["New Swing"], TagBadgeMode::Or),
    (
        "alltags",
        &[
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
            "Precision",
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
        TagBadgeMode::And,
    ),
];

// hardcoded here to not bother with dynamic fetching, they never change
pub const HARDEST_PACK_TIERS: &[(&str, &str)] = &[
    ("iron", "Iron Tier"),
    ("gold", "Gold Tier"),
    ("ruby", "Ruby Tier"),
    ("sapphire", "Sapphire Tier"),
    ("pearl", "Pearl Tier"),
    ("diamond", "Diamond Tier"),
];

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
    "Fuck",
];

pub struct AvailableBadges;

impl AvailableBadges {
    pub fn get_all() -> Vec<String> {
        let mut badges = SINGLE_BADGES
            .iter()
            .map(|badge| (*badge).to_string())
            .collect::<Vec<_>>();

        badges.extend(TIERED_BADGES.iter().flat_map(|(prefix, values)| {
            values.iter().map(move |value| format!("{prefix}.{value}"))
        }));

        badges
    }
}
