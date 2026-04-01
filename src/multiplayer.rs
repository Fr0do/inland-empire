use crate::character::Character;
use crate::skills::{Attribute, Skill};
use crate::stats::compute_stats;
use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};

// ── Portable Format ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct PortableCharacter {
    pub version: u32,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub character: Character,
}

pub fn export_character(ch: &Character) -> Result<String, String> {
    let portable = PortableCharacter {
        version: 1,
        exported_at: chrono::Utc::now(),
        character: ch.clone(),
    };
    serde_json::to_string_pretty(&portable).map_err(|e| e.to_string())
}

pub fn import_character(json: &str) -> Result<Character, String> {
    let portable: PortableCharacter =
        serde_json::from_str(json).map_err(|e| format!("Invalid .ie.json: {e}"))?;
    if portable.version != 1 {
        return Err(format!("Unsupported format version: {}", portable.version));
    }
    let mut ch = portable.character;
    ch.migrate();
    Ok(ch)
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
    pub total_checks: usize,
    pub pass_rate: f64,
    pub best_streak: usize,
    pub worst_streak: usize,
    pub critical_successes: usize,
    pub top_skills: Vec<(Skill, i8)>,
    pub attributes: Vec<(Attribute, u8)>,
}

pub struct Comparison {
    pub left: CharacterSummary,
    pub right: CharacterSummary,
    pub verdicts: Vec<Verdict>,
}

fn summarize(ch: &Character) -> CharacterSummary {
    let stats = compute_stats(ch);
    let pass_rate = if stats.total_checks > 0 {
        stats.successes as f64 / stats.total_checks as f64 * 100.0
    } else {
        0.0
    };

    // Top 3 skills by effective value
    let mut skills: Vec<(Skill, i8)> = Skill::all()
        .iter()
        .map(|s| (*s, ch.effective_skill(*s)))
        .collect();
    skills.sort_by(|a, b| b.1.cmp(&a.1));
    let top_skills = skills.into_iter().take(3).collect();

    let attributes = vec![
        (Attribute::Intellect, ch.attributes.get(&Attribute::Intellect).copied().unwrap_or(1)),
        (Attribute::Psyche, ch.attributes.get(&Attribute::Psyche).copied().unwrap_or(1)),
        (Attribute::Physique, ch.attributes.get(&Attribute::Physique).copied().unwrap_or(1)),
        (Attribute::Motorics, ch.attributes.get(&Attribute::Motorics).copied().unwrap_or(1)),
    ];

    CharacterSummary {
        name: ch.name.clone(),
        archetype: ch.archetype.clone(),
        level: ch.level,
        total_xp: stats.total_xp,
        total_checks: stats.total_checks,
        pass_rate,
        best_streak: stats.best_streak,
        worst_streak: stats.worst_streak,
        critical_successes: stats.critical_successes,
        top_skills,
        attributes,
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
                &format!("{:.0}%", ls.pass_rate), &format!("{:.0}%", rs.pass_rate)),
        verdict("Total Checks", ls.total_checks as f64, rs.total_checks as f64,
                &ls.total_checks.to_string(), &rs.total_checks.to_string()),
        verdict("Best Streak", ls.best_streak as f64, rs.best_streak as f64,
                &ls.best_streak.to_string(), &rs.best_streak.to_string()),
        verdict("Worst Streak", rs.worst_streak as f64, ls.worst_streak as f64, // lower is better
                &ls.worst_streak.to_string(), &rs.worst_streak.to_string()),
        verdict("Critical ✓", ls.critical_successes as f64, rs.critical_successes as f64,
                &ls.critical_successes.to_string(), &rs.critical_successes.to_string()),
    ];

    // Attribute verdicts
    for (attr, lv) in &ls.attributes {
        let rv = rs.attributes.iter()
            .find(|(a, _)| a == attr)
            .map(|(_, v)| *v)
            .unwrap_or(1);
        verdicts.push(verdict(
            &attr.to_string(),
            *lv as f64, rv as f64,
            &lv.to_string(), &rv.to_string(),
        ));
    }

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
    } else if diff >= 4 {
        FLAVOR_DECISIVE
    } else {
        FLAVOR_CLOSE
    };

    let idx = rand::rng().random_range(0..pool.len());
    pool[idx]
}

pub fn generate_card(ch: &Character) -> String {
    // Get theme colors
    let t = crate::storybook::theme(ch.genre);
    let stats = compute_stats(ch);
    let pass_rate = if stats.total_checks > 0 {
        format!("{:.0}%", stats.successes as f64 / stats.total_checks as f64 * 100.0)
    } else {
        "—".to_string()
    };

    // Get the radar chart SVG (300x300) and portrait SVG (220x420)
    // We'll embed them scaled inside our card
    let radar = crate::dashboard::svg_radar_chart(ch);
    let portrait = crate::dashboard::svg_portrait(ch);

    let name = crate::storybook::escape_html(&ch.name);
    let archetype = crate::storybook::escape_html(&ch.archetype);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 420" width="800" height="420">
  <defs>
    <style>
      .card-bg {{ fill: {bg}; }}
      .card-border {{ fill: none; stroke: {border}; stroke-width: 2; }}
      .card-name {{ fill: {accent}; font-family: {header_font}; font-size: 22px; font-weight: bold; }}
      .card-sub {{ fill: {muted}; font-family: {font_stack}; font-size: 12px; letter-spacing: 0.1em; text-transform: uppercase; }}
      .card-stat-val {{ fill: {accent}; font-family: {font_stack}; font-size: 20px; font-weight: bold; }}
      .card-stat-label {{ fill: {muted}; font-family: {font_stack}; font-size: 9px; letter-spacing: 0.08em; text-transform: uppercase; }}
      .card-divider {{ stroke: {border}; stroke-width: 1; }}
      .card-brand {{ fill: {muted}; font-family: {font_stack}; font-size: 9px; letter-spacing: 0.15em; text-transform: uppercase; opacity: 0.5; }}
    </style>
  </defs>
  <!-- Background -->
  <rect class="card-bg" width="800" height="420" rx="8"/>
  <rect class="card-border" x="1" y="1" width="798" height="418" rx="8"/>

  <!-- Portrait (scaled down, left side) -->
  <g transform="translate(15, 0) scale(0.85)">
    {portrait}
  </g>

  <!-- Divider -->
  <line class="card-divider" x1="210" y1="20" x2="210" y2="400"/>

  <!-- Name & info -->
  <text class="card-name" x="230" y="45">{name}</text>
  <text class="card-sub" x="230" y="65">{archetype}  ·  Level {level}  ·  {genre}</text>

  <!-- Stats row -->
  <text class="card-stat-val" x="230" y="110">{level}</text>
  <text class="card-stat-label" x="230" y="125">Level</text>

  <text class="card-stat-val" x="310" y="110">{checks}</text>
  <text class="card-stat-label" x="310" y="125">Checks</text>

  <text class="card-stat-val" x="410" y="110">{pass_rate}</text>
  <text class="card-stat-label" x="410" y="125">Pass Rate</text>

  <text class="card-stat-val" x="510" y="110">{streak}</text>
  <text class="card-stat-label" x="510" y="125">Best Streak</text>

  <text class="card-stat-val" x="620" y="110">{crits}</text>
  <text class="card-stat-label" x="620" y="125">Criticals</text>

  <line class="card-divider" x1="230" y1="140" x2="780" y2="140"/>

  <!-- Radar chart (scaled, right side) -->
  <g transform="translate(380, 130) scale(0.88)">
    {radar}
  </g>

  <!-- Brand -->
  <text class="card-brand" x="230" y="405">Inland Empire</text>
</svg>"##,
        bg = t.bg,
        border = t.border,
        accent = t.accent,
        muted = t.muted,
        header_font = t.header_font,
        font_stack = t.font_stack,
        name = name,
        archetype = archetype,
        level = ch.level,
        genre = ch.genre,
        checks = stats.total_checks,
        pass_rate = pass_rate,
        streak = stats.best_streak,
        crits = stats.critical_successes,
        portrait = portrait,
        radar = radar,
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
    for (i, (skill, level)) in comp.left.top_skills.iter().enumerate() {
        let right_skill = comp.right.top_skills.get(i);
        let left_str = format!("{} {}", skill, level);
        let right_str = right_skill
            .map(|(s, l)| format!("{} {}", s, l))
            .unwrap_or_default();
        out.push_str(&format!("  {:>28}    {:<28}\n", left_str.cyan(), right_str.cyan()));
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
