use crate::character::{Character, ThoughtPhase};
use crate::equipment;
use crate::skills::{Attribute, Skill};
use crate::substances::Substance;
use crate::time::TimeOfDay;
use colored::Colorize;

pub fn character_sheet(ch: &Character) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n{}  —  {}  (Level {})\n",
        ch.name.bold(),
        ch.archetype.dimmed(),
        ch.level
    ));
    out.push_str(&format!(
        "XP: {}/{}  |  Skill points: {}\n",
        ch.xp,
        ch.xp_to_next_level(),
        ch.skill_points
    ));
    let hearts: String =
        "❤️".repeat(ch.health as usize) + &"🖤".repeat((ch.max_health - ch.health) as usize);
    let thoughts: String =
        "💙".repeat(ch.morale as usize) + &"🩶".repeat((ch.max_morale - ch.morale) as usize);
    out.push_str(&format!(
        "Health: {} ({}/{})  |  Morale: {} ({}/{})\n",
        hearts, ch.health, ch.max_health, thoughts, ch.morale, ch.max_morale
    ));
    out.push_str(&"─".repeat(50));
    out.push('\n');
    for attr in &[
        Attribute::Intellect,
        Attribute::Psyche,
        Attribute::Physique,
        Attribute::Motorics,
    ] {
        let attr_val = ch.attributes.get(attr).copied().unwrap_or(1);
        let attr_str = format!("{} [{}]", attr, attr_val);
        let colored_attr = match attr {
            Attribute::Intellect => attr_str.blue().bold().to_string(),
            Attribute::Psyche => attr_str.magenta().bold().to_string(),
            Attribute::Physique => attr_str.red().bold().to_string(),
            Attribute::Motorics => attr_str.yellow().bold().to_string(),
        };
        out.push_str(&format!("\n{}\n", colored_attr));
        for skill in Skill::all() {
            if skill.attribute() == *attr {
                let base = *ch.skills.get(skill).unwrap_or(&1);
                let effective = ch.effective_skill(*skill);
                let diff = effective - base as i8;
                let bonus = if diff > 0 {
                    format!(" (+{})", diff).green().to_string()
                } else if diff < 0 {
                    format!(" ({})", diff).red().to_string()
                } else {
                    String::new()
                };
                let bar = "█".repeat(effective.max(0) as usize);
                let bar_colored = match attr {
                    Attribute::Intellect => bar.blue().to_string(),
                    Attribute::Psyche => bar.magenta().to_string(),
                    Attribute::Physique => bar.red().to_string(),
                    Attribute::Motorics => bar.yellow().to_string(),
                };
                let prefix = if ch.signature_skill == Some(*skill) {
                    "★ "
                } else {
                    "  "
                };
                out.push_str(&format!(
                    "{}{:24} {:>2}{}  {}\n",
                    prefix,
                    skill.to_string(),
                    effective,
                    bonus,
                    bar_colored
                ));
            }
        }
    }
    if !ch.active_effects.is_empty() {
        out.push_str(&format!("\n{}\n", "ACTIVE EFFECTS".bold()));
        for effect in &ch.active_effects {
            let mods: Vec<String> = effect
                .skill_modifiers
                .iter()
                .map(|(s, v)| {
                    if *v >= 0 {
                        format!("{} +{}", s, v).green().to_string()
                    } else {
                        format!("{} {}", s, v).red().to_string()
                    }
                })
                .collect();
            out.push_str(&format!(
                "  {} ({} checks left)  {}\n",
                effect.substance.to_string().magenta(),
                effect.checks_remaining,
                mods.join(", ")
            ));
        }
    }
    if !ch.inventory.is_empty() {
        out.push_str(&format!("\n{}\n", "INVENTORY".bold()));
        for substance in Substance::all() {
            if let Some(&count) = ch.inventory.get(substance) {
                if count > 0 {
                    out.push_str(&format!(
                        "  {} x{} — {}\n",
                        substance,
                        count,
                        substance.info().description.dimmed()
                    ));
                }
            }
        }
    }
    if !ch.loadout.slots.is_empty() {
        out.push_str(&equipment::format_loadout(&ch.loadout));
    }
    if !ch.thoughts.is_empty() {
        out.push_str(&format!("\n{}\n", "THOUGHT CABINET".bold()));
        for thought in &ch.thoughts {
            match thought.phase {
                ThoughtPhase::Researching { checks_remaining } => {
                    let mods: Vec<String> = thought
                        .research_modifiers
                        .iter()
                        .map(|(s, v)| {
                            if *v >= 0 {
                                format!("{} +{}", s, v).green().to_string()
                            } else {
                                format!("{} {}", s, v).red().to_string()
                            }
                        })
                        .collect();
                    let penalties = if mods.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", mods.join(", "))
                    };
                    out.push_str(&format!(
                        "  {} {} ({} checks left){}\n    {}\n",
                        "⟳".yellow(),
                        thought.name.italic().yellow(),
                        checks_remaining,
                        penalties,
                        thought.description.dimmed()
                    ));
                }
                ThoughtPhase::Internalized => {
                    let mods: Vec<String> = thought
                        .skill_modifiers
                        .iter()
                        .map(|(s, v)| {
                            if *v >= 0 {
                                format!("{} +{}", s, v).green().to_string()
                            } else {
                                format!("{} {}", s, v).red().to_string()
                            }
                        })
                        .collect();
                    let bonuses = if mods.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", mods.join(", "))
                    };
                    out.push_str(&format!(
                        "  {} {}{}\n    {}\n",
                        "✓".green(),
                        thought.name.italic(),
                        bonuses,
                        thought.description.dimmed()
                    ));
                }
            }
        }
    }
    let recent: Vec<_> = ch.check_history.iter().rev().take(5).collect();
    if !recent.is_empty() {
        out.push_str(&format!("\n{}\n", "RECENT CHECKS".bold()));
        for r in recent {
            let icon = if r.success { "✓" } else { "✗" };
            let colored_icon = if r.success {
                icon.green().to_string()
            } else {
                icon.red().to_string()
            };
            out.push_str(&format!(
                "  {} {} ({}+{}={} vs {}) {}\n",
                colored_icon,
                r.skill,
                r.roll.0 + r.roll.1,
                r.modifier,
                r.total,
                r.difficulty,
                r.context.dimmed()
            ));
        }
    }
    out
}

pub fn status_line(ch: &Character) -> String {
    let time = TimeOfDay::current();
    format!(
        "{} Lv{} | XP {}/{} | SP {} | ❤{}/{} 💭{}/{} | {} {}",
        ch.name,
        ch.level,
        ch.xp,
        ch.xp_to_next_level(),
        ch.skill_points,
        ch.health,
        ch.max_health,
        ch.morale,
        ch.max_morale,
        time.icon(),
        time.label()
    )
}

pub fn status_oneline(ch: &Character) -> String {
    let time = TimeOfDay::current();
    let hearts =
        "❤".repeat(ch.health as usize) + &"🖤".repeat((ch.max_health - ch.health) as usize);
    let morale_bar =
        "💙".repeat(ch.morale as usize) + &"🩶".repeat((ch.max_morale - ch.morale) as usize);
    let sig = ch
        .signature_skill
        .map(|s| format!("★{}", s))
        .unwrap_or_default();
    format!(
        "{} L{} {} {} {} XP:{}/{} {}{}",
        ch.name,
        ch.level,
        hearts,
        morale_bar,
        time.icon(),
        ch.xp,
        ch.xp_to_next_level(),
        sig,
        if ch.skill_points > 0 {
            format!(" +{}SP", ch.skill_points)
        } else {
            String::new()
        }
    )
}
