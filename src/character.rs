use crate::skills::{Attribute, Skill};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub archetype: String,
    pub level: u32,
    pub xp: u32,
    pub skill_points: u32,
    pub skills: HashMap<Skill, u8>,
    pub attributes: HashMap<Attribute, u8>,
    pub thoughts: Vec<Thought>,
    pub check_history: Vec<CheckRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub name: String,
    pub description: String,
    pub skill_modifiers: HashMap<Skill, i8>,
    pub internalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRecord {
    pub skill: Skill,
    pub difficulty: u8,
    pub roll: (u8, u8),
    pub modifier: i8,
    pub total: i8,
    pub success: bool,
    pub context: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Archetype {
    pub name: &'static str,
    pub description: &'static str,
    pub attributes: [(Attribute, u8); 4],
}

pub const ARCHETYPES: &[Archetype] = &[
    Archetype { name: "Thinker", description: "High Intellect. Analyzes everything, misses nothing logical.", attributes: [(Attribute::Intellect, 5), (Attribute::Psyche, 3), (Attribute::Physique, 2), (Attribute::Motorics, 2)] },
    Archetype { name: "Sensitive", description: "High Psyche. Feels the code, understands the user.", attributes: [(Attribute::Intellect, 3), (Attribute::Psyche, 5), (Attribute::Physique, 2), (Attribute::Motorics, 2)] },
    Archetype { name: "Bruiser", description: "High Physique. Brute-forces through problems.", attributes: [(Attribute::Intellect, 2), (Attribute::Psyche, 2), (Attribute::Physique, 5), (Attribute::Motorics, 3)] },
    Archetype { name: "Operator", description: "High Motorics. Precise, fast, elegant.", attributes: [(Attribute::Intellect, 2), (Attribute::Psyche, 2), (Attribute::Physique, 3), (Attribute::Motorics, 5)] },
    Archetype { name: "Generalist", description: "Balanced across all attributes.", attributes: [(Attribute::Intellect, 3), (Attribute::Psyche, 3), (Attribute::Physique, 3), (Attribute::Motorics, 3)] },
];

impl Character {
    pub fn new(name: String, archetype_name: &str) -> Self {
        let arch = ARCHETYPES.iter().find(|a| a.name.eq_ignore_ascii_case(archetype_name)).unwrap_or(&ARCHETYPES[4]);
        let mut attributes = HashMap::new();
        for (attr, val) in &arch.attributes { attributes.insert(*attr, *val); }
        let mut skills = HashMap::new();
        for skill in Skill::all() {
            let base = attributes.get(&skill.attribute()).copied().unwrap_or(1);
            skills.insert(*skill, base);
        }
        let now = Utc::now();
        Character { name, archetype: arch.name.to_string(), level: 1, xp: 0, skill_points: 0, skills, attributes, thoughts: Vec::new(), check_history: Vec::new(), created_at: now, updated_at: now }
    }

    pub fn effective_skill(&self, skill: Skill) -> i8 {
        let base = *self.skills.get(&skill).unwrap_or(&1) as i8;
        let thought_bonus: i8 = self.thoughts.iter().filter(|t| t.internalized).filter_map(|t| t.skill_modifiers.get(&skill)).sum();
        base + thought_bonus
    }

    pub fn xp_to_next_level(&self) -> u32 { self.level * 100 }

    pub fn add_xp(&mut self, amount: u32) -> Option<u32> {
        self.xp += amount;
        let needed = self.xp_to_next_level();
        if self.xp >= needed { self.xp -= needed; self.level += 1; self.skill_points += 1; self.updated_at = Utc::now(); Some(self.level) }
        else { self.updated_at = Utc::now(); None }
    }

    pub fn develop_skill(&mut self, skill: Skill) -> Result<u8, String> {
        if self.skill_points == 0 { return Err("No skill points available".into()); }
        let current = self.skills.get(&skill).copied().unwrap_or(1);
        let attr_cap = self.attributes.get(&skill.attribute()).copied().unwrap_or(1) + 6;
        if current >= attr_cap { return Err(format!("{} is at cap ({}) for your {} level", skill, attr_cap, skill.attribute())); }
        self.skill_points -= 1;
        let new_val = current + 1;
        self.skills.insert(skill, new_val);
        self.updated_at = Utc::now();
        Ok(new_val)
    }

    pub fn internalize_thought(&mut self, thought: Thought) { self.thoughts.push(thought); self.updated_at = Utc::now(); }

    pub fn record_check(&mut self, record: CheckRecord) {
        let xp = if record.success { 10 } else { 5 };
        self.check_history.push(record);
        self.add_xp(xp);
    }

    pub fn save(&self) -> Result<(), String> {
        let path = profile_path(&self.name);
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Self, String> {
        let path = profile_path(name);
        let data = std::fs::read_to_string(&path).map_err(|_| format!("Character '{}' not found at {}", name, path.display()))?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    pub fn load_active() -> Result<Self, String> {
        let active_path = data_dir().join("active");
        let name = std::fs::read_to_string(&active_path).map_err(|_| "No active character. Run: ie new <name>".to_string())?;
        Self::load(name.trim())
    }

    pub fn set_active(&self) -> Result<(), String> {
        let active_path = data_dir().join("active");
        if let Some(parent) = active_path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        std::fs::write(&active_path, &self.name).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn data_dir() -> PathBuf { dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("inland-empire") }
fn profile_path(name: &str) -> PathBuf { data_dir().join("profiles").join(format!("{}.json", name)) }

pub fn list_profiles() -> Vec<String> {
    let dir = data_dir().join("profiles");
    std::fs::read_dir(dir).into_iter().flatten().filter_map(|e| e.ok()).filter_map(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        name.strip_suffix(".json").map(String::from)
    }).collect()
}
