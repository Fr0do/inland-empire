use crate::character::Character;
use crate::skills::Skill;
use crate::substances::Substance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objective: Objective,
    pub reward_xp: u32,
    pub reward_substance: Option<Substance>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Objective {
    CheckCount(usize),
    SkillChecks(Skill, usize),
    SuccessStreak(usize),
    ReachLevel(u32),
    InternalizeThoughts(usize),
    UseSubstances(usize),
    EquipSlots(usize),
    CriticalHits(usize),
}

pub fn all_cases() -> Vec<Case> {
    vec![
        Case {
            id: "rookie".into(),
            name: "The Rookie's First Case".into(),
            description: "Every detective starts somewhere. Pass your first ten checks.".into(),
            objective: Objective::CheckCount(10),
            reward_xp: 50,
            reward_substance: Some(Substance::Coffee),
            completed: false,
        },
        Case {
            id: "specialist".into(),
            name: "Tool Specialist".into(),
            description: "Master the interface. Pass twenty Interfacing checks.".into(),
            objective: Objective::SkillChecks(Skill::Interfacing, 20),
            reward_xp: 100,
            reward_substance: Some(Substance::Speed),
            completed: false,
        },
        Case {
            id: "streak".into(),
            name: "Hot Streak".into(),
            description: "Ride the wave. Get ten successes in a row.".into(),
            objective: Objective::SuccessStreak(10),
            reward_xp: 150,
            reward_substance: Some(Substance::Nostalgia),
            completed: false,
        },
        Case {
            id: "veteran".into(),
            name: "The Veteran".into(),
            description: "Survive long enough to call yourself experienced. Reach level 10.".into(),
            objective: Objective::ReachLevel(10),
            reward_xp: 200,
            reward_substance: Some(Substance::Pyrholidon),
            completed: false,
        },
        Case {
            id: "philosopher".into(),
            name: "Deep Thinker".into(),
            description: "The cabinet fills. Internalize five thoughts.".into(),
            objective: Objective::InternalizeThoughts(5),
            reward_xp: 100,
            reward_substance: None,
            completed: false,
        },
        Case {
            id: "experimenter".into(),
            name: "The Experimenter".into(),
            description: "Chemistry is a tool like any other. Use four different substances."
                .into(),
            objective: Objective::UseSubstances(4),
            reward_xp: 100,
            reward_substance: Some(Substance::Mushrooms),
            completed: false,
        },
        Case {
            id: "fashionista".into(),
            name: "Well-Dressed Detective".into(),
            description: "Appearance matters on the field. Fill six equipment slots.".into(),
            objective: Objective::EquipSlots(6),
            reward_xp: 100,
            reward_substance: None,
            completed: false,
        },
        Case {
            id: "lucky".into(),
            name: "Lady Luck".into(),
            description: "Double sixes. Five times. The universe is on your side.".into(),
            objective: Objective::CriticalHits(5),
            reward_xp: 150,
            reward_substance: Some(Substance::Lsd),
            completed: false,
        },
        Case {
            id: "grinder".into(),
            name: "The Grind".into(),
            description: "One hundred checks. The work never stops.".into(),
            objective: Objective::CheckCount(100),
            reward_xp: 300,
            reward_substance: Some(Substance::Pyrholidon),
            completed: false,
        },
        Case {
            id: "centurion".into(),
            name: "Case Centurion".into(),
            description: "Five hundred checks. You are the job now.".into(),
            objective: Objective::CheckCount(500),
            reward_xp: 500,
            reward_substance: None,
            completed: false,
        },
    ]
}

pub fn check_case_progress(case: &Case, ch: &Character) -> (usize, usize) {
    match &case.objective {
        Objective::CheckCount(target) => (ch.check_history.len().min(*target), *target),
        Objective::SkillChecks(skill, target) => {
            let count = ch
                .check_history
                .iter()
                .filter(|r| r.skill == *skill && r.success)
                .count();
            (count.min(*target), *target)
        }
        Objective::SuccessStreak(target) => {
            let streak = best_success_streak(ch);
            (streak.min(*target), *target)
        }
        Objective::ReachLevel(target) => (ch.level.min(*target) as usize, *target as usize),
        Objective::InternalizeThoughts(target) => {
            use crate::character::ThoughtPhase;
            let count = ch
                .thoughts
                .iter()
                .filter(|t| matches!(t.phase, ThoughtPhase::Internalized))
                .count();
            (count.min(*target), *target)
        }
        Objective::UseSubstances(target) => {
            // Count distinct substances ever used (appeared in inventory history via active effects)
            // We track via journal entries mentioning SubstanceUsed — approximate via unique substances
            // seen in inventory or active effects. Better: track in check_history context, but we
            // don't have that. Use a heuristic: count distinct substances ever listed in active_effects
            // by scanning check_history context strings is unreliable, so count unique substances
            // the player has at least used once (seen in active_effects or inventory depletion).
            // Since we don't have a dedicated log, we count unique substances currently known
            // (in inventory + active effects). This is an approximation; a dedicated counter
            // would require a schema addition. For now count unique substances seen in active_effects.
            let mut seen = std::collections::HashSet::new();
            for e in &ch.active_effects {
                seen.insert(e.substance);
            }
            // Also count substances in inventory as "encountered"
            for s in ch.inventory.keys() {
                seen.insert(*s);
            }
            (seen.len().min(*target), *target)
        }
        Objective::EquipSlots(target) => {
            let filled = ch.loadout.slots.len();
            (filled.min(*target), *target)
        }
        Objective::CriticalHits(target) => {
            let count = ch
                .check_history
                .iter()
                .filter(|r| r.roll.0 == 6 && r.roll.1 == 6)
                .count();
            (count.min(*target), *target)
        }
    }
}

fn best_success_streak(ch: &Character) -> usize {
    let mut best = 0usize;
    let mut current = 0usize;
    for r in &ch.check_history {
        if r.success {
            current += 1;
            best = best.max(current);
        } else {
            current = 0;
        }
    }
    best
}

pub fn format_cases(cases: &[Case], ch: &Character) -> String {
    use colored::Colorize;
    let mut out = String::new();
    out.push_str(&format!("\n{}\n\n", "CASES".bold()));

    let completed_count = cases.iter().filter(|c| c.completed).count();
    out.push_str(&format!(
        "  Progress: {}/{} completed\n\n",
        completed_count,
        cases.len()
    ));

    for case in cases {
        let (current, target) = check_case_progress(case, ch);
        let pct = if target == 0 {
            1.0
        } else {
            (current as f32 / target as f32).min(1.0)
        };
        let bar_len = 20usize;
        let filled = (pct * bar_len as f32).round() as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);

        if case.completed {
            out.push_str(&format!(
                "  {} {}\n",
                "✓".green().bold(),
                case.name.bold().green()
            ));
            out.push_str(&format!("    {}\n", case.description.dimmed()));
            out.push_str(&format!(
                "    {} Completed — {} XP rewarded\n\n",
                bar.green(),
                case.reward_xp
            ));
        } else if pct >= 1.0 {
            // Objective met but not yet claimed (will complete on next check)
            out.push_str(&format!(
                "  {} {}\n",
                "!".yellow().bold(),
                case.name.bold().yellow()
            ));
            out.push_str(&format!("    {}\n", case.description.dimmed()));
            let reward_str = match case.reward_substance {
                Some(sub) => format!("{} XP + {}", case.reward_xp, sub),
                None => format!("{} XP", case.reward_xp),
            };
            out.push_str(&format!(
                "    {} {}/{} — {} — Reward: {}\n\n",
                bar.yellow(),
                current,
                target,
                "OBJECTIVE MET — completes on next check".yellow().bold(),
                reward_str.cyan()
            ));
        } else {
            out.push_str(&format!("  {} {}\n", "○".dimmed(), case.name.bold()));
            out.push_str(&format!("    {}\n", case.description.dimmed()));
            let reward_str = match case.reward_substance {
                Some(sub) => format!("{} XP + {}", case.reward_xp, sub),
                None => format!("{} XP", case.reward_xp),
            };
            out.push_str(&format!(
                "    {} {}/{} — Reward: {}\n\n",
                bar.dimmed(),
                current,
                target,
                reward_str.cyan()
            ));
        }
    }
    out
}
