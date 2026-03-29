use crate::character::Character;
use crate::journal::EntryType;
use crate::skills::Skill;
use colored::Colorize;
use std::collections::HashMap;

pub struct Stats {
    pub total_checks: usize,
    pub successes: usize,
    pub failures: usize,
    pub critical_successes: usize,
    pub critical_failures: usize,
    pub current_streak: i32, // positive = success streak, negative = fail streak
    pub best_streak: usize,
    pub worst_streak: usize,
    pub by_skill: HashMap<Skill, (usize, usize)>, // (successes, total)
    pub by_tool: HashMap<String, usize>,           // tool -> count
    pub total_xp: u32,
    pub substances_used: HashMap<String, usize>,
}

pub fn compute_stats(ch: &Character) -> Stats {
    let mut by_skill: HashMap<Skill, (usize, usize)> = HashMap::new();
    let mut by_tool: HashMap<String, usize> = HashMap::new();
    let mut successes = 0usize;
    let mut failures = 0usize;
    let mut critical_successes = 0usize;
    let mut critical_failures = 0usize;

    // Streaks
    let mut current_streak: i32 = 0;
    let mut best_streak: usize = 0;
    let mut worst_streak: usize = 0;
    let mut running_success: usize = 0;
    let mut running_fail: usize = 0;

    for record in &ch.check_history {
        // by_skill
        let entry = by_skill.entry(record.skill).or_insert((0, 0));
        entry.1 += 1;
        if record.success {
            entry.0 += 1;
        }

        // by_tool — first token of context is the tool name
        let tool = record.context.split_whitespace().next().unwrap_or("unknown").to_string();
        *by_tool.entry(tool).or_insert(0) += 1;

        // overall counts
        if record.success {
            successes += 1;
            running_success += 1;
            running_fail = 0;
            if running_success > best_streak {
                best_streak = running_success;
            }
        } else {
            failures += 1;
            running_fail += 1;
            running_success = 0;
            if running_fail > worst_streak {
                worst_streak = running_fail;
            }
        }

        // criticals
        if record.roll.0 == 6 && record.roll.1 == 6 {
            critical_successes += 1;
        } else if record.roll.0 == 1 && record.roll.1 == 1 {
            critical_failures += 1;
        }
    }

    // Current streak from end of history
    if let Some(last) = ch.check_history.last() {
        if last.success {
            // count backwards while success
            current_streak = ch.check_history.iter().rev().take_while(|r| r.success).count() as i32;
        } else {
            current_streak = -(ch.check_history.iter().rev().take_while(|r| !r.success).count() as i32);
        }
    }

    // Total XP: 10 per success, 5 per failure (matches record_check logic)
    let total_xp = (successes as u32 * 10) + (failures as u32 * 5);

    // Substances from journal
    let mut substances_used: HashMap<String, usize> = HashMap::new();
    for entry in &ch.journal {
        if entry.entry_type == EntryType::SubstanceUsed {
            // Extract substance name from content — look for known patterns across genres.
            // All templates reference the substance in first few words; extract bracketed or bare name.
            // We parse out the substance from the content heuristically.
            let substance = extract_substance_from_entry(&entry.content);
            if !substance.is_empty() {
                *substances_used.entry(substance).or_insert(0) += 1;
            }
        }
    }

    Stats {
        total_checks: ch.check_history.len(),
        successes,
        failures,
        critical_successes,
        critical_failures,
        current_streak,
        best_streak,
        worst_streak,
        by_skill,
        by_tool,
        total_xp,
        substances_used,
    }
}

/// Heuristically extract the substance name from a journal entry's content.
/// Handles all genre templates which embed the substance as a bare word or after "the".
fn extract_substance_from_entry(content: &str) -> String {
    // Try "the <word>" pattern — e.g. "Reached for the Coffee."
    if let Some(pos) = content.find("the ") {
        let after = &content[pos + 4..];
        let word: String = after
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .next()
            .unwrap_or("")
            .to_string();
        if !word.is_empty() {
            return word;
        }
    }
    // Fallback: first capitalised word that isn't a sentence opener
    let words: Vec<&str> = content.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
        if clean.len() > 2 && clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && i > 0 {
            return clean;
        }
    }
    String::new()
}

fn bar(fraction: f32, width: usize, filled_char: &str, empty_char: &str) -> String {
    let filled = ((fraction * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!("{}{}", filled_char.repeat(filled), empty_char.repeat(empty))
}

pub fn format_stats(stats: &Stats) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(String::new());
    lines.push(format!("{}", "CASE FILE — STATISTICS".bold()));
    lines.push(format!("{}", "─".repeat(40).dimmed()));

    // ── Overall ───────────────────────────────────────────────────────────────
    lines.push(String::new());
    lines.push(format!("{}", "OVERALL".bold()));

    let success_rate = if stats.total_checks > 0 {
        stats.successes as f32 / stats.total_checks as f32
    } else {
        0.0
    };
    let rate_pct = (success_rate * 100.0).round() as u32;
    let filled = bar(success_rate, 20, "█", "░");
    let bar_colored = if success_rate >= 0.6 {
        filled.green().to_string()
    } else if success_rate >= 0.4 {
        filled.yellow().to_string()
    } else {
        filled.red().to_string()
    };

    lines.push(format!(
        "  Total checks  {}",
        stats.total_checks.to_string().bold()
    ));
    lines.push(format!(
        "  Success rate  {} {}%",
        bar_colored,
        rate_pct.to_string().bold()
    ));
    lines.push(format!(
        "  Successes     {}  Failures  {}",
        stats.successes.to_string().green().bold(),
        stats.failures.to_string().red().bold()
    ));
    lines.push(format!(
        "  Total XP      {}",
        stats.total_xp.to_string().yellow()
    ));

    // ── Criticals ─────────────────────────────────────────────────────────────
    lines.push(String::new());
    lines.push(format!("{}", "CRITICALS".bold()));
    lines.push(format!(
        "  Crit successes  {}  (double-6)",
        stats.critical_successes.to_string().green().bold()
    ));
    lines.push(format!(
        "  Crit failures   {}  (double-1)",
        stats.critical_failures.to_string().red().bold()
    ));

    // ── Streaks ───────────────────────────────────────────────────────────────
    lines.push(String::new());
    lines.push(format!("{}", "STREAKS".bold()));
    let streak_str = if stats.current_streak > 0 {
        format!(
            "{} success streak",
            stats.current_streak.to_string().green().bold()
        )
    } else if stats.current_streak < 0 {
        format!(
            "{} fail streak",
            (-stats.current_streak).to_string().red().bold()
        )
    } else {
        "none".dimmed().to_string()
    };
    lines.push(format!("  Current   {}", streak_str));
    lines.push(format!(
        "  Best win  {}  Worst loss  {}",
        stats.best_streak.to_string().green(),
        stats.worst_streak.to_string().red()
    ));

    // ── By Skill (top 5 most used) ────────────────────────────────────────────
    if !stats.by_skill.is_empty() {
        lines.push(String::new());
        lines.push(format!("{}", "BY SKILL  (top 5)".bold()));

        let mut skill_vec: Vec<(Skill, usize, usize)> = stats
            .by_skill
            .iter()
            .map(|(s, (succ, total))| (*s, *succ, *total))
            .collect();
        skill_vec.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));

        for (skill, succ, total) in skill_vec.iter().take(5) {
            let rate = if *total > 0 {
                *succ as f32 / *total as f32
            } else {
                0.0
            };
            let pct = (rate * 100.0).round() as u32;
            let mini = bar(rate, 10, "█", "░");
            let mini_colored = if rate >= 0.6 {
                mini.green().to_string()
            } else if rate >= 0.4 {
                mini.yellow().to_string()
            } else {
                mini.red().to_string()
            };
            lines.push(format!(
                "  {:20} {} {:3}%  ({}/{})",
                skill.to_string().bold(),
                mini_colored,
                pct,
                succ,
                total
            ));
        }

        // Most and least used
        let most = skill_vec.first().map(|(s, _, t)| (s, t));
        let least = skill_vec.last().map(|(s, _, t)| (s, t));
        if let (Some((most_s, most_t)), Some((least_s, least_t))) = (most, least) {
            if most_s != least_s {
                lines.push(String::new());
                lines.push(format!(
                    "  Most used   {} ({}x)",
                    most_s.to_string().cyan(),
                    most_t
                ));
                lines.push(format!(
                    "  Least used  {} ({}x)",
                    least_s.to_string().dimmed(),
                    least_t
                ));
            }
        }
    }

    // ── By Tool ───────────────────────────────────────────────────────────────
    if !stats.by_tool.is_empty() {
        lines.push(String::new());
        lines.push(format!("{}", "BY TOOL  (top 5)".bold()));

        let mut tool_vec: Vec<(&String, usize)> =
            stats.by_tool.iter().map(|(t, c)| (t, *c)).collect();
        tool_vec.sort_by(|a, b| b.1.cmp(&a.1));

        for (tool, count) in tool_vec.iter().take(5) {
            lines.push(format!(
                "  {:20} {}",
                tool.bold(),
                count.to_string().dimmed()
            ));
        }
    }

    // ── Substances ────────────────────────────────────────────────────────────
    if !stats.substances_used.is_empty() {
        lines.push(String::new());
        lines.push(format!("{}", "SUBSTANCES".bold()));
        let mut sub_vec: Vec<(&String, usize)> =
            stats.substances_used.iter().map(|(s, c)| (s, *c)).collect();
        sub_vec.sort_by(|a, b| b.1.cmp(&a.1));
        for (sub, count) in &sub_vec {
            lines.push(format!(
                "  {} {}x",
                sub.magenta(),
                count.to_string().bold()
            ));
        }
    }

    lines.push(String::new());
    lines.join("\n")
}
