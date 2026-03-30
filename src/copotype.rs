use crate::character::Character;
use crate::skills::Skill;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Copotype {
    Architect,
    Cowboy,
    Craftsman,
    Empath,
    Mystic,
    Enforcer,
    Scholar,
    Bruteforcer,
}

pub struct CopotypeInfo {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub primary_skills: [Skill; 2],
    pub catchphrase: &'static str,
}

impl Copotype {
    pub fn info(self) -> CopotypeInfo {
        match self {
            Copotype::Architect => CopotypeInfo {
                name: "Architect",
                title: "Systems Thinker, First Class",
                description: "You see the scaffolding behind every system. The diagram comes before the code — always.",
                primary_skills: [Skill::Logic, Skill::Conceptualization],
                catchphrase: "Let me draw this out before we write a single line.",
            },
            Copotype::Cowboy => CopotypeInfo {
                name: "Cowboy",
                title: "Reckless Deploy Specialist",
                description: "You live fast and push hard. Production is just another staging environment.",
                primary_skills: [Skill::Electrochemistry, Skill::ReactionSpeed],
                catchphrase: "Ship it. We'll fix it in prod.",
            },
            Copotype::Craftsman => CopotypeInfo {
                name: "Craftsman",
                title: "Artisan of Clean Code",
                description: "Every variable name is a choice. Every function a small act of devotion.",
                primary_skills: [Skill::HandEyeCoordination, Skill::Perception],
                catchphrase: "Every line matters. Every test matters.",
            },
            Copotype::Empath => CopotypeInfo {
                name: "Empath",
                title: "User Whisperer",
                description: "You feel the user's confusion before they've typed the bug report.",
                primary_skills: [Skill::Empathy, Skill::Esprit],
                catchphrase: "But how does the USER feel about this?",
            },
            Copotype::Mystic => CopotypeInfo {
                name: "Mystic",
                title: "Code Intuition Officer",
                description: "The bug reveals itself to you as a shiver — a wrongness in the codebase's soul.",
                primary_skills: [Skill::InlandEmpire, Skill::Shivers],
                catchphrase: "I can feel the bug. It's in the auth module.",
            },
            Copotype::Enforcer => CopotypeInfo {
                name: "Enforcer",
                title: "Standards Compliance Marshal",
                description: "Style guides are not suggestions. The linter is not your enemy. It is the law.",
                primary_skills: [Skill::Authority, Skill::Volition],
                catchphrase: "The linter says no. I say no.",
            },
            Copotype::Scholar => CopotypeInfo {
                name: "Scholar",
                title: "Knowledge Curator",
                description: "You have read the RFC. All of them. You will cite them.",
                primary_skills: [Skill::Encyclopedia, Skill::Rhetoric],
                catchphrase: "Actually, according to the RFC...",
            },
            Copotype::Bruteforcer => CopotypeInfo {
                name: "Bruteforcer",
                title: "Raw Computational Power",
                description: "Why optimize when you can parallelize? Why parallelize when you can throw hardware at it?",
                primary_skills: [Skill::PhysicalInstrument, Skill::Endurance],
                catchphrase: "More threads. More memory. Problem solved.",
            },
        }
    }

    pub fn all() -> &'static [Copotype] {
        &[
            Copotype::Architect,
            Copotype::Cowboy,
            Copotype::Craftsman,
            Copotype::Empath,
            Copotype::Mystic,
            Copotype::Enforcer,
            Copotype::Scholar,
            Copotype::Bruteforcer,
        ]
    }
}

pub fn detect_copotype(ch: &Character) -> Copotype {
    let mut best = Copotype::Architect;
    let mut best_score = i16::MIN;

    for &copotype in Copotype::all() {
        let info = copotype.info();
        let score: i16 = info
            .primary_skills
            .iter()
            .map(|&s| ch.effective_skill(s) as i16)
            .sum();
        if score > best_score {
            best_score = score;
            best = copotype;
        }
    }

    best
}

pub fn format_copotype(copotype: Copotype, ch: &Character) -> String {
    let info = copotype.info();
    let [s1, s2] = info.primary_skills;
    let v1 = ch.effective_skill(s1);
    let v2 = ch.effective_skill(s2);

    let mut out = String::new();

    out.push_str(&format!(
        "\n{}\n",
        "╔══════════════════════════════════════╗".cyan()
    ));
    out.push_str(&format!(
        "{}  {:<36}{}\n",
        "║".cyan(),
        format!("COPOTYPE: {}", info.name.to_uppercase()),
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}  {:<36}{}\n",
        "║".cyan(),
        info.title.italic().to_string(),
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}\n",
        "╠══════════════════════════════════════╣".cyan()
    ));
    out.push_str(&format!(
        "{}  {:<36}{}\n",
        "║".cyan(),
        info.description,
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}\n",
        "╠══════════════════════════════════════╣".cyan()
    ));
    out.push_str(&format!(
        "{}  Primary Skills:{:<20}{}\n",
        "║".cyan(),
        "",
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}    {:<16} {:<18}{}\n",
        "║".cyan(),
        format!("{}", s1),
        format_skill_bar(v1),
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}    {:<16} {:<18}{}\n",
        "║".cyan(),
        format!("{}", s2),
        format_skill_bar(v2),
        "║".cyan()
    ));
    out.push_str(&format!(
        "{}\n",
        "╠══════════════════════════════════════╣".cyan()
    ));
    out.push_str(&format!(
        "{}  \"{}\"\n",
        "║".cyan(),
        info.catchphrase.italic()
    ));
    out.push_str(&format!(
        "{}\n",
        "╚══════════════════════════════════════╝".cyan()
    ));

    out
}

fn format_skill_bar(val: i8) -> String {
    let clamped = val.max(0) as usize;
    let filled = clamped.min(10);
    let bar: String = "█".repeat(filled) + &"░".repeat(10usize.saturating_sub(filled));
    format!("{} {}", bar.yellow(), val)
}
