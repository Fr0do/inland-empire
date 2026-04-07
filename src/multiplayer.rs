use crate::character::Character;
use crate::skills::Skill;
use crate::stats::compute_stats;
use chrono::{DateTime, Utc};
use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};

// ── Portable Format ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct PortableCharacter {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub character: Character,
}

pub fn export_character(ch: &Character) -> Result<String, String> {
    let portable = PortableCharacter {
        version: 1,
        exported_at: Utc::now(),
        character: ch.clone(),
    };
    serde_json::to_string_pretty(&portable).map_err(|e| e.to_string())
}

pub fn import_character(json: &str) -> Result<Character, String> {
    let portable: PortableCharacter = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(portable.character)
}

pub fn resolve_name_conflict(name: &str, existing: &[String]) -> String {
    if !existing.contains(&name.to_string()) {
        return name.to_string();
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{name}-{n}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

// ── Comparison ───────────────────────────────────────────────────────────────

pub enum Winner {
    Left,
    Right,
    Tie,
}

pub struct Verdict {
    pub category: String,
    pub left_val: String,
    pub right_val: String,
    pub winner: Winner,
}

pub struct CharacterSummary {
    pub name: String,
    pub archetype: String,
    pub level: u32,
    pub total_xp: u32,
    pub pass_rate: f64,    // 0.0–1.0
    pub best_streak: u32,
    pub top_skills: Vec<String>,  // top 3 skills by usage
}

pub struct Comparison {
    pub left: CharacterSummary,
    pub right: CharacterSummary,
    pub verdicts: Vec<Verdict>,
}

pub fn summarize(ch: &Character) -> CharacterSummary {
    let stats = compute_stats(ch);
    let pass_rate = if stats.total_checks > 0 {
        stats.successes as f64 / stats.total_checks as f64
    } else {
        0.0
    };

    // Top 3 skills by usage (from check_history)
    let mut skill_counts: std::collections::HashMap<Skill, usize> = std::collections::HashMap::new();
    for record in &ch.check_history {
        *skill_counts.entry(record.skill).or_insert(0) += 1;
    }
    let mut skill_vec: Vec<(Skill, usize)> = skill_counts.into_iter().collect();
    skill_vec.sort_by(|a, b| b.1.cmp(&a.1));
    let top_skills = skill_vec.into_iter().take(3).map(|(s, _)| s.to_string()).collect();

    CharacterSummary {
        name: ch.name.clone(),
        archetype: ch.archetype.clone(),
        level: ch.level,
        total_xp: stats.total_xp,
        pass_rate,
        best_streak: stats.best_streak as u32,
        top_skills,
    }
}

fn verdict(category: &str, left: f64, right: f64, fmt_left: &str, fmt_right: &str) -> Verdict {
    let winner = if left > right {
        Winner::Left
    } else if right > left {
        Winner::Right
    } else {
        Winner::Tie
    };
    Verdict {
        category: category.to_string(),
        left_val: fmt_left.to_string(),
        right_val: fmt_right.to_string(),
        winner,
    }
}

pub fn compare(left: &Character, right: &Character) -> Comparison {
    let ls = summarize(left);
    let rs = summarize(right);

    let mut verdicts = vec![
        verdict("Level", ls.level as f64, rs.level as f64,
                &ls.level.to_string(), &rs.level.to_string()),
        verdict("Total XP", ls.total_xp as f64, rs.total_xp as f64,
                &ls.total_xp.to_string(), &rs.total_xp.to_string()),
        verdict("Pass Rate", ls.pass_rate, rs.pass_rate,
                &format!("{:.1}%", ls.pass_rate * 100.0), &format!("{:.1}%", rs.pass_rate * 100.0)),
        verdict("Best Streak", ls.best_streak as f64, rs.best_streak as f64,
                &ls.best_streak.to_string(), &rs.best_streak.to_string()),
    ];

    // Top Skill (comparing top skill name might be weird, but let's just pick one category for it)
    let left_top = ls.top_skills.first().cloned().unwrap_or_else(|| "None".to_string());
    let right_top = rs.top_skills.first().cloned().unwrap_or_else(|| "None".to_string());
    verdicts.push(Verdict {
        category: "Top Skill".to_string(),
        left_val: left_top,
        right_val: right_top,
        winner: Winner::Tie, // Skills are incomparable by value here
    });

    Comparison { left: ls, right: rs, verdicts }
}

// ── DE Flavor Text ───────────────────────────────────────────────────────────

const FLAVOR_DECISIVE: &[&str] = &[
    "AUTHORITY [Trivial: Success] — It isn't even close.",
    "ENCYCLOPEDIA [Medium: Success] — The historical record favors one detective clearly.",
    "LOGIC [Easy: Success] — The numbers don't lie. One of you is simply better.",
    "DRAMA [Challenging: Success] — Even the blind can see who the real star is.",
    "PHYSICAL INSTRUMENT [Easy: Success] — Raw power speaks for itself.",
];

const FLAVOR_CLOSE: &[&str] = &[
    "INLAND EMPIRE [Heroic: Failure] — Two reflections in a cracked mirror. Which is real?",
    "EMPATHY [Challenging: Success] — You are more alike than different.",
    "COMPOSURE [Medium: Success] — Evenly matched. The dice could tip either way.",
    "SHIVERS [Formidable: Success] — The city holds its breath. Both could be the one.",
    "SUGGESTION [Medium: Success] — You could convince anyone either is the better detective.",
];

const FLAVOR_TIE: &[&str] = &[
    "SHIVERS [Legendary: Success] — The city sees two of the same soul.",
    "HALF LIGHT [Formidable: Failure] — Neither stands above. Both stand in shadow.",
    "VOLITION [Medium: Success] — Equal in will. Equal in purpose.",
];

fn pick_flavor(comp: &Comparison) -> &'static str {
    let left_wins = comp.verdicts.iter().filter(|v| matches!(v.winner, Winner::Left)).count();
    let right_wins = comp.verdicts.iter().filter(|v| matches!(v.winner, Winner::Right)).count();
    let diff = (left_wins as i32 - right_wins as i32).unsigned_abs() as usize;

    let pool = if diff == 0 {
        FLAVOR_TIE
    } else if diff >= 3 {
        FLAVOR_DECISIVE
    } else {
        FLAVOR_CLOSE
    };

    let idx = rand::rng().random_range(0..pool.len());
    pool[idx]
}

pub fn generate_card(ch: &Character) -> String {
    let t = crate::storybook::theme(ch.genre);
    let stats = crate::stats::compute_stats(ch);
    let summary = summarize(ch);

    let pass_rate = if stats.total_checks > 0 {
        format!("{:.1}%", (stats.successes as f64 / stats.total_checks as f64) * 100.0)
    } else {
        "0%".to_string()
    };

    let top_skill = summary.top_skills.first().cloned().unwrap_or_else(|| "None".to_string());

    let portrait_svg = crate::dashboard::svg_portrait(ch);
    let radar_svg = crate::dashboard::svg_radar_chart(ch);

    let name = crate::storybook::escape_html(&ch.name);
    let archetype = crate::storybook::escape_html(&ch.archetype);

    format!(
        r##"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg width="600" height="400" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg">
  <rect width="600" height="400" fill="#1a1a2e" />
  
  <!-- Left Section: Portrait -->
  <g transform="translate(-10, -10) scale(0.95)">
    {portrait_svg}
  </g>

  <!-- Right Section: Radar Chart -->
  <g transform="translate(410, 80) scale(0.65)">
    {radar_svg}
  </g>

  <!-- Center Strip: Key Stats -->
  <g transform="translate(200, 40)">
    <text x="100" y="20" fill="{accent}" font-family="{header_font}" font-size="28" font-weight="bold" text-anchor="middle">{name}</text>
    <text x="100" y="45" fill="{muted}" font-family="{font_stack}" font-size="14" text-anchor="middle" letter-spacing="0.1em">{archetype}</text>
    
    <line x1="20" y1="70" x2="180" y2="70" stroke="{border}" stroke-width="1" opacity="0.5" />
    
    <g transform="translate(0, 90)">
        <text x="100" y="0" fill="{muted}" font-family="{font_stack}" font-size="10" text-anchor="middle" text-transform="uppercase">Level</text>
        <text x="100" y="25" fill="{accent}" font-family="{font_stack}" font-size="24" font-weight="bold" text-anchor="middle">{level}</text>
    </g>

    <g transform="translate(0, 150)">
        <text x="100" y="0" fill="{muted}" font-family="{font_stack}" font-size="10" text-anchor="middle" text-transform="uppercase">Pass Rate</text>
        <text x="100" y="25" fill="{accent}" font-family="{font_stack}" font-size="24" font-weight="bold" text-anchor="middle">{pass_rate}</text>
    </g>

    <g transform="translate(0, 210)">
        <text x="100" y="0" fill="{muted}" font-family="{font_stack}" font-size="10" text-anchor="middle" text-transform="uppercase">Best Streak</text>
        <text x="100" y="25" fill="{accent}" font-family="{font_stack}" font-size="24" font-weight="bold" text-anchor="middle">{streak}</text>
    </g>

    <g transform="translate(0, 270)">
        <text x="100" y="0" fill="{muted}" font-family="{font_stack}" font-size="10" text-anchor="middle" text-transform="uppercase">Top Skill</text>
        <text x="100" y="25" fill="{accent}" font-family="{font_stack}" font-size="18" font-weight="bold" text-anchor="middle">{top_skill}</text>
    </g>

    <line x1="20" y1="320" x2="180" y2="320" stroke="{border}" stroke-width="1" opacity="0.5" />
    <text x="100" y="345" fill="{muted}" font-family="{font_stack}" font-size="9" text-anchor="middle" letter-spacing="0.2em" opacity="0.4">INLAND EMPIRE</text>
  </g>
</svg>
"##,
        accent = t.accent,
        muted = t.muted,
        border = t.border,
        header_font = t.header_font,
        font_stack = t.font_stack,
        name = name,
        archetype = archetype,
        level = ch.level,
        pass_rate = pass_rate,
        streak = stats.best_streak,
        top_skill = top_skill,
        portrait_svg = portrait_svg,
        radar_svg = radar_svg,
    )
}


pub fn format_comparison(comp: &Comparison) -> String {
    let mut out = String::new();

    // Header with flavor
    let flavor = pick_flavor(comp);
    out.push_str(&format!("\n{}\n", flavor.dimmed()));
    out.push_str(&format!("\n{}", "═".repeat(60).dimmed()));
    out.push_str(&format!("\n{:>28} vs {:<28}\n",
        comp.left.name.bold(), comp.right.name.bold()));
    out.push_str(&format!("{:>28}    {:<28}\n",
        comp.left.archetype.dimmed(), comp.right.archetype.dimmed()));
    out.push_str(&format!("{}\n", "═".repeat(60).dimmed()));

    // Verdicts table
    for v in &comp.verdicts {
        let (left_colored, right_colored) = match v.winner {
            Winner::Left => (
                v.left_val.green().bold().to_string(),
                v.right_val.red().to_string(),
            ),
            Winner::Right => (
                v.left_val.red().to_string(),
                v.right_val.green().bold().to_string(),
            ),
            Winner::Tie => (
                v.left_val.yellow().to_string(),
                v.right_val.yellow().to_string(),
            ),
        };
        let indicator = match v.winner {
            Winner::Left => "◄".green().to_string(),
            Winner::Right => "►".green().to_string(),
            Winner::Tie => "═".yellow().to_string(),
        };
        out.push_str(&format!("  {:>12}  {:>8} {} {:<8}  {}\n",
            v.category.dimmed(),
            left_colored,
            indicator,
            right_colored,
            "",
        ));
    }

    out.push_str(&format!("{}\n", "─".repeat(60).dimmed()));

    // Top skills comparison
    out.push_str(&format!("\n{}  Top Skills\n", "★".yellow()));
    let max_skills = comp.left.top_skills.len().max(comp.right.top_skills.len());
    for i in 0..max_skills {
        let left_skill = comp.left.top_skills.get(i).cloned().unwrap_or_default();
        let right_skill = comp.right.top_skills.get(i).cloned().unwrap_or_default();
        out.push_str(&format!("  {:>28}    {:<28}\n", left_skill.cyan(), right_skill.cyan()));
    }

    out.push('\n');

    // Overall winner
    let left_wins = comp.verdicts.iter().filter(|v| matches!(v.winner, Winner::Left)).count();
    let right_wins = comp.verdicts.iter().filter(|v| matches!(v.winner, Winner::Right)).count();
    let ties = comp.verdicts.iter().filter(|v| matches!(v.winner, Winner::Tie)).count();

    let result = if left_wins > right_wins {
        format!("{} wins {}-{} ({} ties)", comp.left.name.green().bold(), left_wins, right_wins, ties)
    } else if right_wins > left_wins {
        format!("{} wins {}-{} ({} ties)", comp.right.name.green().bold(), right_wins, left_wins, ties)
    } else {
        format!("{}", "Dead even.".yellow().bold())
    };
    out.push_str(&format!("  {result}\n\n"));

    out
}
