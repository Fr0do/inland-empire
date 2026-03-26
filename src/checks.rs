use crate::character::{Character, CheckRecord};
use crate::skills::Skill;
use chrono::Utc;
use colored::Colorize;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Trivial, Easy, Medium, Challenging, Formidable, Legendary, Heroic, Godly,
}

impl Difficulty {
    pub fn threshold(&self) -> u8 {
        match self { Difficulty::Trivial => 6, Difficulty::Easy => 8, Difficulty::Medium => 10, Difficulty::Challenging => 12, Difficulty::Formidable => 14, Difficulty::Legendary => 16, Difficulty::Heroic => 18, Difficulty::Godly => 20 }
    }
    pub fn from_threshold(n: u8) -> Self {
        match n { 0..=6 => Difficulty::Trivial, 7..=8 => Difficulty::Easy, 9..=10 => Difficulty::Medium, 11..=12 => Difficulty::Challenging, 13..=14 => Difficulty::Formidable, 15..=16 => Difficulty::Legendary, 17..=18 => Difficulty::Heroic, _ => Difficulty::Godly }
    }
    pub fn label(&self) -> &'static str {
        match self { Difficulty::Trivial => "Trivial", Difficulty::Easy => "Easy", Difficulty::Medium => "Medium", Difficulty::Challenging => "Challenging", Difficulty::Formidable => "Formidable", Difficulty::Legendary => "Legendary", Difficulty::Heroic => "Heroic", Difficulty::Godly => "Godly" }
    }
    pub fn for_action(tool: &str, context: &str) -> Self {
        let ctx = context.to_lowercase();
        match tool {
            "Read" | "read" | "Glob" | "glob" | "Grep" | "grep" => Difficulty::Trivial,
            "Edit" | "edit" => Difficulty::Easy,
            "Write" | "write" => Difficulty::Medium,
            "Bash" | "bash" => {
                if ctx.contains("rm ") || ctx.contains("reset --hard") || ctx.contains("force") || ctx.contains("drop") { Difficulty::Formidable }
                else if ctx.contains("git push") || ctx.contains("deploy") { Difficulty::Challenging }
                else if ctx.contains("git") || ctx.contains("npm") || ctx.contains("cargo") { Difficulty::Easy }
                else { Difficulty::Medium }
            }
            "Agent" | "agent" => Difficulty::Medium,
            _ => Difficulty::Medium,
        }
    }
}

#[derive(Debug)]
pub struct CheckResult {
    pub skill: Skill, pub die1: u8, pub die2: u8, pub modifier: i8, pub total: i8,
    pub threshold: u8, pub success: bool, pub critical_success: bool, pub critical_failure: bool,
}

impl CheckResult {
    pub fn format_de_style(&self, context: &str) -> String {
        let skill_name = self.skill.to_string().to_uppercase();
        let attr = self.skill.attribute();
        let difficulty = Difficulty::from_threshold(self.threshold);
        let mod_str = if self.modifier >= 0 { format!("+{}", self.modifier) } else { format!("{}", self.modifier) };
        let header = format!("{} [{}] — {} check (DC {})", skill_name, attr, difficulty.label(), self.threshold);
        let dice_line = format!("  {} + {} {} = {}  vs  {}", self.die1, self.die2, mod_str, self.total, self.threshold);
        let result_line = if self.critical_success { "  CRITICAL SUCCESS".green().bold().to_string() }
            else if self.critical_failure { "  CRITICAL FAILURE".red().bold().to_string() }
            else if self.success { "  SUCCESS".green().to_string() }
            else { "  FAILURE".red().to_string() };
        let domain = self.skill.claude_domain();
        let flavor = if self.success { format!("  {} whispers: You've got this. {}.", skill_name, domain) }
            else { format!("  {} mutters: Not this time. {} eludes you.", skill_name, domain) };
        format!("\n{}\n{}\n{}\n  {}\n{}\n", header.bold(), dice_line, result_line, context.dimmed(), flavor.italic())
    }
}

pub fn roll_check(character: &Character, skill: Skill, threshold: u8, _context: &str) -> CheckResult {
    let mut rng = rand::thread_rng();
    let die1: u8 = rng.gen_range(1..=6);
    let die2: u8 = rng.gen_range(1..=6);
    let modifier = character.effective_skill(skill);
    let total = die1 as i8 + die2 as i8 + modifier;
    let critical_success = die1 == 6 && die2 == 6;
    let critical_failure = die1 == 1 && die2 == 1;
    let success = if critical_success { true } else if critical_failure { false } else { total >= threshold as i8 };
    CheckResult { skill, die1, die2, modifier, total, threshold, success, critical_success, critical_failure }
}

pub fn perform_check(character: &mut Character, skill: Skill, threshold: u8, context: &str) -> CheckResult {
    let result = roll_check(character, skill, threshold, context);
    let record = CheckRecord { skill, difficulty: threshold, roll: (result.die1, result.die2), modifier: result.modifier, total: result.total, success: result.success, context: context.to_string(), timestamp: Utc::now() };
    character.record_check(record);
    result
}

#[allow(dead_code)]
pub fn passive_check(character: &Character, skill: Skill, threshold: u8) -> bool {
    let effective = character.effective_skill(skill);
    (effective + 6) >= threshold as i8
}
