use crate::skills::Skill;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    Hat,
    Jacket,
    Shirt,
    Pants,
    Shoes,
    Accessory,
}

impl fmt::Display for EquipSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EquipSlot::Hat => write!(f, "Hat"),
            EquipSlot::Jacket => write!(f, "Jacket"),
            EquipSlot::Shirt => write!(f, "Shirt"),
            EquipSlot::Pants => write!(f, "Pants"),
            EquipSlot::Shoes => write!(f, "Shoes"),
            EquipSlot::Accessory => write!(f, "Accessory"),
        }
    }
}

impl FromStr for EquipSlot {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hat" => Ok(EquipSlot::Hat),
            "jacket" => Ok(EquipSlot::Jacket),
            "shirt" => Ok(EquipSlot::Shirt),
            "pants" => Ok(EquipSlot::Pants),
            "shoes" => Ok(EquipSlot::Shoes),
            "accessory" => Ok(EquipSlot::Accessory),
            _ => Err(format!("Unknown slot: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub slot: EquipSlot,
    pub description: String,
    pub skill_modifiers: HashMap<Skill, i8>,
}

impl Equipment {
    fn new(id: &str, name: &str, slot: EquipSlot, description: &str, mods: &[(Skill, i8)]) -> Self {
        Equipment {
            id: id.to_string(),
            name: name.to_string(),
            slot,
            description: description.to_string(),
            skill_modifiers: mods.iter().copied().collect(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Loadout {
    pub slots: HashMap<EquipSlot, Equipment>,
}

impl Loadout {
    pub fn equip(&mut self, item: Equipment) -> Option<Equipment> {
        self.slots.insert(item.slot, item)
    }

    pub fn unequip(&mut self, slot: EquipSlot) -> Option<Equipment> {
        self.slots.remove(&slot)
    }

    pub fn total_modifiers(&self) -> HashMap<Skill, i8> {
        let mut totals: HashMap<Skill, i8> = HashMap::new();
        for item in self.slots.values() {
            for (skill, val) in &item.skill_modifiers {
                *totals.entry(*skill).or_insert(0) += val;
            }
        }
        totals
    }

    pub fn get(&self, slot: EquipSlot) -> Option<&Equipment> {
        self.slots.get(&slot)
    }
}

pub fn catalog() -> Vec<Equipment> {
    vec![
        // Hats
        Equipment::new(
            "thought-helmet",
            "Thought Helmet",
            EquipSlot::Hat,
            "A tinfoil hat lined with copper wire. It blocks the distracting signals from Slack.",
            &[
                (Skill::Conceptualization, 1),
                (Skill::InlandEmpire, 1),
                (Skill::Interfacing, -1),
            ],
        ),
        Equipment::new(
            "headphones-nc",
            "Noise-Cancelling Headphones",
            EquipSlot::Hat,
            "The world disappears. Just you and the code.",
            &[(Skill::Volition, 2), (Skill::Empathy, -1)],
        ),
        Equipment::new(
            "backwards-cap",
            "Backwards Cap",
            EquipSlot::Hat,
            "You look like you haven't slept in three days. Because you haven't.",
            &[(Skill::Endurance, 1), (Skill::Savoir, -1)],
        ),
        // Jackets
        Equipment::new(
            "hoodie-black",
            "Black Hoodie",
            EquipSlot::Jacket,
            "The uniform of the late-night coder. Stains of unknown origin.",
            &[
                (Skill::Composure, 1),
                (Skill::Shivers, 1),
                (Skill::Authority, -1),
            ],
        ),
        Equipment::new(
            "blazer-tech",
            "Tech Lead Blazer",
            EquipSlot::Jacket,
            "You look like you make decisions. Important ones.",
            &[
                (Skill::Authority, 2),
                (Skill::Rhetoric, 1),
                (Skill::Electrochemistry, -1),
            ],
        ),
        Equipment::new(
            "vest-tactical",
            "Tactical Cargo Vest",
            EquipSlot::Jacket,
            "Seventeen pockets. USB drives in each one.",
            &[
                (Skill::Encyclopedia, 1),
                (Skill::HandEyeCoordination, 1),
                (Skill::Savoir, -1),
            ],
        ),
        // Shirts
        Equipment::new(
            "tshirt-rust",
            "Rust Evangelism T-Shirt",
            EquipSlot::Shirt,
            "'Fearless Concurrency' printed on the front. Coffee stain on the back.",
            &[
                (Skill::Logic, 1),
                (Skill::Rhetoric, 1),
                (Skill::Empathy, -1),
            ],
        ),
        Equipment::new(
            "shirt-hawaiian",
            "Hawaiian Shirt",
            EquipSlot::Shirt,
            "Loud. Confident. You're either a genius or unhinged.",
            &[
                (Skill::Suggestion, 1),
                (Skill::Drama, 1),
                (Skill::Composure, -1),
            ],
        ),
        Equipment::new(
            "turtleneck",
            "Black Turtleneck",
            EquipSlot::Shirt,
            "You're not cosplaying. You're channeling.",
            &[
                (Skill::Conceptualization, 1),
                (Skill::Savoir, 1),
                (Skill::Esprit, -1),
            ],
        ),
        // Pants
        Equipment::new(
            "jeans-debug",
            "Debug Jeans",
            EquipSlot::Pants,
            "Worn at the knees from kneeling before the server rack.",
            &[(Skill::PainThreshold, 1), (Skill::Perception, 1)],
        ),
        Equipment::new(
            "sweatpants",
            "Grey Sweatpants",
            EquipSlot::Pants,
            "Maximum comfort. Minimum respect.",
            &[
                (Skill::Endurance, 1),
                (Skill::PhysicalInstrument, 1),
                (Skill::Authority, -2),
            ],
        ),
        Equipment::new(
            "cargo-pants",
            "Cargo Pants",
            EquipSlot::Pants,
            "More pockets than your IDE has panels.",
            &[
                (Skill::Encyclopedia, 1),
                (Skill::Interfacing, 1),
                (Skill::Savoir, -1),
            ],
        ),
        // Shoes
        Equipment::new(
            "sneakers-silent",
            "Silent Sneakers",
            EquipSlot::Shoes,
            "You move through the office like a ghost.",
            &[(Skill::ReactionSpeed, 1), (Skill::Composure, 1)],
        ),
        Equipment::new(
            "boots-steel",
            "Steel-Toe Boots",
            EquipSlot::Shoes,
            "For when you need to kick the deployment server.",
            &[
                (Skill::PhysicalInstrument, 1),
                (Skill::HalfLight, 1),
                (Skill::Savoir, -1),
            ],
        ),
        Equipment::new(
            "sandals-socks",
            "Socks with Sandals",
            EquipSlot::Shoes,
            "You've transcended caring. True freedom.",
            &[
                (Skill::InlandEmpire, 1),
                (Skill::Volition, 1),
                (Skill::Esprit, -1),
                (Skill::Authority, -1),
            ],
        ),
        // Accessories
        Equipment::new(
            "rubber-duck",
            "Rubber Duck",
            EquipSlot::Accessory,
            "You talk to it. It listens. It judges.",
            &[(Skill::Logic, 2), (Skill::Drama, 1), (Skill::Esprit, -1)],
        ),
        Equipment::new(
            "mechanical-kb",
            "Mechanical Keyboard",
            EquipSlot::Accessory,
            "Cherry MX Blues. Everyone hates you. You don't care.",
            &[
                (Skill::HandEyeCoordination, 1),
                (Skill::Electrochemistry, 1),
                (Skill::Empathy, -1),
            ],
        ),
        Equipment::new(
            "energy-drink",
            "Energy Drink (warm)",
            EquipSlot::Accessory,
            "It's been on your desk since Tuesday. You drink it anyway.",
            &[
                (Skill::ReactionSpeed, 1),
                (Skill::Endurance, 1),
                (Skill::Composure, -1),
            ],
        ),
    ]
}

pub fn starting_loadout() -> Loadout {
    let items = catalog();
    let mut loadout = Loadout::default();
    let starting_ids = [
        "backwards-cap",
        "hoodie-black",
        "jeans-debug",
        "sneakers-silent",
    ];
    for item in items {
        if starting_ids.contains(&item.id.as_str()) {
            loadout.equip(item);
        }
    }
    loadout
}

pub fn format_loadout(loadout: &Loadout) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n{}\n", "EQUIPMENT".bold()));
    out.push_str(&"─".repeat(50));
    out.push('\n');

    let slot_order = [
        EquipSlot::Hat,
        EquipSlot::Jacket,
        EquipSlot::Shirt,
        EquipSlot::Pants,
        EquipSlot::Shoes,
        EquipSlot::Accessory,
    ];

    for slot in &slot_order {
        let slot_label = format!("{:10}", slot.to_string());
        match loadout.get(*slot) {
            None => {
                out.push_str(&format!(
                    "  {}  {}\n",
                    slot_label.dimmed(),
                    "(empty)".dimmed()
                ));
            }
            Some(item) => {
                let mods: Vec<String> = item
                    .skill_modifiers
                    .iter()
                    .map(|(skill, val)| {
                        if *val >= 0 {
                            format!("{} +{}", skill, val).green().to_string()
                        } else {
                            format!("{} {}", skill, val).red().to_string()
                        }
                    })
                    .collect();
                let mods_str = if mods.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", mods.join(", "))
                };
                out.push_str(&format!(
                    "  {}  {}{}\n",
                    slot_label.yellow(),
                    item.name.bold(),
                    mods_str
                ));
                out.push_str(&format!("             {}\n", item.description.dimmed()));
            }
        }
    }

    out
}
