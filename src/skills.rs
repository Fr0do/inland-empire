use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attribute {
    Intellect,
    Psyche,
    Physique,
    Motorics,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Intellect => write!(f, "INTELLECT"),
            Attribute::Psyche => write!(f, "PSYCHE"),
            Attribute::Physique => write!(f, "PHYSIQUE"),
            Attribute::Motorics => write!(f, "MOTORICS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    // Intellect
    Logic,
    Encyclopedia,
    Rhetoric,
    Drama,
    Conceptualization,
    VisualCalculus,
    // Psyche
    Volition,
    InlandEmpire,
    Empathy,
    Authority,
    Esprit,
    Suggestion,
    // Physique
    Endurance,
    PainThreshold,
    PhysicalInstrument,
    Electrochemistry,
    Shivers,
    HalfLight,
    // Motorics
    HandEyeCoordination,
    Perception,
    ReactionSpeed,
    Savoir,
    Interfacing,
    Composure,
}

impl Skill {
    pub fn attribute(&self) -> Attribute {
        match self {
            Skill::Logic
            | Skill::Encyclopedia
            | Skill::Rhetoric
            | Skill::Drama
            | Skill::Conceptualization
            | Skill::VisualCalculus => Attribute::Intellect,
            Skill::Volition
            | Skill::InlandEmpire
            | Skill::Empathy
            | Skill::Authority
            | Skill::Esprit
            | Skill::Suggestion => Attribute::Psyche,
            Skill::Endurance
            | Skill::PainThreshold
            | Skill::PhysicalInstrument
            | Skill::Electrochemistry
            | Skill::Shivers
            | Skill::HalfLight => Attribute::Physique,
            Skill::HandEyeCoordination
            | Skill::Perception
            | Skill::ReactionSpeed
            | Skill::Savoir
            | Skill::Interfacing
            | Skill::Composure => Attribute::Motorics,
        }
    }

    pub fn all() -> &'static [Skill] {
        &[
            Skill::Logic,
            Skill::Encyclopedia,
            Skill::Rhetoric,
            Skill::Drama,
            Skill::Conceptualization,
            Skill::VisualCalculus,
            Skill::Volition,
            Skill::InlandEmpire,
            Skill::Empathy,
            Skill::Authority,
            Skill::Esprit,
            Skill::Suggestion,
            Skill::Endurance,
            Skill::PainThreshold,
            Skill::PhysicalInstrument,
            Skill::Electrochemistry,
            Skill::Shivers,
            Skill::HalfLight,
            Skill::HandEyeCoordination,
            Skill::Perception,
            Skill::ReactionSpeed,
            Skill::Savoir,
            Skill::Interfacing,
            Skill::Composure,
        ]
    }

    pub fn claude_domain(&self) -> &'static str {
        match self {
            Skill::Logic => "reasoning through complex code paths",
            Skill::Encyclopedia => "recalling APIs, docs, and obscure knowledge",
            Skill::Rhetoric => "arguing for architectural decisions",
            Skill::Drama => "detecting lies in error messages and logs",
            Skill::Conceptualization => "creative problem-solving and abstractions",
            Skill::VisualCalculus => "spatial reasoning over code structure",
            Skill::Volition => "resisting scope creep and staying on task",
            Skill::InlandEmpire => "gut feelings about code smells",
            Skill::Empathy => "understanding user intent and UX",
            Skill::Authority => "enforcing code standards and conventions",
            Skill::Esprit => "team morale and collaborative coding",
            Skill::Suggestion => "persuading the user to accept refactors",
            Skill::Endurance => "handling massive codebases without fatigue",
            Skill::PainThreshold => "tolerating legacy code and tech debt",
            Skill::PhysicalInstrument => "brute-force solutions and raw throughput",
            Skill::Electrochemistry => "the thrill of deploying to production",
            Skill::Shivers => "sensing the state of the system holistically",
            Skill::HalfLight => "fight-or-flight on critical production bugs",
            Skill::HandEyeCoordination => "precise surgical edits to code",
            Skill::Perception => "noticing subtle bugs and edge cases",
            Skill::ReactionSpeed => "quick hotfixes and incident response",
            Skill::Savoir => "writing elegant, stylish code",
            Skill::Interfacing => "working with external APIs and systems",
            Skill::Composure => "staying calm under failing CI pipelines",
        }
    }

    pub fn for_tool(tool: &str) -> Self {
        match tool {
            "Bash" | "bash" => Skill::Interfacing,
            "Edit" | "edit" => Skill::HandEyeCoordination,
            "Write" | "write" => Skill::Conceptualization,
            "Read" | "read" => Skill::Perception,
            "Glob" | "glob" => Skill::VisualCalculus,
            "Grep" | "grep" => Skill::Encyclopedia,
            "WebFetch" | "web_fetch" => Skill::Interfacing,
            "WebSearch" | "web_search" => Skill::Encyclopedia,
            "Agent" | "agent" => Skill::Authority,
            "git push" | "push" => Skill::Electrochemistry,
            "git reset" | "reset" => Skill::HalfLight,
            "rm" | "delete" => Skill::HalfLight,
            "deploy" => Skill::Electrochemistry,
            _ => Skill::Logic,
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Skill::Logic => "Logic",
            Skill::Encyclopedia => "Encyclopedia",
            Skill::Rhetoric => "Rhetoric",
            Skill::Drama => "Drama",
            Skill::Conceptualization => "Conceptualization",
            Skill::VisualCalculus => "Visual Calculus",
            Skill::Volition => "Volition",
            Skill::InlandEmpire => "Inland Empire",
            Skill::Empathy => "Empathy",
            Skill::Authority => "Authority",
            Skill::Esprit => "Esprit de Corps",
            Skill::Suggestion => "Suggestion",
            Skill::Endurance => "Endurance",
            Skill::PainThreshold => "Pain Threshold",
            Skill::PhysicalInstrument => "Physical Instrument",
            Skill::Electrochemistry => "Electrochemistry",
            Skill::Shivers => "Shivers",
            Skill::HalfLight => "Half Light",
            Skill::HandEyeCoordination => "Hand/Eye Coordination",
            Skill::Perception => "Perception",
            Skill::ReactionSpeed => "Reaction Speed",
            Skill::Savoir => "Savoir Faire",
            Skill::Interfacing => "Interfacing",
            Skill::Composure => "Composure",
        };
        write!(f, "{}", name)
    }
}

impl std::str::FromStr for Skill {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
            "logic" | "log" => Ok(Skill::Logic),
            "encyclopedia" | "enc" => Ok(Skill::Encyclopedia),
            "rhetoric" | "rhe" => Ok(Skill::Rhetoric),
            "drama" | "dra" => Ok(Skill::Drama),
            "conceptualization" | "con" => Ok(Skill::Conceptualization),
            "visualcalculus" | "vis" => Ok(Skill::VisualCalculus),
            "volition" | "vol" => Ok(Skill::Volition),
            "inlandempire" | "inl" => Ok(Skill::InlandEmpire),
            "empathy" | "emp" => Ok(Skill::Empathy),
            "authority" | "aut" => Ok(Skill::Authority),
            "espritdecorps" | "esprit" | "esp" => Ok(Skill::Esprit),
            "suggestion" | "sug" => Ok(Skill::Suggestion),
            "endurance" | "end" => Ok(Skill::Endurance),
            "painthreshold" | "pai" => Ok(Skill::PainThreshold),
            "physicalinstrument" | "phy" => Ok(Skill::PhysicalInstrument),
            "electrochemistry" | "ele" => Ok(Skill::Electrochemistry),
            "shivers" | "shi" => Ok(Skill::Shivers),
            "halflight" | "hal" => Ok(Skill::HalfLight),
            "handeyecoordination" | "han" => Ok(Skill::HandEyeCoordination),
            "perception" | "per" => Ok(Skill::Perception),
            "reactionspeed" | "rea" => Ok(Skill::ReactionSpeed),
            "savoirfaire" | "savoir" | "sav" => Ok(Skill::Savoir),
            "interfacing" | "int" => Ok(Skill::Interfacing),
            "composure" | "com" => Ok(Skill::Composure),
            _ => Err(format!("Unknown skill: {s}")),
        }
    }
}
