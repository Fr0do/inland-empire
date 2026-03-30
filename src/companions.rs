use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Companion {
    Detective,
    Kim,
    Cuno,
}

impl fmt::Display for Companion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Companion::Detective => write!(f, "The Detective"),
            Companion::Kim => write!(f, "Kim Kitsuragi"),
            Companion::Cuno => write!(f, "Cuno"),
        }
    }
}

impl FromStr for Companion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "opus" | "detective" => Ok(Companion::Detective),
            "sonnet" | "kim" => Ok(Companion::Kim),
            "haiku" | "cuno" => Ok(Companion::Cuno),
            other => Err(format!("Unknown companion: '{}'", other)),
        }
    }
}

pub struct CompanionInfo {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub model: &'static str,
}

impl Companion {
    pub fn info(self) -> CompanionInfo {
        match self {
            Companion::Detective => CompanionInfo {
                name: "The Detective",
                title: "Senior Detective, RCM",
                description: "You. The one who makes the calls.",
                model: "opus",
            },
            Companion::Kim => CompanionInfo {
                name: "Kim Kitsuragi",
                title: "Lieutenant, Precinct 57",
                description: "Your partner. Steady hands, clean code.",
                model: "sonnet",
            },
            Companion::Cuno => CompanionInfo {
                name: "Cuno",
                title: "Street Kid",
                description:
                    "Fast. Loud. Finds things before you knew you were looking. Cuno doesn't care.",
                model: "haiku",
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionAction {
    Arrives,
    Completes,
    Fails,
    Observes,
}

fn templates(companion: Companion, action: CompanionAction) -> &'static [&'static str] {
    match (companion, action) {
        (Companion::Kim, CompanionAction::Arrives) => &[
            "Kim adjusts his glasses. 'I'll handle this.'",
            "Kim opens his notebook. Time to work.",
            "Your partner steps forward. Quiet confidence.",
        ],
        (Companion::Kim, CompanionAction::Completes) => &[
            "Kim closes his notebook. 'Done.' No fanfare needed.",
            "'It's finished,' Kim says, reviewing his work once more.",
            "Kim nods. Clean work. As always.",
        ],
        (Companion::Kim, CompanionAction::Fails) => &[
            "Kim frowns. 'This... didn't go as planned.'",
            "A rare sigh from Kim. He'll try a different approach.",
            "'Unexpected,' Kim mutters, already analyzing what went wrong.",
        ],
        (Companion::Kim, CompanionAction::Observes) => &[
            "Kim watches silently, taking notes.",
            "You catch Kim's eye. He gives a slight nod.",
            "Kim is already two steps ahead in his mind.",
        ],
        (Companion::Cuno, CompanionAction::Arrives) => &[
            "Cuno's HERE! Cuno finds things FAST!",
            "Cuno doesn't wait. Cuno GOES.",
            "You need something found? CUNO'S ON IT.",
        ],
        (Companion::Cuno, CompanionAction::Completes) => &[
            "BOOM! Cuno found it! Cuno ALWAYS finds it!",
            "Cuno's done. Was there ever any doubt? NO!",
            "That was NOTHING for Cuno!",
        ],
        (Companion::Cuno, CompanionAction::Fails) => &[
            "Cuno doesn't FAIL! Cuno just... didn't find it YET!",
            "Whatever! Cuno doesn't CARE about that one!",
            "NOT Cuno's fault! The codebase is STUPID!",
        ],
        (Companion::Cuno, CompanionAction::Observes) => &[
            "Cuno is watching. Cuno sees EVERYTHING.",
            "Cuno already knows what you're gonna do.",
            "Cuno's bored. When does Cuno get to DO something?",
        ],
        (Companion::Detective, CompanionAction::Arrives) => &[
            "You take a breath. This one needs careful thought.",
            "The case unfolds before you. Time to think.",
            "This is what you're here for. The big picture.",
        ],
        (Companion::Detective, CompanionAction::Completes) => &[
            "The pieces fit. Another part of the puzzle solved.",
            "You lean back. The plan is sound.",
            "It's all coming together. You can feel it.",
        ],
        (Companion::Detective, CompanionAction::Fails) => &[
            "The threads slip through your fingers. Regroup.",
            "Not every lead pans out. But you're still on the case.",
            "A setback. But detectives don't quit.",
        ],
        (Companion::Detective, CompanionAction::Observes) => &[
            "You watch from a distance. Observing. Thinking.",
            "Something about this doesn't sit right...",
            "The detective's instinct kicks in. Wait for it.",
        ],
    }
}

pub fn companion_for_model(model_hint: &str) -> Companion {
    if model_hint.contains("opus") {
        Companion::Detective
    } else if model_hint.contains("haiku") {
        Companion::Cuno
    } else {
        Companion::Kim
    }
}

pub fn random_line(companion: Companion, action: CompanionAction) -> String {
    let lines = templates(companion, action);
    let idx = rand::rng().random_range(0..lines.len());
    lines[idx].to_string()
}

pub fn format_companion_line(companion: Companion, action: CompanionAction) -> String {
    let name = match companion {
        Companion::Detective => companion.to_string().blue().bold().to_string(),
        Companion::Kim => companion.to_string().yellow().bold().to_string(),
        Companion::Cuno => companion.to_string().red().bold().to_string(),
    };
    let line = random_line(companion, action);
    format!("{}: {}", name, line)
}
