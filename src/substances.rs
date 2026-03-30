use crate::skills::Skill;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

// ── Substance enum ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Substance {
    Coffee,
    Cigarette,
    Alcohol,
    Speed,
    Nostalgia,
    Pyrholidon,
    Lsd,
    Mushrooms,
}

impl Substance {
    pub fn all() -> &'static [Substance] {
        &[
            Substance::Coffee,
            Substance::Cigarette,
            Substance::Alcohol,
            Substance::Speed,
            Substance::Nostalgia,
            Substance::Pyrholidon,
            Substance::Lsd,
            Substance::Mushrooms,
        ]
    }

    /// Substances unlocked by default (no thought required)
    #[allow(dead_code)]
    pub fn basic() -> &'static [Substance] {
        &[Substance::Coffee, Substance::Cigarette, Substance::Alcohol]
    }

    /// Returns the thought name required to unlock this substance, if any
    pub fn required_thought(&self) -> Option<&'static str> {
        match self {
            Substance::Coffee | Substance::Cigarette | Substance::Alcohol => None,
            Substance::Speed | Substance::Nostalgia => None,
            Substance::Pyrholidon | Substance::Lsd | Substance::Mushrooms => {
                Some("Расширение сознания")
            }
        }
    }

    pub fn info(&self) -> &'static SubstanceInfo {
        match self {
            Substance::Coffee => &COFFEE_INFO,
            Substance::Cigarette => &CIGARETTE_INFO,
            Substance::Alcohol => &ALCOHOL_INFO,
            Substance::Speed => &SPEED_INFO,
            Substance::Nostalgia => &NOSTALGIA_INFO,
            Substance::Pyrholidon => &PYRHOLIDON_INFO,
            Substance::Lsd => &LSD_INFO,
            Substance::Mushrooms => &MUSHROOMS_INFO,
        }
    }

    pub fn to_active_effect(self) -> ActiveEffect {
        let info = self.info();
        let skill_modifiers = info
            .skill_modifiers
            .iter()
            .copied()
            .collect::<HashMap<Skill, i8>>();
        ActiveEffect {
            substance: self,
            skill_modifiers,
            checks_remaining: info.duration,
        }
    }
}

impl fmt::Display for Substance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

impl FromStr for Substance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "coffee" => Ok(Substance::Coffee),
            "cigarette" | "cig" | "smoke" => Ok(Substance::Cigarette),
            "alcohol" | "booze" | "beer" | "wine" | "drink" => Ok(Substance::Alcohol),
            "speed" | "amphetamine" | "amp" => Ok(Substance::Speed),
            "nostalgia" | "memory" => Ok(Substance::Nostalgia),
            "pyrholidon" | "pyro" | "pyr" => Ok(Substance::Pyrholidon),
            "lsd" | "acid" | "кислота" => Ok(Substance::Lsd),
            "mushrooms" | "shrooms" | "грибы" => Ok(Substance::Mushrooms),
            other => Err(format!(
                "Unknown substance: '{other}'. Try: coffee, cigarette, alcohol, speed, nostalgia, pyrholidon, lsd, mushrooms"
            )),
        }
    }
}

// ── SubstanceInfo ─────────────────────────────────────────────────────────────

pub struct SubstanceInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub health_restore: i8,
    pub morale_restore: i8,
    pub skill_modifiers: &'static [(Skill, i8)],
    pub duration: u8,
}

// ── Static catalog ────────────────────────────────────────────────────────────

static COFFEE_INFO: SubstanceInfo = SubstanceInfo {
    name: "Coffee",
    description: "Bitter, black, no sugar. The fog in your mind lifts — just a little.",
    health_restore: 1,
    morale_restore: 0,
    skill_modifiers: &[(Skill::Logic, 1), (Skill::Volition, 1)],
    duration: 5,
};

static CIGARETTE_INFO: SubstanceInfo = SubstanceInfo {
    name: "Cigarette",
    description: "Smoke rises. Your hands stop shaking. You look like you know what you're doing.",
    health_restore: 1,
    morale_restore: 0,
    skill_modifiers: &[(Skill::Composure, 1), (Skill::Savoir, 1)],
    duration: 5,
};

static ALCOHOL_INFO: SubstanceInfo = SubstanceInfo {
    name: "Alcohol",
    description:
        "The world softens. Edges blur. Everything seems possible — and nothing seems to matter.",
    health_restore: 0,
    morale_restore: 2,
    skill_modifiers: &[
        (Skill::Suggestion, 1),
        (Skill::Drama, 1),
        (Skill::HandEyeCoordination, -1),
    ],
    duration: 8,
};

static SPEED_INFO: SubstanceInfo = SubstanceInfo {
    name: "Speed",
    description:
        "Everything accelerates. You accelerate. The code scrolls by and you catch every line.",
    health_restore: 0,
    morale_restore: 1,
    skill_modifiers: &[
        (Skill::ReactionSpeed, 2),
        (Skill::Perception, 1),
        (Skill::Composure, -1),
        (Skill::Endurance, -1),
    ],
    duration: 6,
};

static NOSTALGIA_INFO: SubstanceInfo = SubstanceInfo {
    name: "Nostalgia",
    description: "Memories flood in. Warm and bitter. Someone else's code feels like your own.",
    health_restore: 0,
    morale_restore: 2,
    skill_modifiers: &[
        (Skill::InlandEmpire, 1),
        (Skill::Empathy, 1),
        (Skill::Volition, -1),
    ],
    duration: 6,
};

static PYRHOLIDON_INFO: SubstanceInfo = SubstanceInfo {
    name: "Pyrholidon",
    description:
        "The world cracks open. You see through the walls of code, into the machine itself.",
    health_restore: 2,
    morale_restore: 0,
    skill_modifiers: &[
        (Skill::Shivers, 2),
        (Skill::InlandEmpire, 1),
        (Skill::Logic, -1),
        (Skill::Encyclopedia, -1),
    ],
    duration: 8,
};

static LSD_INFO: SubstanceInfo = SubstanceInfo {
    name: "LSD",
    description: "The code dissolves into fractals. Every function is a universe. You understand everything — or nothing.",
    health_restore: 0,
    morale_restore: 3,
    skill_modifiers: &[
        (Skill::Conceptualization, 3),
        (Skill::InlandEmpire, 2),
        (Skill::Shivers, 2),
        (Skill::Logic, -2),
        (Skill::HandEyeCoordination, -2),
        (Skill::Composure, -1),
    ],
    duration: 12,
};

static MUSHROOMS_INFO: SubstanceInfo = SubstanceInfo {
    name: "Mushrooms",
    description: "The boundaries between you and the codebase dissolve. The functions breathe. The types have feelings.",
    health_restore: 1,
    morale_restore: 2,
    skill_modifiers: &[
        (Skill::Empathy, 3),
        (Skill::InlandEmpire, 2),
        (Skill::Conceptualization, 1),
        (Skill::Volition, -2),
        (Skill::Authority, -2),
        (Skill::ReactionSpeed, -1),
    ],
    duration: 10,
};

// ── ActiveEffect ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub substance: Substance,
    pub skill_modifiers: HashMap<Skill, i8>,
    pub checks_remaining: u8,
}

impl ActiveEffect {
    /// Tick one check off the duration. Returns true if the effect has expired.
    pub fn tick(&mut self) -> bool {
        if self.checks_remaining > 0 {
            self.checks_remaining -= 1;
        }
        self.checks_remaining == 0
    }

    pub fn is_expired(&self) -> bool {
        self.checks_remaining == 0
    }
}

impl fmt::Display for ActiveEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} checks remaining)",
            self.substance, self.checks_remaining
        )
    }
}

// ── Default inventory ─────────────────────────────────────────────────────────

pub fn starting_inventory() -> HashMap<Substance, u8> {
    let mut inv = HashMap::new();
    inv.insert(Substance::Coffee, 2);
    inv.insert(Substance::Cigarette, 2);
    inv.insert(Substance::Alcohol, 1);
    inv
}

// ── Loot ─────────────────────────────────────────────────────────────────────

/// Generate a random loot drop based on unlocked substances.
/// `unlocked_thoughts` is a list of thought names the character has internalized.
pub fn loot_drop(unlocked_thoughts: &[String]) -> Substance {
    use rand::Rng;
    let mut pool: Vec<Substance> = Vec::new();
    for s in Substance::all() {
        match s.required_thought() {
            None => pool.push(*s),
            Some(thought) => {
                if unlocked_thoughts.iter().any(|t| t == thought) {
                    pool.push(*s);
                }
            }
        }
    }
    if pool.is_empty() {
        return Substance::Coffee;
    }
    let idx = rand::rng().random_range(0..pool.len());
    pool[idx]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_aliases() {
        assert_eq!("cig".parse::<Substance>().unwrap(), Substance::Cigarette);
        assert_eq!("booze".parse::<Substance>().unwrap(), Substance::Alcohol);
        assert_eq!("beer".parse::<Substance>().unwrap(), Substance::Alcohol);
        assert_eq!("wine".parse::<Substance>().unwrap(), Substance::Alcohol);
        assert_eq!("pyro".parse::<Substance>().unwrap(), Substance::Pyrholidon);
    }

    #[test]
    fn test_from_str_case_insensitive() {
        assert_eq!("COFFEE".parse::<Substance>().unwrap(), Substance::Coffee);
        assert_eq!("Speed".parse::<Substance>().unwrap(), Substance::Speed);
    }

    #[test]
    fn test_to_active_effect() {
        let effect = Substance::Coffee.to_active_effect();
        assert_eq!(effect.substance, Substance::Coffee);
        assert_eq!(effect.checks_remaining, 5);
        assert_eq!(effect.skill_modifiers.get(&Skill::Logic), Some(&1));
        assert_eq!(effect.skill_modifiers.get(&Skill::Volition), Some(&1));
    }

    #[test]
    fn test_tick_expiry() {
        let mut effect = Substance::Coffee.to_active_effect();
        assert!(!effect.is_expired());
        for _ in 0..5 {
            effect.tick();
        }
        assert!(effect.is_expired());
    }

    #[test]
    fn test_starting_inventory() {
        let inv = starting_inventory();
        assert_eq!(inv.get(&Substance::Coffee), Some(&2));
        assert_eq!(inv.get(&Substance::Cigarette), Some(&2));
        assert_eq!(inv.get(&Substance::Alcohol), Some(&1));
        assert_eq!(inv.len(), 3);
    }

    #[test]
    fn test_all_substances_have_info() {
        for s in Substance::all() {
            let info = s.info();
            assert!(!info.name.is_empty());
            assert!(!info.description.is_empty());
        }
    }
}
