use crate::achievements::{self as achievements, Achievement};
use crate::cases::{self as cases, Case};
use crate::equipment::{self as equipment, Loadout};
use crate::journal::{self, EntryType, Genre, JournalEntry};
use crate::skills::{Attribute, Skill};
use crate::substances::{ActiveEffect, Substance, starting_inventory};
use crate::time::TimeOfDay;
use crate::types::CheckColor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn default_health() -> u8 { 4 }
fn default_morale() -> u8 { 6 }

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
    #[serde(default)]
    pub signature_skill: Option<Skill>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_health")]
    pub health: u8,
    #[serde(default = "default_health")]
    pub max_health: u8,
    #[serde(default = "default_morale")]
    pub morale: u8,
    #[serde(default = "default_morale")]
    pub max_morale: u8,
    #[serde(default)]
    pub inventory: HashMap<Substance, u8>,
    #[serde(default)]
    pub active_effects: Vec<ActiveEffect>,
    #[serde(default)]
    pub journal: Vec<JournalEntry>,
    #[serde(default)]
    pub genre: Genre,
    #[serde(default)]
    pub loadout: Loadout,
    #[serde(default)]
    pub cases: Vec<Case>,
    #[serde(default)]
    pub achievements: HashSet<Achievement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThoughtPhase {
    Researching { checks_remaining: u8 },
    Internalized,
}

fn default_phase() -> ThoughtPhase { ThoughtPhase::Internalized }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub name: String,
    pub description: String,
    pub skill_modifiers: HashMap<Skill, i8>,
    /// Kept for serde backwards compatibility with old save files; ignored in logic
    #[serde(default)]
    pub internalized: bool,
    #[serde(default = "default_phase")]
    pub phase: ThoughtPhase,
    /// Penalty modifiers while researching (negative of some bonuses)
    #[serde(default)]
    pub research_modifiers: HashMap<Skill, i8>,
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
    #[serde(default)]
    pub check_color: CheckColor,
    #[serde(default)]
    pub skill_level_at_check: i8,
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
    pub fn new(name: String, archetype_name: &str, signature: Option<Skill>) -> Self {
        let arch = ARCHETYPES.iter().find(|a| a.name.eq_ignore_ascii_case(archetype_name)).unwrap_or(&ARCHETYPES[4]);
        let mut attributes = HashMap::new();
        for (attr, val) in &arch.attributes { attributes.insert(*attr, *val); }
        let mut skills = HashMap::new();
        for skill in Skill::all() {
            let base = attributes.get(&skill.attribute()).copied().unwrap_or(1);
            skills.insert(*skill, base);
        }
        let physique = attributes.get(&Attribute::Physique).copied().unwrap_or(1);
        let psyche = attributes.get(&Attribute::Psyche).copied().unwrap_or(1);
        let max_health = (physique * 2).max(2);
        let max_morale = (psyche * 2).max(2);
        let now = Utc::now();
        Character { name, archetype: arch.name.to_string(), level: 1, xp: 0, skill_points: 0, skills, attributes, thoughts: Vec::new(), check_history: Vec::new(), signature_skill: signature, created_at: now, updated_at: now, health: max_health, max_health, morale: max_morale, max_morale, inventory: starting_inventory(), active_effects: Vec::new(), journal: Vec::new(), genre: Genre::default(), loadout: equipment::starting_loadout(), cases: cases::all_cases(), achievements: HashSet::new() }
    }

    pub fn effective_skill(&self, skill: Skill) -> i8 {
        let base = *self.skills.get(&skill).unwrap_or(&1) as i8;
        let thought_bonus: i8 = self.thoughts.iter()
            .filter_map(|t| match t.phase {
                ThoughtPhase::Researching { .. } => t.research_modifiers.get(&skill),
                ThoughtPhase::Internalized => t.skill_modifiers.get(&skill),
            })
            .sum();
        let signature_bonus: i8 = if self.signature_skill == Some(skill) { 1 } else { 0 };
        let time_bonus = TimeOfDay::current().modifiers().get(&skill).copied().unwrap_or(0) as i8;
        let substance_bonus: i8 = self.active_effects.iter().filter_map(|e| e.skill_modifiers.get(&skill)).sum();
        let equipment_bonus: i8 = self.loadout.total_modifiers().get(&skill).copied().unwrap_or(0);
        base + thought_bonus + signature_bonus + time_bonus + substance_bonus + equipment_bonus
    }

    pub fn xp_to_next_level(&self) -> u32 { self.level * 100 }

    /// Returns (new_level, loot_substance) on level up
    pub fn add_xp(&mut self, amount: u32) -> Option<(u32, Substance)> {
        self.xp += amount;
        let needed = self.xp_to_next_level();
        if self.xp >= needed {
            self.xp -= needed; self.level += 1; self.skill_points += 1; self.updated_at = Utc::now();
            let vars = HashMap::from([("level".into(), self.level.to_string())]);
            self.journal.push(journal::generate_entry(self.genre, EntryType::LevelUp, &vars));
            let thought_names: Vec<String> = self.thoughts.iter().map(|t| t.name.clone()).collect();
            let loot = crate::substances::loot_drop(&thought_names);
            *self.inventory.entry(loot).or_insert(0) += 1;
            Some((self.level, loot))
        }
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

    pub fn last_failed_white_check(&self, skill: Skill) -> Option<&CheckRecord> {
        self.check_history.iter().rev().find(|r| r.skill == skill && !r.success && r.check_color == CheckColor::White)
    }

    pub fn take_physical_damage(&mut self, amount: u8) -> bool {
        self.health = self.health.saturating_sub(amount);
        self.health == 0
    }
    pub fn take_morale_damage(&mut self, amount: u8) -> bool {
        self.morale = self.morale.saturating_sub(amount);
        self.morale == 0
    }
    pub fn heal(&mut self, amount: u8) { self.health = (self.health + amount).min(self.max_health); }
    pub fn restore_morale(&mut self, amount: u8) { self.morale = (self.morale + amount).min(self.max_morale); }
    pub fn is_dead(&self) -> bool { self.health == 0 || self.morale == 0 }

    pub fn equip_thought(&mut self, mut thought: Thought) -> Result<(), String> {
        let researching_count = self.thoughts.iter()
            .filter(|t| matches!(t.phase, ThoughtPhase::Researching { .. }))
            .count();
        if researching_count >= 3 {
            return Err("Too many thoughts being researched (max 3). Forget one first.".into());
        }
        // Generate research penalties: -1 for each positive bonus skill
        let mut research_mods = HashMap::new();
        for (skill, val) in &thought.skill_modifiers {
            if *val > 0 { research_mods.insert(*skill, -1i8); }
        }
        thought.research_modifiers = research_mods;
        thought.phase = ThoughtPhase::Researching { checks_remaining: 5 };
        let vars = HashMap::from([("thought".into(), thought.name.clone())]);
        let entry = journal::generate_entry(self.genre, EntryType::ThoughtInternalized, &vars);
        self.journal.push(entry);
        self.thoughts.push(thought);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn forget_thought(&mut self, name: &str) -> Result<(), String> {
        let pos = self.thoughts.iter().position(|t| t.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("Thought '{}' not found in cabinet.", name))?;
        self.thoughts.remove(pos);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn use_substance(&mut self, substance: Substance) -> Result<String, String> {
        // Check thought requirement
        if let Some(required) = substance.required_thought() {
            let has_thought = self.thoughts.iter().any(|t| t.name == required);
            if !has_thought {
                return Err(format!("You don't know about {} yet. You need to think deeper...", substance));
            }
        }
        let count = self.inventory.get(&substance).copied().unwrap_or(0);
        if count == 0 { return Err(format!("No {} in inventory.", substance)); }
        self.inventory.insert(substance, count - 1);
        if count - 1 == 0 { self.inventory.remove(&substance); }
        let info = substance.info();
        if info.health_restore > 0 { self.heal(info.health_restore as u8); }
        if info.morale_restore > 0 { self.restore_morale(info.morale_restore as u8); }
        self.active_effects.push(substance.to_active_effect());
        let vars = HashMap::from([("substance".into(), substance.to_string())]);
        let entry = journal::generate_entry(self.genre, EntryType::SubstanceUsed, &vars);
        self.journal.push(entry);
        self.updated_at = Utc::now();
        Ok(info.description.to_string())
    }

    pub fn tick_effects(&mut self) {
        for effect in &mut self.active_effects { effect.tick(); }
        self.active_effects.retain(|e| !e.is_expired());
    }

    pub fn rest(&mut self) {
        self.health = self.max_health;
        self.morale = self.max_morale;
        self.active_effects.clear();
        let entry = journal::generate_entry(self.genre, EntryType::Rest, &HashMap::new());
        self.journal.push(entry);
        self.updated_at = Utc::now();
    }

    pub fn add_journal_entry(&mut self, content: String) {
        let vars = HashMap::from([("content".into(), content)]);
        let entry = journal::generate_entry(self.genre, EntryType::Manual, &vars);
        self.journal.push(entry);
        self.updated_at = Utc::now();
    }

    /// Returns (new_level, loot) if leveled up
    pub fn record_check(&mut self, record: CheckRecord) -> Option<(u32, Substance)> {
        let xp = if record.success { 10 } else { 5 };
        let vars = HashMap::from([
            ("skill".into(), record.skill.to_string()),
            ("context".into(), record.context.clone()),
        ]);
        let details = journal::CheckDetails {
            skill: record.skill.to_string(),
            roll: record.roll,
            modifier: record.modifier,
            total: record.total,
            threshold: record.difficulty,
            check_color: record.check_color.label().to_string(),
            tool: record.context.split_whitespace().next().unwrap_or("unknown").to_string(),
            context: record.context.clone(),
            success: record.success,
        };
        if record.roll.0 == 6 && record.roll.1 == 6 {
            self.journal.push(journal::generate_check_entry(self.genre, EntryType::CriticalSuccess, &vars, details));
        } else if record.roll.0 == 1 && record.roll.1 == 1 {
            self.journal.push(journal::generate_check_entry(self.genre, EntryType::CriticalFailure, &vars, details));
        } else {
            let entry = journal::JournalEntry {
                timestamp: chrono::Utc::now(),
                entry_type: if record.success { EntryType::CheckPass } else { EntryType::CheckFail },
                content: if record.success {
                    format!("{} held on {}.", record.skill, record.context)
                } else {
                    format!("{} faltered on {}.", record.skill, record.context)
                },
                details: Some(details),
            };
            self.journal.push(entry);
        }
        self.check_history.push(record);
        self.tick_effects();
        // Progress researching thoughts
        for thought in &mut self.thoughts {
            if let ThoughtPhase::Researching { ref mut checks_remaining } = thought.phase {
                if *checks_remaining > 0 {
                    *checks_remaining -= 1;
                }
                if *checks_remaining == 0 {
                    thought.phase = ThoughtPhase::Internalized;
                }
            }
        }
        // Check case progress
        let case_ids: Vec<usize> = self.cases.iter().enumerate()
            .filter(|(_, c)| !c.completed)
            .map(|(i, _)| i)
            .collect();
        for i in case_ids {
            let (current, target) = cases::check_case_progress(&self.cases[i], self);
            if current >= target {
                self.cases[i].completed = true;
                let reward_xp = self.cases[i].reward_xp;
                let reward_sub = self.cases[i].reward_substance;
                self.add_xp(reward_xp);
                if let Some(sub) = reward_sub {
                    *self.inventory.entry(sub).or_insert(0) += 1;
                }
            }
        }
        // Check for newly earned achievements
        let newly_earned = achievements::check_achievements(self);
        for ach in newly_earned {
            self.achievements.insert(ach);
        }
        self.add_xp(xp)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = profile_path(&self.name);
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        // Lock the file for writing
        let lock_path = path.with_extension("lock");
        let lock_file = std::fs::File::create(&lock_path).map_err(|e| e.to_string())?;
        fs2::FileExt::lock_exclusive(&lock_file).map_err(|e| e.to_string())?;
        // Atomic write: write to temp file, then rename
        let tmp_path = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp_path, &json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        // Unlock
        fs2::FileExt::unlock(&lock_file).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(name: &str) -> Result<Self, String> {
        let path = profile_path(name);
        let lock_path = path.with_extension("lock");
        if let Ok(lock_file) = std::fs::File::create(&lock_path) {
            let _ = fs2::FileExt::lock_shared(&lock_file);
        }
        let data = std::fs::read_to_string(&path).map_err(|_| format!("Character '{}' not found at {}", name, path.display()))?;
        let mut ch: Self = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        // Migration: grant starting inventory to pre-substance characters
        if ch.inventory.is_empty() {
            ch.inventory = starting_inventory();
        }
        // Migration: initialize health/morale for pre-health characters
        if ch.max_health == 0 {
            let physique = ch.attributes.get(&Attribute::Physique).copied().unwrap_or(1);
            ch.max_health = (physique * 2).max(2);
            ch.health = ch.max_health;
        }
        if ch.max_morale == 0 {
            let psyche = ch.attributes.get(&Attribute::Psyche).copied().unwrap_or(1);
            ch.max_morale = (psyche * 2).max(2);
            ch.morale = ch.max_morale;
        }
        // Migration: grant starting loadout to pre-equipment characters
        if ch.loadout.slots.is_empty() {
            ch.loadout = equipment::starting_loadout();
        }
        // Migration: populate cases for pre-cases characters
        if ch.cases.is_empty() {
            ch.cases = cases::all_cases();
        }
        Ok(ch)
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
