use crate::character::{Character, ThoughtPhase};
use crate::equipment::EquipSlot;
use crate::skills::Skill;
use chrono::Timelike;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Rarity {
    pub fn label(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Legendary => "Legendary",
        }
    }

    pub fn color(&self, s: &str) -> String {
        match self {
            Rarity::Common => s.white().to_string(),
            Rarity::Uncommon => s.green().to_string(),
            Rarity::Rare => s.blue().bold().to_string(),
            Rarity::Legendary => s.yellow().bold().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Achievement {
    FirstBlood,      // First critical success
    SnakeEyes,       // First critical failure
    Survivor,        // Survive a game over (health/morale hit 0 but continued)
    Workaholic,      // 100 total checks
    Grinder,         // 500 total checks
    LevelFive,       // Reach level 5
    LevelTen,        // Reach level 10
    Mixologist,      // Use all 6 base substances (coffee, cigarette, alcohol, speed, nostalgia, pyrholidon)
    PsychonautInit,  // Use LSD or Mushrooms
    ThoughtLeader,   // Internalize 5 thoughts
    WellDressed,     // Have all 6 equipment slots occupied
    NightOwl,        // Pass a check during Night (0–5 AM)
    CoffeeAddict,    // Use 10 coffees total
    IronWill,        // 10 success streak
    Unlucky,         // 5 fail streak
    JackOfAllTrades, // Pass a check with 12+ different skills
}

pub struct AchievementInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub rarity: Rarity,
}

impl Achievement {
    pub fn all() -> &'static [Achievement] {
        &[
            Achievement::FirstBlood,
            Achievement::SnakeEyes,
            Achievement::Survivor,
            Achievement::Workaholic,
            Achievement::Grinder,
            Achievement::LevelFive,
            Achievement::LevelTen,
            Achievement::Mixologist,
            Achievement::PsychonautInit,
            Achievement::ThoughtLeader,
            Achievement::WellDressed,
            Achievement::NightOwl,
            Achievement::CoffeeAddict,
            Achievement::IronWill,
            Achievement::Unlucky,
            Achievement::JackOfAllTrades,
        ]
    }

    pub fn info(&self) -> AchievementInfo {
        match self {
            Achievement::FirstBlood => AchievementInfo {
                name: "First Blood",
                description: "Land your first critical success (double 6).",
                icon: "★",
                rarity: Rarity::Common,
            },
            Achievement::SnakeEyes => AchievementInfo {
                name: "Snake Eyes",
                description: "Roll your first critical failure (double 1).",
                icon: "⚁",
                rarity: Rarity::Common,
            },
            Achievement::Survivor => AchievementInfo {
                name: "Survivor",
                description: "Survive a game over — health or morale collapsed but you kept going.",
                icon: "♻",
                rarity: Rarity::Uncommon,
            },
            Achievement::Workaholic => AchievementInfo {
                name: "Workaholic",
                description: "Complete 100 total skill checks.",
                icon: "⚙",
                rarity: Rarity::Common,
            },
            Achievement::Grinder => AchievementInfo {
                name: "Grinder",
                description: "Complete 500 total skill checks. You never stop.",
                icon: "⛏",
                rarity: Rarity::Rare,
            },
            Achievement::LevelFive => AchievementInfo {
                name: "Rising Star",
                description: "Reach level 5.",
                icon: "◆",
                rarity: Rarity::Common,
            },
            Achievement::LevelTen => AchievementInfo {
                name: "Veteran",
                description: "Reach level 10. This city has nothing left to teach you.",
                icon: "◈",
                rarity: Rarity::Rare,
            },
            Achievement::Mixologist => AchievementInfo {
                name: "Mixologist",
                description: "Sample all 6 base substances.",
                icon: "⚗",
                rarity: Rarity::Uncommon,
            },
            Achievement::PsychonautInit => AchievementInfo {
                name: "Psychonaut",
                description: "Use LSD or Mushrooms. The walls breathe.",
                icon: "✦",
                rarity: Rarity::Rare,
            },
            Achievement::ThoughtLeader => AchievementInfo {
                name: "Thought Leader",
                description: "Internalize 5 thoughts in your Thought Cabinet.",
                icon: "💡",
                rarity: Rarity::Uncommon,
            },
            Achievement::WellDressed => AchievementInfo {
                name: "Well Dressed",
                description: "Fill all 6 equipment slots simultaneously.",
                icon: "👔",
                rarity: Rarity::Uncommon,
            },
            Achievement::NightOwl => AchievementInfo {
                name: "Night Owl",
                description: "Pass a skill check between midnight and 5 AM.",
                icon: "🌙",
                rarity: Rarity::Common,
            },
            Achievement::CoffeeAddict => AchievementInfo {
                name: "Coffee Addict",
                description: "Consume 10 cups of coffee in total.",
                icon: "☕",
                rarity: Rarity::Common,
            },
            Achievement::IronWill => AchievementInfo {
                name: "Iron Will",
                description: "Achieve a streak of 10 consecutive successful checks.",
                icon: "⚔",
                rarity: Rarity::Rare,
            },
            Achievement::Unlucky => AchievementInfo {
                name: "Unlucky",
                description: "Fail 5 checks in a row. The dice have spoken.",
                icon: "☠",
                rarity: Rarity::Common,
            },
            Achievement::JackOfAllTrades => AchievementInfo {
                name: "Jack of All Trades",
                description: "Pass checks using 12 or more different skills.",
                icon: "⚜",
                rarity: Rarity::Legendary,
            },
        }
    }
}

/// Returns all achievements currently earned by the character.
/// Call this to diff against `ch.achievements` to find newly earned ones.
pub fn check_achievements(ch: &Character) -> Vec<Achievement> {
    let mut earned = Vec::new();

    // FirstBlood — any critical success in history
    if ch.check_history.iter().any(|r| r.roll.0 == 6 && r.roll.1 == 6) {
        earned.push(Achievement::FirstBlood);
    }

    // SnakeEyes — any critical failure in history
    if ch.check_history.iter().any(|r| r.roll.0 == 1 && r.roll.1 == 1) {
        earned.push(Achievement::SnakeEyes);
    }

    // Survivor — journal contains a game over entry (health or morale hit 0)
    // We detect this by looking for GameOver entry type in the journal
    let had_game_over = ch.journal.iter().any(|e| {
        matches!(e.entry_type, crate::journal::EntryType::GameOver)
    });
    if had_game_over {
        earned.push(Achievement::Survivor);
    }

    // Workaholic — 100 total checks
    if ch.check_history.len() >= 100 {
        earned.push(Achievement::Workaholic);
    }

    // Grinder — 500 total checks
    if ch.check_history.len() >= 500 {
        earned.push(Achievement::Grinder);
    }

    // LevelFive / LevelTen
    if ch.level >= 5 {
        earned.push(Achievement::LevelFive);
    }
    if ch.level >= 10 {
        earned.push(Achievement::LevelTen);
    }

    // Mixologist — used all 6 base substances (coffee, cigarette, alcohol, speed, nostalgia, pyrholidon)
    // We detect via journal SubstanceUsed entries
    let base_substances = [
        "Coffee", "Cigarette", "Alcohol", "Speed", "Nostalgia", "Pyrholidon",
    ];
    let used_substances: HashSet<&str> = ch.journal.iter()
        .filter(|e| matches!(e.entry_type, crate::journal::EntryType::SubstanceUsed))
        .flat_map(|e| base_substances.iter().filter(|&&s| e.content.contains(s)).copied())
        .collect();
    if used_substances.len() >= 6 {
        earned.push(Achievement::Mixologist);
    }

    // PsychonautInit — used LSD or Mushrooms
    let used_psychedelic = ch.journal.iter()
        .filter(|e| matches!(e.entry_type, crate::journal::EntryType::SubstanceUsed))
        .any(|e| e.content.contains("LSD") || e.content.contains("Mushrooms"));
    if used_psychedelic {
        earned.push(Achievement::PsychonautInit);
    }

    // ThoughtLeader — 5 internalized thoughts (cumulative, including forgotten ones)
    // Count both current internalized + journal entries for ThoughtInternalized
    let internalized_journal = ch.journal.iter()
        .filter(|e| matches!(e.entry_type, crate::journal::EntryType::ThoughtInternalized))
        .count();
    let currently_internalized = ch.thoughts.iter()
        .filter(|t| matches!(t.phase, ThoughtPhase::Internalized))
        .count();
    // Use journal count as cumulative (journal is append-only)
    if internalized_journal >= 5 || currently_internalized >= 5 {
        earned.push(Achievement::ThoughtLeader);
    }

    // WellDressed — all 6 slots equipped simultaneously
    let all_slots = [
        EquipSlot::Hat, EquipSlot::Jacket, EquipSlot::Shirt,
        EquipSlot::Pants, EquipSlot::Shoes, EquipSlot::Accessory,
    ];
    if all_slots.iter().all(|slot| ch.loadout.slots.get(slot).is_some()) {
        earned.push(Achievement::WellDressed);
    }

    // NightOwl — passed a check with timestamp hour 0–5
    let night_pass = ch.check_history.iter().any(|r| {
        r.success && {
            let hour = r.timestamp.time().hour();
            hour < 6
        }
    });
    if night_pass {
        earned.push(Achievement::NightOwl);
    }

    // CoffeeAddict — 10 coffee uses via journal
    let coffee_uses = ch.journal.iter()
        .filter(|e| matches!(e.entry_type, crate::journal::EntryType::SubstanceUsed) && e.content.contains("Coffee"))
        .count();
    if coffee_uses >= 10 {
        earned.push(Achievement::CoffeeAddict);
    }

    // IronWill — 10 consecutive successes
    if max_success_streak(&ch.check_history) >= 10 {
        earned.push(Achievement::IronWill);
    }

    // Unlucky — 5 consecutive failures
    if max_fail_streak(&ch.check_history) >= 5 {
        earned.push(Achievement::Unlucky);
    }

    // JackOfAllTrades — 12+ distinct skills with at least one success
    let distinct_passed_skills: HashSet<Skill> = ch.check_history.iter()
        .filter(|r| r.success)
        .map(|r| r.skill)
        .collect();
    if distinct_passed_skills.len() >= 12 {
        earned.push(Achievement::JackOfAllTrades);
    }

    earned
}

fn max_success_streak(history: &[crate::character::CheckRecord]) -> usize {
    let mut max = 0usize;
    let mut cur = 0usize;
    for r in history {
        if r.success { cur += 1; max = max.max(cur); } else { cur = 0; }
    }
    max
}

fn max_fail_streak(history: &[crate::character::CheckRecord]) -> usize {
    let mut max = 0usize;
    let mut cur = 0usize;
    for r in history {
        if !r.success { cur += 1; max = max.max(cur); } else { cur = 0; }
    }
    max
}

/// Format all achievements — earned ones highlighted, unearned dimmed.
pub fn format_achievements(earned: &HashSet<Achievement>) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n{}\n\n", "ACHIEVEMENTS".bold()));

    // Group by rarity in display order
    let order = [Rarity::Legendary, Rarity::Rare, Rarity::Uncommon, Rarity::Common];
    for rarity in &order {
        let group: Vec<Achievement> = Achievement::all().iter()
            .copied()
            .filter(|a| a.info().rarity == *rarity)
            .collect();
        if group.is_empty() { continue; }
        out.push_str(&format!("  {}\n", rarity.color(rarity.label())));
        for ach in &group {
            let info = ach.info();
            if earned.contains(ach) {
                let name = rarity.color(&format!("{} {}", info.icon, info.name));
                out.push_str(&format!("  {}  {}\n", name, info.description.dimmed()));
            } else {
                out.push_str(&format!("  {}  {}\n",
                    format!("  {}", info.name).dimmed(),
                    info.description.dimmed(),
                ));
            }
        }
        out.push('\n');
    }
    out
}
