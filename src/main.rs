mod achievements;
mod cases;
mod character;
mod checks;
mod companions;
mod copotype;
mod display;
mod equipment;
mod journal;
mod portrait;
mod skills;
mod stats;
mod storybook;
mod substances;
mod time;
mod types;

use character::{list_profiles, Character, Thought, ThoughtPhase, ARCHETYPES};
use equipment::{EquipSlot, catalog as equipment_catalog};
use checks::{passive_interjections, perform_check, Difficulty};
use journal::Genre;
use substances::Substance;
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
    Status {
        /// Show full body ASCII art portrait
        #[arg(long)]
        art: bool,
    },
    /// List all saved profiles
    Profiles,
    /// Switch active profile
    Switch { name: String },
    /// Perform a skill check
    Check { tool: String, #[arg(short, long, default_value = "")] context: String, #[arg(short, long)] difficulty: Option<u8>, #[arg(short, long)] skill: Option<String> },
    /// Develop a skill (spend skill points)
    Develop { skill: String },
    /// Add a thought to the thought cabinet (begins researching)
    Think { name: String, #[arg(short, long, default_value = "")] description: String, #[arg(short, long, default_value = "")] modifiers: String },
    /// Forget a thought from the cabinet
    Forget { thought: String },
    /// Retry last failed white check for a skill
    Retry { skill: String },
    /// Hook mode: check a tool action, output JSON for Claude hooks
    #[command(name = "hook-check")]
    HookCheck { tool: String, #[arg(short, long, default_value = "")] context: String },
    /// Use a substance from inventory
    Use { substance: String },
    /// Show inventory and active effects
    Inventory,
    /// Rest: full heal, clear substance effects
    Rest,
    /// Journal: view, write, or change genre
    Journal {
        /// Show check details (rolls, modifiers, DCs)
        #[arg(short, long)]
        verbose: bool,
        #[command(subcommand)]
        action: Option<JournalAction>,
    },
    /// Equip an item
    Equip { item: String },
    /// Unequip a slot
    Unequip { slot: String },
    /// Show available equipment catalog
    Catalog,
    /// List available archetypes
    Archetypes,
    /// Show skill list with descriptions
    Skills,
    /// Show companion roster and model mappings
    Companions {
        /// Show which companion maps to this model hint (e.g. opus, sonnet, haiku)
        #[arg(short, long)]
        model: Option<String>,
    },
    /// Show ASCII art character portrait
    Portrait {
        /// Show compact head-only portrait
        #[arg(long)]
        compact: bool,
    },
    /// Show your coding personality type based on skill distribution
    Copotype,
    /// Export journal as HTML storybook
    #[command(name = "storybook")]
    Storybook {
        /// Output file path (default: {name}-journal.html)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show check analytics and case file statistics
    Stats,
    /// Show cases (long-running objectives) with progress
    Cases,
    /// Show achievements and badges
    Achievements,
}

#[derive(Subcommand)]
enum JournalAction {
    /// Write a manual journal entry
    Write { text: String },
    /// Change journal genre
    Genre { genre: String },
    /// Show all journal entries
    Full,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, archetype, signature } => cmd_new(&name, &archetype, signature.as_deref()),
        Commands::Status { art } => cmd_status(art),
        Commands::Profiles => cmd_profiles(),
        Commands::Switch { name } => cmd_switch(&name),
        Commands::Check { tool, context, difficulty, skill } => cmd_check(&tool, &context, difficulty, skill.as_deref()),
        Commands::Retry { skill } => cmd_retry(&skill),
        Commands::Develop { skill } => cmd_develop(&skill),
        Commands::Think { name, description, modifiers } => cmd_think(&name, &description, &modifiers),
        Commands::Forget { thought } => cmd_forget(&thought),
        Commands::HookCheck { tool, context } => cmd_hook_check(&tool, &context),
        Commands::Use { substance } => cmd_use(&substance),
        Commands::Inventory => cmd_inventory(),
        Commands::Rest => cmd_rest(),
        Commands::Journal { verbose, action } => cmd_journal(action, verbose),
        Commands::Equip { item } => cmd_equip(&item),
        Commands::Unequip { slot } => cmd_unequip(&slot),
        Commands::Catalog => cmd_catalog(),
        Commands::Archetypes => cmd_archetypes(),
        Commands::Skills => cmd_skills(),
        Commands::Companions { model } => cmd_companions(model.as_deref()),
        Commands::Portrait { compact } => cmd_portrait(compact),
        Commands::Copotype => cmd_copotype(),
        Commands::Storybook { output } => cmd_storybook(output),
        Commands::Stats => cmd_stats(),
        Commands::Cases => cmd_cases(),
        Commands::Achievements => cmd_achievements(),
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

fn cmd_status(art: bool) {
    match Character::load_active() {
        Ok(ch) => {
            if art { print!("{}", portrait::render_character(&ch)); }
            println!("{}", display::character_sheet(&ch));
            let ct = copotype::detect_copotype(&ch);
            let info = ct.info();
            use colored::Colorize;
            println!("  Copotype  {} — {}", info.name.cyan().bold(), info.title.dimmed());
            let total = achievements::Achievement::all().len();
            let earned = ch.achievements.len();
            println!("  Achievements  {}/{}", earned.to_string().yellow().bold(), total);
            let cases_done = ch.cases.iter().filter(|c| c.completed).count();
            let cases_total = ch.cases.len();
            println!("  Cases         {}/{} completed", cases_done, cases_total);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_copotype() {
    match Character::load_active() {
        Ok(ch) => {
            let ct = copotype::detect_copotype(&ch);
            print!("{}", copotype::format_copotype(ct, &ch));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_portrait(compact: bool) {
    match Character::load_active() {
        Ok(ch) => {
            if compact {
                print!("{}", portrait::render_portrait(&ch));
            } else {
                print!("{}", portrait::render_character(&ch));
            }
        }
        Err(e) => eprintln!("{}", e),
    }
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
    let thought = Thought {
        name: name.to_string(),
        description: description.to_string(),
        skill_modifiers,
        internalized: false,
        phase: ThoughtPhase::Internalized, // will be overwritten by equip_thought
        research_modifiers: HashMap::new(),
    };
    match ch.equip_thought(thought) {
        Ok(()) => {
            use colored::Colorize;
            println!("Thought '{}' equipped — researching ({} checks to internalize).", name.italic(), 5);
            println!("  {}", "Penalties apply while researching.".dimmed());
            ch.save().expect("Failed to save");
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_forget(name: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    match ch.forget_thought(name) {
        Ok(()) => {
            ch.save().expect("Failed to save");
            println!("Thought '{}' forgotten.", name);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_hook_check(tool: &str, context: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(_) => { println!(r#"{{"allow": true, "reason": "no active character"}}"#); return; }
    };
    let skill = Skill::for_tool(tool);
    let threshold = Difficulty::for_action(tool, context).threshold();

    // Fast path: auto-pass trivial checks for skilled characters (small XP, no journal entry)
    if threshold <= 6 {
        let effective = ch.effective_skill(skill);
        if effective >= 4 {
            // Small XP for exploration (no dice, no journal)
            ch.add_xp(2);  // 2 XP per exploration action (vs 10/5 for real checks)
            let json = serde_json::json!({
                "allow": true,
                "skill": skill.to_string(),
                "roll": [0, 0],
                "modifier": effective,
                "total": effective,
                "threshold": threshold,
                "critical_success": false,
                "critical_failure": false,
                "check_color": "White",
                "game_over": false,
                "retryable": false,
                "reason": format!("{} auto-pass (trivial, +2xp)", skill)
            });
            println!("{}", json);
            ch.save().ok();
            return;
        }
    }

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

fn cmd_use(substance_str: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let substance: Substance = substance_str.parse().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    match ch.use_substance(substance) {
        Ok(desc) => {
            use colored::Colorize;
            println!("\n{} {}", substance.to_string().magenta().bold(), desc.italic());
            let info = substance.info();
            if info.health_restore > 0 { println!("  {} +{}", "Health".red(), info.health_restore); }
            if info.morale_restore > 0 { println!("  {} +{}", "Morale".magenta(), info.morale_restore); }
            for (skill, val) in info.skill_modifiers {
                if *val > 0 { println!("  {} +{} ({} checks)", skill, val, info.duration); }
                else { println!("  {} {} ({} checks)", skill, val, info.duration); }
            }
            println!();
        }
        Err(e) => eprintln!("{}", e),
    }
    ch.save().expect("Failed to save");
}

fn cmd_inventory() {
    let ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    use colored::Colorize;
    println!("\n{}", "INVENTORY".bold());
    let mut any = false;
    for substance in Substance::all() {
        if let Some(&count) = ch.inventory.get(substance) {
            if count > 0 { println!("  {} x{} — {}", substance, count, substance.info().description.dimmed()); any = true; }
        }
    }
    if !any { println!("  {}", "(empty)".dimmed()); }
    if !ch.active_effects.is_empty() {
        println!("\n{}", "ACTIVE EFFECTS".bold());
        for effect in &ch.active_effects {
            let mods: Vec<String> = effect.skill_modifiers.iter().map(|(s, v)| if *v >= 0 { format!("{} +{}", s, v).green().to_string() } else { format!("{} {}", s, v).red().to_string() }).collect();
            println!("  {} ({} checks left)  {}", effect.substance.to_string().magenta(), effect.checks_remaining, mods.join(", "));
        }
    }
    println!();
}

fn cmd_rest() {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    ch.rest();
    ch.save().expect("Failed to save");
    use colored::Colorize;
    println!("\n{}", "REST".bold());
    println!("  Health: {}/{}  Morale: {}/{}", ch.health, ch.max_health, ch.morale, ch.max_morale);
    println!("  {}", "All effects cleared. Ready for a new day.".dimmed());
    println!();
}

fn cmd_journal(action: Option<JournalAction>, verbose: bool) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    match action {
        None => {
            use colored::Colorize;
            if ch.journal.is_empty() { println!("\n{}\n  {}\n", "JOURNAL".bold(), "(no entries yet)".dimmed()); return; }
            println!("\n{} ({}){}\n", "JOURNAL".bold(), ch.genre.to_string().dimmed(), if verbose { " [verbose]" } else { "" });
            println!("{}", journal::format_journal(&ch.journal, 10, verbose));
            println!();
        }
        Some(JournalAction::Write { text }) => {
            ch.add_journal_entry(text);
            ch.save().expect("Failed to save");
            println!("Entry added.");
        }
        Some(JournalAction::Genre { genre }) => {
            let g: Genre = genre.parse().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
            ch.genre = g;
            ch.save().expect("Failed to save");
            println!("Journal genre set to {}.", g);
        }
        Some(JournalAction::Full) => {
            use colored::Colorize;
            if ch.journal.is_empty() { println!("\n{}\n  {}\n", "JOURNAL".bold(), "(no entries yet)".dimmed()); return; }
            println!("\n{} ({}){}\n", "JOURNAL".bold(), ch.genre.to_string().dimmed(), if verbose { " [verbose]" } else { "" });
            println!("{}", journal::format_journal(&ch.journal, ch.journal.len(), verbose));
            println!();
        }
    }
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

fn cmd_equip(item_name: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let items = equipment_catalog();
    let item = match items.into_iter().find(|i| i.id.eq_ignore_ascii_case(item_name) || i.name.to_lowercase().contains(&item_name.to_lowercase())) {
        Some(i) => i,
        None => { eprintln!("Unknown item: '{}'. Run 'ie catalog' to see available items.", item_name); return; }
    };
    use colored::Colorize;
    let slot = item.slot;
    let name = item.name.clone();
    let prev = ch.loadout.equip(item);
    ch.save().expect("Failed to save");
    println!("Equipped {} in {} slot.", name.bold(), slot);
    if let Some(p) = prev { println!("  Replaced: {}", p.name.dimmed()); }
}

fn cmd_unequip(slot_str: &str) {
    let mut ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    let slot: EquipSlot = slot_str.parse().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    match ch.loadout.unequip(slot) {
        Some(item) => { ch.save().expect("Failed to save"); println!("Unequipped {} from {} slot.", item.name, slot); }
        None => eprintln!("Nothing equipped in {} slot.", slot),
    }
}

fn cmd_catalog() {
    use colored::Colorize;
    let items = equipment_catalog();
    println!("\n{}\n", "EQUIPMENT CATALOG".bold());
    for slot in &[EquipSlot::Hat, EquipSlot::Jacket, EquipSlot::Shirt, EquipSlot::Pants, EquipSlot::Shoes, EquipSlot::Accessory] {
        let slot_items: Vec<_> = items.iter().filter(|i| i.slot == *slot).collect();
        if slot_items.is_empty() { continue; }
        println!("  {}", slot.to_string().bold().underline());
        for item in slot_items {
            let mods: Vec<String> = item.skill_modifiers.iter().map(|(s, v)| {
                if *v >= 0 { format!("{} +{}", s, v).green().to_string() }
                else { format!("{} {}", s, v).red().to_string() }
            }).collect();
            println!("    {} — {}", item.name.bold(), item.description.dimmed());
            println!("      id: {}  |  {}", item.id.dimmed(), mods.join(", "));
        }
        println!();
    }
}

fn cmd_storybook(output: Option<String>) {
    let ch = match Character::load_active() { Ok(c) => c, Err(e) => { eprintln!("{}", e); return; } };
    if ch.journal.is_empty() { eprintln!("Journal is empty. Nothing to export."); return; }
    let html = storybook::generate_storybook(&ch.name, &ch.archetype, ch.level, ch.genre, &ch.journal);
    let path = output.unwrap_or_else(|| format!("{}-journal.html", ch.name.to_lowercase().replace(' ', "-")));
    std::fs::write(&path, &html).expect("Failed to write storybook");
    use colored::Colorize;
    println!("Storybook exported to {}", path.bold());
}

fn cmd_stats() {
    match Character::load_active() {
        Ok(ch) => {
            let s = stats::compute_stats(&ch);
            print!("{}", stats::format_stats(&s));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_cases() {
    match Character::load_active() {
        Ok(ch) => print!("{}", cases::format_cases(&ch.cases, &ch)),
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_achievements() {
    match Character::load_active() {
        Ok(ch) => {
            let total = achievements::Achievement::all().len();
            let earned = ch.achievements.len();
            use colored::Colorize;
            println!("  Achievements: {}/{}\n", earned.to_string().yellow().bold(), total);
            print!("{}", achievements::format_achievements(&ch.achievements));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_companions(model: Option<&str>) {
    use colored::Colorize;
    use companions::{companion_for_model, format_companion_line, Companion, CompanionAction};

    if let Some(hint) = model {
        let c = companion_for_model(hint);
        let info = c.info();
        println!("\n  Model hint '{}' → {}\n", hint, c);
        println!("  {}  {}", info.name.bold(), info.title.dimmed());
        println!("  {}", info.description.italic());
        println!("\n  {}\n", format_companion_line(c, CompanionAction::Arrives));
        return;
    }

    println!("\n{}\n", "COMPANIONS".bold());
    for c in [Companion::Detective, Companion::Kim, Companion::Cuno] {
        let info = c.info();
        println!("  {}  —  {}", info.name.bold(), info.title.dimmed());
        println!("  model: {}    {}", info.model.cyan(), info.description.italic());
        println!("  {}", format_companion_line(c, CompanionAction::Observes));
        println!();
    }
}
