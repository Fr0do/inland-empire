mod achievements;
mod agents;
mod cases;
mod character;
mod checks;
mod companion;
mod companions;
mod copotype;
mod dashboard;
mod display;
mod equipment;
mod journal;
mod multiplayer;
mod narrator;
mod portrait;
mod publish;
mod skills;
mod stats;
mod storybook;
mod substances;
mod terminal;
mod time;
mod types;

use character::{list_profiles, Character, Thought, ThoughtPhase, ARCHETYPES};
use checks::{passive_interjections, perform_check, Difficulty, DifficultyTier};
use clap::{Parser, Subcommand};
use equipment::{catalog as equipment_catalog, EquipSlot};
use journal::Genre;
use skills::Skill;
use std::collections::HashMap;
use substances::Substance;
use types::CheckColor;

#[derive(Parser)]
#[command(
    name = "ie",
    about = "INLAND EMPIRE — Disco Elysium skill checks for Claude Code",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new character profile
    New {
        name: String,
        #[arg(short, long, default_value = "generalist")]
        archetype: String,
        #[arg(short = 's', long)]
        signature: Option<String>,
    },
    /// Show character sheet
    Status {
        /// Show full body ASCII art portrait
        #[arg(long)]
        art: bool,
        /// One-line compact output for status line integration
        #[arg(long)]
        oneline: bool,
    },
    /// List all saved profiles
    Profiles,
    /// Switch active profile
    Switch { name: String },
    /// Perform a skill check
    Check {
        tool: String,
        #[arg(short, long, default_value = "")]
        context: String,
        #[arg(short, long)]
        difficulty: Option<u8>,
        #[arg(short, long)]
        skill: Option<String>,
    },
    /// Develop a skill (spend skill points)
    Develop { skill: String },
    /// Add a thought to the thought cabinet (begins researching)
    Think {
        name: String,
        #[arg(short, long, default_value = "")]
        description: String,
        #[arg(short, long, default_value = "")]
        modifiers: String,
    },
    /// Forget a thought from the cabinet
    Forget { thought: String },
    /// Retry last failed white check for a skill
    Retry { skill: String },
    /// Hook mode: check a tool action, output JSON for Claude hooks
    #[command(name = "hook-check")]
    HookCheck {
        tool: String,
        #[arg(short, long, default_value = "")]
        context: String,
    },
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
    /// Generate a multi-page static site from journal entries
    #[command(name = "publish")]
    Publish {
        /// Base URL for the site (used in Atom feed links)
        #[arg(short, long, default_value = "https://example.com")]
        base_url: String,
        /// Output directory (default: {name}-site)
        #[arg(short, long)]
        output: Option<String>,
        /// Deploy to GitHub Pages (push to gh-pages branch)
        #[arg(long)]
        deploy: bool,
    },
    /// Generate a timeline aggregator to follow other players' journals
    #[command(name = "timeline")]
    Timeline {
        /// Your published site URL (for pre-populating feeds.json)
        #[arg(long, default_value = "")]
        url: String,
        /// Output directory (default: timeline)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show check analytics and case file statistics
    Stats,
    /// Show cases (long-running objectives) with progress
    Cases,
    /// Show achievements and badges
    Achievements,
    /// Export character as portable .ie.json file
    Export {
        /// Character name (default: active character)
        name: Option<String>,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Import character from .ie.json file
    Import {
        /// Path to .ie.json file
        file: String,
    },
    /// Compare two characters side by side
    Compare {
        /// First character name
        name1: String,
        /// Second character name (default: active character)
        name2: Option<String>,
    },
    /// Generate shareable SVG character card
    Card {
        /// Character name (default: active character)
        name: Option<String>,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Launch web dashboard in browser
    Dashboard {
        /// Port to serve on (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },
    /// Show your inner voice companion status
    #[command(name = "inner-voice")]
    InnerVoice,
    /// Set difficulty tier (casual/normal/hardcore)
    Difficulty {
        tier: String,
    },
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
        Commands::New {
            name,
            archetype,
            signature,
        } => cmd_new(&name, &archetype, signature.as_deref()),
        Commands::Status { art, oneline } => cmd_status(art, oneline),
        Commands::Profiles => cmd_profiles(),
        Commands::Switch { name } => cmd_switch(&name),
        Commands::Check {
            tool,
            context,
            difficulty,
            skill,
        } => cmd_check(&tool, &context, difficulty, skill.as_deref()),
        Commands::Retry { skill } => cmd_retry(&skill),
        Commands::Develop { skill } => cmd_develop(&skill),
        Commands::Think {
            name,
            description,
            modifiers,
        } => cmd_think(&name, &description, &modifiers),
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
        Commands::Publish {
            base_url,
            output,
            deploy,
        } => cmd_publish(base_url, output, deploy),
        Commands::Timeline { url, output } => cmd_timeline(&url, output),
        Commands::Stats => cmd_stats(),
        Commands::Cases => cmd_cases(),
        Commands::Achievements => cmd_achievements(),
        Commands::Export { name, output } => cmd_export(name, output),
        Commands::Import { file } => cmd_import(&file),
        Commands::Compare { name1, name2 } => cmd_compare(&name1, name2.as_deref()),
        Commands::Card { name, output } => cmd_card(name, output),
        Commands::Dashboard { port, no_open } => cmd_dashboard(port, no_open),
        Commands::InnerVoice => cmd_inner_voice(),
        Commands::Difficulty { tier } => cmd_difficulty(&tier),
    }
}

fn cmd_new(name: &str, archetype: &str, signature: Option<&str>) {
    let sig = signature.map(|s| {
        s.parse::<Skill>().unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        })
    });
    let ch = Character::new(name.to_string(), archetype, sig);
    ch.save().expect("Failed to save character");
    ch.set_active().expect("Failed to set active");
    println!("{}", display::character_sheet(&ch));
    if let Some(s) = sig {
        println!("★ Signature Skill: {}", s);
    }
    println!("Character '{}' created and set as active.", name);
}

fn cmd_status(art: bool, oneline: bool) {
    match Character::load_active() {
        Ok(ch) => {
            if oneline {
                println!("{}", display::status_oneline(&ch));
                return;
            }
            if art {
                print!("{}", portrait::render_character(&ch));
            }
            println!("{}", display::character_sheet(&ch));
            let ct = copotype::detect_copotype(&ch);
            let info = ct.info();
            use colored::Colorize;
            println!(
                "  Copotype  {} — {}",
                info.name.cyan().bold(),
                info.title.dimmed()
            );
            println!("  Agent     {}", agents::Agent::detect().to_string().cyan());
            println!("  Terminal  {}", terminal::Terminal::detect().name().cyan());
            let total = achievements::Achievement::all().len();
            let earned = ch.achievements.len();
            println!(
                "  Achievements  {}/{}",
                earned.to_string().yellow().bold(),
                total
            );
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
    if profiles.is_empty() {
        println!("No profiles found. Create one with: ie new <name>");
        return;
    }
    let active = Character::load_active().ok().map(|c| c.name);
    for p in profiles {
        let marker = if active.as_deref() == Some(&p) {
            " ←"
        } else {
            ""
        };
        println!("  {}{}", p, marker);
    }
}

fn cmd_switch(name: &str) {
    match Character::load(name) {
        Ok(ch) => {
            ch.set_active().expect("Failed to set active");
            println!("Switched to '{}'.", name);
            println!("{}", display::status_line(&ch));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_check(tool: &str, context: &str, difficulty: Option<u8>, skill_override: Option<&str>) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let skill = if let Some(s) = skill_override {
        s.parse::<Skill>().unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        })
    } else {
        Skill::for_tool(tool, context)
    };
    let threshold = difficulty.unwrap_or_else(|| Difficulty::for_action(tool, context).threshold_for_level(ch.level, ch.difficulty_tier));
    let color = CheckColor::for_action(tool, context);
    let ctx = if context.is_empty() {
        format!("{} action", tool)
    } else {
        context.to_string()
    };
    for ij in passive_interjections(&ch, tool, &ctx) {
        println!("{}", ij.format_de_style());
    }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(&mut ch, skill, threshold, &ctx, color, is_signature);
    println!("{}", result.format_de_style(&ctx));
    if result.game_over {
        use colored::Colorize;
        eprintln!("{}", "\n╔══════════════════════════════════════╗\n║           G A M E   O V E R          ║\n║  Your spirit breaks. The work stops.  ║\n║  (mercy rule: health/morale set to 1) ║\n╚══════════════════════════════════════╝".red().bold());
        ch.health = ch.health.max(1);
        ch.morale = ch.morale.max(1);
    }
    // Inner voice commentary
    if let Some(mut voice) = ch.inner_voice.clone() {
        voice.update_after_check(&ch, result.success, result.critical_success);
        let event = companion::CompanionEvent::PostCheck {
            success: result.success,
            critical: result.critical_success,
            skill,
        };
        if let Some(text) = companion::commentary(&voice, event, &ch) {
            println!("{}", companion::format_companion_line(&voice, &text));
        }
        ch.inner_voice = Some(voice);
    }
    ch.save().expect("Failed to save character");
}

fn cmd_retry(skill_str: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let skill: Skill = skill_str.parse().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let (threshold, context, old_level) = match ch.last_failed_white_check(skill) {
        Some(r) => (r.difficulty, r.context.clone(), r.skill_level_at_check),
        None => {
            eprintln!("No failed white check found for {}", skill);
            return;
        }
    };
    let current_level = ch.effective_skill(skill);
    if current_level <= old_level {
        eprintln!(
            "{} hasn't improved since the last failure ({} → {}). Develop it first.",
            skill, old_level, current_level
        );
        return;
    }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(
        &mut ch,
        skill,
        threshold,
        &context,
        CheckColor::White,
        is_signature,
    );
    println!("{}", result.format_de_style(&context));
    ch.save().expect("Failed to save character");
}

fn cmd_develop(skill_str: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let skill: Skill = skill_str.parse().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    match ch.develop_skill(skill) {
        Ok(new_val) => {
            ch.save().expect("Failed to save");
            println!("{} → {}", skill, new_val);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_think(name: &str, description: &str, modifiers: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let mut skill_modifiers = HashMap::new();
    if !modifiers.is_empty() {
        for part in modifiers.split(',') {
            let parts: Vec<&str> = part.trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                if let Ok(skill) = parts[0].parse::<Skill>() {
                    if let Ok(val) = parts[1].parse::<i8>() {
                        skill_modifiers.insert(skill, val);
                    }
                }
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
            println!(
                "Thought '{}' equipped — researching ({} checks to internalize).",
                name.italic(),
                5
            );
            println!("  {}", "Penalties apply while researching.".dimmed());
            ch.save().expect("Failed to save");
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_forget(name: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
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
        Err(_) => {
            println!(r#"{{"allow": true, "reason": "no active character"}}"#);
            return;
        }
    };
    let skill = Skill::for_tool(tool, context);
    let threshold = Difficulty::for_action(tool, context).threshold_for_level(ch.level, ch.difficulty_tier);

    // Fast path: auto-pass trivial checks for skilled characters (small XP, no journal entry)
    if threshold <= 6 {
        let effective = ch.effective_skill(skill);
        if effective >= 4 {
            // Small XP for exploration (no dice, no journal)
            ch.add_xp(2); // 2 XP per exploration action (vs 10/5 for real checks)
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
    let ctx = if context.is_empty() {
        format!("{} action", tool)
    } else {
        context.to_string()
    };
    for ij in passive_interjections(&ch, tool, &ctx) {
        eprintln!("{}", ij.format_de_style());
    }
    let is_signature = ch.signature_skill == Some(skill);
    let result = perform_check(&mut ch, skill, threshold, &ctx, color, is_signature);
    eprintln!("{}", result.format_de_style(&ctx));
    if result.game_over {
        use colored::Colorize;
        eprintln!("{}", "\n╔══════════════════════════════════════╗\n║           G A M E   O V E R          ║\n║  Your spirit breaks. The work stops.  ║\n║  (mercy rule: health/morale set to 1) ║\n╚══════════════════════════════════════╝".red().bold());
        ch.health = ch.health.max(1);
        ch.morale = ch.morale.max(1);
    }
    // Companion flavor
    let model_hint = std::env::var("CLAUDE_MODEL")
        .or_else(|_| std::env::var("IE_COMPANION"))
        .unwrap_or_default();
    let companion = companions::companion_for_model(&model_hint);
    let companion_action = if result.success {
        companions::CompanionAction::Completes
    } else {
        companions::CompanionAction::Fails
    };
    let companion_line = companions::random_line(companion, companion_action);

    // Journal entry for subagent companions (Kim or Cuno)
    if companion != companions::Companion::Detective {
        let journal_text = format!("[{}] {}", companion, companion_line);
        ch.add_journal_entry(journal_text);
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
            else { format!("{} check FAILED ({} vs {})", result.skill, result.total, result.threshold) },
        "companion": companion_line
    });
    println!("{}", json);

    // Narrator events — structured prompts for the agent to weave into its response
    let narrator_events = narrator::narrator_triggers(&result, &ch, tool, &ctx);
    for event in &narrator_events {
        eprintln!(
            "{}",
            serde_json::to_string(&event.to_json()).unwrap_or_default()
        );
    }

    // Inner voice commentary
    if let Some(mut voice) = ch.inner_voice.clone() {
        voice.update_after_check(&ch, result.success, result.critical_success);
        let iv_event = companion::CompanionEvent::PostCheck {
            success: result.success,
            critical: result.critical_success,
            skill,
        };
        if let Some(text) = companion::commentary(&voice, iv_event, &ch) {
            eprintln!("{}", companion::format_companion_line(&voice, &text));
        }
        ch.inner_voice = Some(voice);
    }

    ch.save().ok();
}

fn cmd_use(substance_str: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let substance: Substance = substance_str.parse().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    match ch.use_substance(substance) {
        Ok(desc) => {
            use colored::Colorize;
            println!(
                "\n{} {}",
                substance.to_string().magenta().bold(),
                desc.italic()
            );
            let info = substance.info();
            if info.health_restore > 0 {
                println!("  {} +{}", "Health".red(), info.health_restore);
            }
            if info.morale_restore > 0 {
                println!("  {} +{}", "Morale".magenta(), info.morale_restore);
            }
            for (skill, val) in info.skill_modifiers {
                if *val > 0 {
                    println!("  {} +{} ({} checks)", skill, val, info.duration);
                } else {
                    println!("  {} {} ({} checks)", skill, val, info.duration);
                }
            }
            println!();
        }
        Err(e) => eprintln!("{}", e),
    }
    ch.save().expect("Failed to save");
}

fn cmd_inventory() {
    let ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    use colored::Colorize;
    println!("\n{}", "INVENTORY".bold());
    let mut any = false;
    for substance in Substance::all() {
        if let Some(&count) = ch.inventory.get(substance) {
            if count > 0 {
                println!(
                    "  {} x{} — {}",
                    substance,
                    count,
                    substance.info().description.dimmed()
                );
                any = true;
            }
        }
    }
    if !any {
        println!("  {}", "(empty)".dimmed());
    }
    if !ch.active_effects.is_empty() {
        println!("\n{}", "ACTIVE EFFECTS".bold());
        for effect in &ch.active_effects {
            let mods: Vec<String> = effect
                .skill_modifiers
                .iter()
                .map(|(s, v)| {
                    if *v >= 0 {
                        format!("{} +{}", s, v).green().to_string()
                    } else {
                        format!("{} {}", s, v).red().to_string()
                    }
                })
                .collect();
            println!(
                "  {} ({} checks left)  {}",
                effect.substance.to_string().magenta(),
                effect.checks_remaining,
                mods.join(", ")
            );
        }
    }
    println!();
}

fn cmd_rest() {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    ch.rest();
    ch.save().expect("Failed to save");
    use colored::Colorize;
    println!("\n{}", "REST".bold());
    println!(
        "  Health: {}/{}  Morale: {}/{}",
        ch.health, ch.max_health, ch.morale, ch.max_morale
    );
    println!("  {}", "All effects cleared. Ready for a new day.".dimmed());
    println!();
}

fn cmd_journal(action: Option<JournalAction>, verbose: bool) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    match action {
        None => {
            use colored::Colorize;
            if ch.journal.is_empty() {
                println!(
                    "\n{}\n  {}\n",
                    "JOURNAL".bold(),
                    "(no entries yet)".dimmed()
                );
                return;
            }
            println!(
                "\n{} ({}){}\n",
                "JOURNAL".bold(),
                ch.genre.to_string().dimmed(),
                if verbose { " [verbose]" } else { "" }
            );
            println!("{}", journal::format_journal(&ch.journal, 10, verbose));
            println!();
        }
        Some(JournalAction::Write { text }) => {
            ch.add_journal_entry(text);
            ch.save().expect("Failed to save");
            println!("Entry added.");
        }
        Some(JournalAction::Genre { genre }) => {
            let g: Genre = genre.parse().unwrap_or_else(|e| {
                eprintln!("{}", e);
                std::process::exit(1);
            });
            ch.genre = g;
            ch.save().expect("Failed to save");
            println!("Journal genre set to {}.", g);
        }
        Some(JournalAction::Full) => {
            use colored::Colorize;
            if ch.journal.is_empty() {
                println!(
                    "\n{}\n  {}\n",
                    "JOURNAL".bold(),
                    "(no entries yet)".dimmed()
                );
                return;
            }
            println!(
                "\n{} ({}){}\n",
                "JOURNAL".bold(),
                ch.genre.to_string().dimmed(),
                if verbose { " [verbose]" } else { "" }
            );
            println!(
                "{}",
                journal::format_journal(&ch.journal, ch.journal.len(), verbose)
            );
            println!();
        }
    }
}

fn cmd_archetypes() {
    println!("\nAvailable Archetypes:\n");
    for arch in ARCHETYPES {
        println!("  {} — {}", arch.name, arch.description);
        for (attr, val) in &arch.attributes {
            println!("    {} {}", attr, "█".repeat(*val as usize));
        }
        println!();
    }
}

fn cmd_skills() {
    println!("\nAll Skills:\n");
    for skill in Skill::all() {
        println!(
            "  {:24} [{}] — {}",
            skill.to_string(),
            skill.attribute(),
            skill.claude_domain()
        );
    }
}

fn cmd_equip(item_name: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let items = equipment_catalog();
    let item = match items.into_iter().find(|i| {
        i.id.eq_ignore_ascii_case(item_name)
            || i.name.to_lowercase().contains(&item_name.to_lowercase())
    }) {
        Some(i) => i,
        None => {
            eprintln!(
                "Unknown item: '{}'. Run 'ie catalog' to see available items.",
                item_name
            );
            return;
        }
    };
    use colored::Colorize;
    let slot = item.slot;
    let name = item.name.clone();
    let prev = ch.loadout.equip(item);
    ch.save().expect("Failed to save");
    println!("Equipped {} in {} slot.", name.bold(), slot);
    if let Some(p) = prev {
        println!("  Replaced: {}", p.name.dimmed());
    }
}

fn cmd_unequip(slot_str: &str) {
    let mut ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let slot: EquipSlot = slot_str.parse().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    match ch.loadout.unequip(slot) {
        Some(item) => {
            ch.save().expect("Failed to save");
            println!("Unequipped {} from {} slot.", item.name, slot);
        }
        None => eprintln!("Nothing equipped in {} slot.", slot),
    }
}

fn cmd_catalog() {
    use colored::Colorize;
    let items = equipment_catalog();
    println!("\n{}\n", "EQUIPMENT CATALOG".bold());
    for slot in &[
        EquipSlot::Hat,
        EquipSlot::Jacket,
        EquipSlot::Shirt,
        EquipSlot::Pants,
        EquipSlot::Shoes,
        EquipSlot::Accessory,
    ] {
        let slot_items: Vec<_> = items.iter().filter(|i| i.slot == *slot).collect();
        if slot_items.is_empty() {
            continue;
        }
        println!("  {}", slot.to_string().bold().underline());
        for item in slot_items {
            let mods: Vec<String> = item
                .skill_modifiers
                .iter()
                .map(|(s, v)| {
                    if *v >= 0 {
                        format!("{} +{}", s, v).green().to_string()
                    } else {
                        format!("{} {}", s, v).red().to_string()
                    }
                })
                .collect();
            println!("    {} — {}", item.name.bold(), item.description.dimmed());
            println!("      id: {}  |  {}", item.id.dimmed(), mods.join(", "));
        }
        println!();
    }
}

fn cmd_storybook(output: Option<String>) {
    let ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    if ch.journal.is_empty() {
        eprintln!("Journal is empty. Nothing to export.");
        return;
    }
    let html =
        storybook::generate_storybook(&ch.name, &ch.archetype, ch.level, ch.genre, &ch.journal);
    let path = output
        .unwrap_or_else(|| format!("{}-journal.html", ch.name.to_lowercase().replace(' ', "-")));
    std::fs::write(&path, &html).expect("Failed to write storybook");
    use colored::Colorize;
    println!("Storybook exported to {}", path.bold());
}

fn cmd_publish(base_url: String, output: Option<String>, deploy: bool) {
    use colored::Colorize;
    let ch = match Character::load_active() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    if ch.journal.is_empty() {
        eprintln!("Journal is empty. Nothing to publish.");
        return;
    }

    let site_dir =
        output.unwrap_or_else(|| format!("{}-site", ch.name.to_lowercase().replace(' ', "-")));
    let files = publish::generate_site(&ch, &base_url);

    let mut count = 0usize;
    for (rel_path, content) in &files {
        let full_path = std::path::Path::new(&site_dir).join(rel_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create output directory");
        }
        std::fs::write(&full_path, content).expect("Failed to write site file");
        count += 1;
    }

    println!("Site generated: {} ({} files)", site_dir.bold(), count);
    println!("  {} {site_dir}/index.html", "→".dimmed());
    println!("  {} {site_dir}/feed.xml", "→".dimmed());

    if deploy {
        deploy_to_gh_pages(&site_dir, &ch.name);
        // Clean up local site dir after successful deploy
        if let Err(e) = std::fs::remove_dir_all(&site_dir) {
            eprintln!("Warning: could not clean up {site_dir}: {e}");
        }
    }
}

fn deploy_to_gh_pages(site_dir: &str, character_name: &str) {
    use colored::Colorize;
    use std::process::Command;

    println!("{}", "Deploying to GitHub Pages...".yellow());

    // 1. Check git is available
    if Command::new("git").arg("--version").output().is_err() {
        eprintln!("Error: git is not available. Install git and try again.");
        return;
    }

    // 2. Get remote URL
    let remote_out = match Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => {
            eprintln!("Error: could not get remote URL. Make sure 'origin' is configured.");
            return;
        }
    };
    let remote_url = String::from_utf8_lossy(&remote_out.stdout)
        .trim()
        .to_string();
    println!("  Remote: {}", remote_url.dimmed());

    // 5. Read git user config from main repo
    let git_name = Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Inland Empire".to_string());
    let git_email = Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "ie@example.com".to_string());

    // 3. Create temp directory with unique name
    let tmp_base = std::env::temp_dir();
    let tmp_dir = tmp_base.join(format!(
        "ie-deploy-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
        eprintln!("Error: could not create temp directory: {e}");
        return;
    }
    println!("  Temp dir: {}", tmp_dir.display().to_string().dimmed());

    // 6. Copy all files from site_dir into temp dir
    let site_path = std::path::Path::new(site_dir);
    if let Err(e) = copy_dir_all(site_path, &tmp_dir) {
        eprintln!("Error: could not copy site files: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }
    println!("  Copied site files");

    // Helper: run a git command in the temp dir
    let run_git = |args: &[&str]| -> Result<(), String> {
        let status = Command::new("git")
            .args(args)
            .current_dir(&tmp_dir)
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git {:?} failed", args))
        }
    };

    // 4. Init repo
    if let Err(e) = run_git(&["init"]) {
        eprintln!("Error: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }

    // Configure user in temp repo
    let _ = Command::new("git")
        .args(["config", "user.name", &git_name])
        .current_dir(&tmp_dir)
        .status();
    let _ = Command::new("git")
        .args(["config", "user.email", &git_email])
        .current_dir(&tmp_dir)
        .status();

    // 7. Stage all files
    if let Err(e) = run_git(&["add", "-A"]) {
        eprintln!("Error: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }

    // 7. Commit
    let msg = format!("Deploy {}'s journal", character_name);
    if let Err(e) = run_git(&["commit", "-m", &msg]) {
        eprintln!("Error: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }
    println!("  Committed: {}", msg.dimmed());

    // 8. Add remote origin
    if let Err(e) = run_git(&["remote", "add", "origin", &remote_url]) {
        eprintln!("Error: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }

    // 9. Force push to gh-pages
    println!("  Pushing to gh-pages...");
    if let Err(e) = run_git(&["push", "--force", "origin", "HEAD:gh-pages"]) {
        eprintln!("Error: {e}");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return;
    }

    // 10. Clean up temp dir
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // 11. Print success with GitHub Pages URL
    let pages_url = remote_url_to_pages_url(&remote_url);
    println!("{}", "Deployed! Your journal is live at:".green());
    println!("{}", format!("  {}", pages_url).green().bold());
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn remote_url_to_pages_url(remote: &str) -> String {
    // Handle SSH: git@github.com:user/repo.git
    // Handle HTTPS: https://github.com/user/repo.git
    let stripped = remote.trim_end_matches(".git");
    if let Some(rest) = stripped.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 {
            return format!("https://{}.github.io/{}/", parts[0], parts[1]);
        }
    }
    if let Some(rest) = stripped.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 {
            return format!("https://{}.github.io/{}/", parts[0], parts[1]);
        }
    }
    format!("{} (gh-pages branch)", remote)
}

fn cmd_timeline(url: &str, output: Option<String>) {
    let files = publish::generate_timeline(url);
    let dir = output.unwrap_or_else(|| "timeline".to_string());
    std::fs::create_dir_all(&dir).expect("Failed to create timeline directory");
    for (path, content) in &files {
        let full = std::path::Path::new(&dir).join(path);
        std::fs::write(&full, content).expect("Failed to write file");
    }
    use colored::Colorize;
    println!("Timeline generated: {} ({} files)", dir.bold(), files.len());
    println!("  \u{2192} {}/index.html", dir);
    println!("  \u{2192} {}/feeds.json", dir);
    println!("\n  Add feeds to follow: edit feeds.json or use the UI");
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
            println!(
                "  Achievements: {}/{}\n",
                earned.to_string().yellow().bold(),
                total
            );
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
        println!(
            "\n  {}\n",
            format_companion_line(c, CompanionAction::Arrives)
        );
        return;
    }

    println!("\n{}\n", "COMPANIONS".bold());
    for c in [Companion::Detective, Companion::Kim, Companion::Cuno] {
        let info = c.info();
        println!("  {}  —  {}", info.name.bold(), info.title.dimmed());
        println!(
            "  model: {}    {}",
            info.model.cyan(),
            info.description.italic()
        );
        println!("  {}", format_companion_line(c, CompanionAction::Observes));
        println!();
    }
}

fn cmd_dashboard(port: u16, no_open: bool) {
    match Character::load_active() {
        Ok(ch) => dashboard::serve(&ch, port, no_open),
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_difficulty(tier: &str) {
    let tier: DifficultyTier = tier.parse().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let mut ch = Character::load_active().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    ch.difficulty_tier = tier;
    ch.save().unwrap();
    println!("Difficulty set to {}", tier);
}

fn cmd_inner_voice() {
    match Character::load_active() {
        Ok(ch) => {
            let voice = ch
                .inner_voice
                .clone()
                .unwrap_or_else(|| companion::InnerVoice::from_character(&ch));
            println!("{}", companion::companion_status(&voice));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_export(name: Option<String>, output: Option<String>) {
    let ch = match name {
        Some(n) => Character::load(&n),
        None => Character::load_active(),
    };
    match ch {
        Ok(ch) => {
            let json = multiplayer::export_character(&ch).expect("Failed to serialize");
            let path = output.unwrap_or_else(|| format!("{}.ie.json", ch.name));
            std::fs::write(&path, &json).expect("Failed to write file");
            println!("Exported {} to {}", ch.name, path);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_import(file: &str) {
    let json = match std::fs::read_to_string(file) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to read {}: {}", file, e);
            return;
        }
    };
    match multiplayer::import_character(&json) {
        Ok(mut ch) => {
            let existing = list_profiles();
            let original_name = ch.name.clone();
            ch.name = multiplayer::resolve_name_conflict(&ch.name, &existing);
            if ch.name != original_name {
                println!("Name conflict: renamed to {}", ch.name);
            }
            ch.save().expect("Failed to save imported character");
            println!("Imported {} from {}", ch.name, file);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_compare(name1: &str, name2: Option<&str>) {
    let left = match Character::load(name1) {
        Ok(ch) => ch,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let right = match name2 {
        Some(n) => Character::load(n),
        None => Character::load_active(),
    };
    match right {
        Ok(right) => {
            let comp = multiplayer::compare(&left, &right);
            print!("{}", multiplayer::format_comparison(&comp));
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn cmd_card(name: Option<String>, output: Option<String>) {
    let ch = match name {
        Some(n) => Character::load(&n),
        None => Character::load_active(),
    };
    match ch {
        Ok(ch) => {
            let svg = multiplayer::generate_card(&ch);
            let path = output.unwrap_or_else(|| format!("{}-card.svg", ch.name));
            std::fs::write(&path, &svg).expect("Failed to write SVG");
            println!("Generated card: {}", path);
        }
        Err(e) => eprintln!("{}", e),
    }
}
