mod character;
mod checks;
mod display;
mod skills;
mod time;
mod types;

use character::{list_profiles, Character, Thought, ARCHETYPES};
use checks::{passive_interjections, perform_check, Difficulty};
use types::CheckColor;
use clap::{Parser, Subcommand};
use skills::Skill;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "ie", about = "INLAND EMPIRE — Disco Elysium skill checks for Claude Code", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new character profile
    New { name: String, #[arg(short, long, default_value = "generalist")] archetype: String, #[arg(short='s', long)] signature: Option<String> },
    /// Show character sheet
    Status,
    /// List all saved profiles
    Profiles,
    /// Switch active profile
    Switch { name: String },
    /// Perform a skill check
    Check { tool: String, #[arg(short, long, default_value = "")] context: String, #[arg(short, long)] difficulty: Option<u8>, #[arg(short, long)] skill: Option<String> },
    /// Develop a skill (spend skill points)
    Develop { skill: String },
    /// Add a thought to the thought cabinet
    Think { name: String, #[arg(short, long, default_value = "")] description: String, #[arg(short, long, default_value = "")] modifiers: String },
    /// Retry last failed white check for a skill
    Retry { skill: String },
    /// Hook mode: check a tool action, output JSON for Claude hooks
    #[command(name = "hook-check")]
    HookCheck { tool: String, #[arg(short, long, default_value = "")] context: String },
    /// List available archetypes
    Archetypes,
    /// Show skill list with descriptions
    Skills,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, archetype, signature } => cmd_new(&name, &archetype, signature.as_deref()),
        Commands::Status => cmd_status(),
        Commands::Profiles => cmd_profiles(),
        Commands::Switch { name } => cmd_switch(&name),
        Commands::Check { tool, context, difficulty, skill } => cmd_check(&tool, &context, difficulty, skill.as_deref()),
        Commands::Retry { skill } => cmd_retry(&skill),
        Commands::Develop { skill } => cmd_develop(&skill),
        Commands::Think { name, description, modifiers } => cmd_think(&name, &description, &modifiers),
        Commands::HookCheck { tool, context } => cmd_hook_check(&tool, &context),
        Commands::Archetypes => cmd_archetypes(),
        Commands::Skills => cmd_skills(),
    }
}

fn cmd_new(name: &str, archetype: &str, signature: Option<&str>) {
    let sig = signature.map(|s| s.parse::<Skill>().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); }));
    let ch = Character::new(name.to_string(), archetype, sig);
    ch.save().expect("Failed to save character");
    ch.set_active().expect("Failed to set active");
    println!("{}", display::character_sheet(&ch));
    if let Some(s) = sig { println!("★ Signature Skill: {}", s); }
    println!("Character '{}' created and set as active.", name);
}

fn cmd_status() {
    match Character::load_active() { Ok(ch) => println!("{}", display::character_sheet(&ch)), Err(e) => eprintln!("{}", e) }
}

fn cmd_profiles() {
    let profiles = list_profiles();
    if profiles.is_empty() { println!("No profiles found. Create one with: ie new <name>"); return; }
    let active = Character::load_active().ok().map(|c| c.name);
    for p in profiles {
        let marker = if active.as_deref() == Some(&p) { " ←" } else { "" };
        println!("  {}{}", p, marker);
    }
}

fn cmd_switch(name: &str) {
    match Character::load(name) {
        Ok(ch) => { ch.set_active().expect("Failed to set active"); println!("Switched to '{}'.", name); println!("{}", display::status_line(&ch)); }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_check(tool: &str, context: &str, difficulty: Option<u8>, skill_override: Option<&str>) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let skill = if let Some(s) = skill_override { s.parse::<Skill>().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); }) } else { Skill::for_tool(tool) };
    let threshold = difficulty.unwrap_or_else(|| Difficulty::for_action(tool, context).threshold());
    let color = CheckColor::for_action(tool, context);
    let ctx = if context.is_empty() { format!("{} action", tool) } else { context.to_string() };
    for ij in passive_interjections(&ch, tool, &ctx) { println!("{}", ij.format_de_style()); }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(&mut ch, skill, threshold, &ctx, color, is_signature);
    println!("{}", result.format_de_style(&ctx));
    if result.game_over {
        use colored::Colorize;
        eprintln!("{}", "\n╔══════════════════════════════════════╗\n║           G A M E   O V E R          ║\n║  Your spirit breaks. The work stops.  ║\n║  (mercy rule: health/morale set to 1) ║\n╚══════════════════════════════════════╝".red().bold());
        ch.health = ch.health.max(1);
        ch.morale = ch.morale.max(1);
    }
    ch.save().expect("Failed to save character");
}

fn cmd_retry(skill_str: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let skill: Skill = skill_str.parse().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    let (threshold, context, old_level) = match ch.last_failed_white_check(skill) {
        Some(r) => (r.difficulty, r.context.clone(), r.skill_level_at_check),
        None => { eprintln!("No failed white check found for {}", skill); return; }
    };
    let current_level = ch.effective_skill(skill);
    if current_level <= old_level {
        eprintln!("{} hasn't improved since the last failure ({} → {}). Develop it first.", skill, old_level, current_level);
        return;
    }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(&mut ch, skill, threshold, &context, CheckColor::White, is_signature);
    println!("{}", result.format_de_style(&context));
    ch.save().expect("Failed to save character");
}

fn cmd_develop(skill_str: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let skill: Skill = skill_str.parse().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    match ch.develop_skill(skill) {
        Ok(new_val) => { ch.save().expect("Failed to save"); println!("{} → {}", skill, new_val); }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_think(name: &str, description: &str, modifiers: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let mut skill_modifiers = HashMap::new();
    if !modifiers.is_empty() {
        for part in modifiers.split(',') {
            let parts: Vec<&str> = part.trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                if let Ok(skill) = parts[0].parse::<Skill>() { if let Ok(val) = parts[1].parse::<i8>() { skill_modifiers.insert(skill, val); } }
            }
        }
    }
    let thought = Thought { name: name.to_string(), description: description.to_string(), skill_modifiers, internalized: true };
    ch.internalize_thought(thought);
    ch.save().expect("Failed to save");
    println!("Thought '{}' internalized.", name);
}

fn cmd_hook_check(tool: &str, context: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(_) => { println!(r#"{{"allow": true, "reason": "no active character"}}"#); return; }
    };
    let skill = Skill::for_tool(tool);
    let threshold = Difficulty::for_action(tool, context).threshold();
    let color = CheckColor::for_action(tool, context);
    let ctx = if context.is_empty() { format!("{} action", tool) } else { context.to_string() };
    for ij in passive_interjections(&ch, tool, &ctx) { eprintln!("{}", ij.format_de_style()); }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(&mut ch, skill, threshold, &ctx, color, is_signature);
    eprintln!("{}", result.format_de_style(&ctx));
    if result.game_over {
        use colored::Colorize;
        eprintln!("{}", "\n╔══════════════════════════════════════╗\n║           G A M E   O V E R          ║\n║  Your spirit breaks. The work stops.  ║\n║  (mercy rule: health/morale set to 1) ║\n╚══════════════════════════════════════╝".red().bold());
        ch.health = ch.health.max(1);
        ch.morale = ch.morale.max(1);
    }
    let json = serde_json::json!({
        "allow": result.success, "skill": result.skill.to_string(),
        "roll": [result.die1, result.die2], "modifier": result.modifier,
        "total": result.total, "threshold": result.threshold,
        "critical_success": result.critical_success, "critical_failure": result.critical_failure,
        "check_color": result.check_color.label(),
        "game_over": result.game_over,
        "retryable": result.check_color == CheckColor::White && !result.success,
        "reason": if result.success { format!("{} check passed ({} vs {})", result.skill, result.total, result.threshold) }
            else { format!("{} check FAILED ({} vs {})", result.skill, result.total, result.threshold) }
    });
    println!("{}", json);
    ch.save().ok();
}

fn cmd_archetypes() {
    println!("\nAvailable Archetypes:\n");
    for arch in ARCHETYPES {
        println!("  {} — {}", arch.name, arch.description);
        for (attr, val) in &arch.attributes { println!("    {} {}", attr, "█".repeat(*val as usize)); }
        println!();
    }
}

fn cmd_skills() {
    println!("\nAll Skills:\n");
    for skill in Skill::all() { println!("  {:24} [{}] — {}", skill.to_string(), skill.attribute(), skill.claude_domain()); }
}
