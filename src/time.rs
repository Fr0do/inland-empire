use crate::skills::Skill;
use chrono::{Local, Timelike};
use std::collections::HashMap;

pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl TimeOfDay {
    pub fn current() -> Self {
        match Local::now().hour() {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=23 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    pub fn modifiers(&self) -> HashMap<Skill, i8> {
        match self {
            TimeOfDay::Morning => HashMap::from([(Skill::Volition, 1), (Skill::Logic, 1)]),
            TimeOfDay::Afternoon => HashMap::from([(Skill::Authority, 1), (Skill::Rhetoric, 1)]),
            TimeOfDay::Evening => HashMap::from([
                (Skill::Empathy, 1),
                (Skill::Suggestion, 1),
                (Skill::Electrochemistry, 1),
            ]),
            TimeOfDay::Night => HashMap::from([
                (Skill::Shivers, 1),
                (Skill::InlandEmpire, 1),
                (Skill::Conceptualization, 1),
                (Skill::Endurance, -1),
            ]),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "\u{2600}",
            TimeOfDay::Afternoon => "\u{2600}",
            TimeOfDay::Evening => "\u{1f306}",
            TimeOfDay::Night => "\u{1f319}",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "Morning",
            TimeOfDay::Afternoon => "Afternoon",
            TimeOfDay::Evening => "Evening",
            TimeOfDay::Night => "Night",
        }
    }
}
