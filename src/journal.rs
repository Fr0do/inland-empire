use chrono::{DateTime, Utc};
use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

// ── Genre ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Genre {
    #[default]
    Noir,
    SciFi,
    Horror,
    Fantasy,
    Cyberpunk,
}

impl Genre {
    #[allow(dead_code)]
    pub fn all() -> &'static [Genre] {
        &[
            Genre::Noir,
            Genre::SciFi,
            Genre::Horror,
            Genre::Fantasy,
            Genre::Cyberpunk,
        ]
    }
}

impl fmt::Display for Genre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Genre::Noir => write!(f, "Detective Noir"),
            Genre::SciFi => write!(f, "Sci-Fi"),
            Genre::Horror => write!(f, "Horror"),
            Genre::Fantasy => write!(f, "Fantasy"),
            Genre::Cyberpunk => write!(f, "Cyberpunk"),
        }
    }
}

impl FromStr for Genre {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "noir" | "detective" | "detective noir" => Ok(Genre::Noir),
            "scifi" | "sci-fi" | "sci_fi" | "science fiction" => Ok(Genre::SciFi),
            "horror" => Ok(Genre::Horror),
            "fantasy" => Ok(Genre::Fantasy),
            "cyberpunk" | "cyber" => Ok(Genre::Cyberpunk),
            other => Err(format!(
                "Unknown genre: '{}'. Try: noir, scifi, horror, fantasy, cyberpunk",
                other
            )),
        }
    }
}

// ── EntryType ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    CriticalSuccess,
    CriticalFailure,
    CheckPass,
    CheckFail,
    GameOver,
    LevelUp,
    ThoughtInternalized,
    SubstanceUsed,
    Rest,
    Manual,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::CriticalSuccess => write!(f, "CRITICAL SUCCESS"),
            EntryType::CriticalFailure => write!(f, "CRITICAL FAILURE"),
            EntryType::CheckPass => write!(f, "CHECK"),
            EntryType::CheckFail => write!(f, "CHECK"),
            EntryType::GameOver => write!(f, "GAME OVER"),
            EntryType::LevelUp => write!(f, "LEVEL UP"),
            EntryType::ThoughtInternalized => write!(f, "THOUGHT"),
            EntryType::SubstanceUsed => write!(f, "SUBSTANCE"),
            EntryType::Rest => write!(f, "REST"),
            EntryType::Manual => write!(f, "NOTE"),
        }
    }
}

// ── JournalEntry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: DateTime<Utc>,
    pub entry_type: EntryType,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<CheckDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckDetails {
    pub skill: String,
    pub roll: (u8, u8),
    pub modifier: i8,
    pub total: i8,
    pub threshold: u8,
    pub check_color: String,
    pub tool: String,
    pub context: String,
    pub success: bool,
}

// ── Template system ───────────────────────────────────────────────────────────

fn templates(genre: Genre, entry_type: EntryType) -> &'static [&'static str] {
    match (genre, entry_type) {
        // CheckPass/CheckFail don't use genre templates — content is set directly
        (_, EntryType::CheckPass) => &["{content}"],
        (_, EntryType::CheckFail) => &["{content}"],
        // ── NOIR ──────────────────────────────────────────────────────────────
        (Genre::Noir, EntryType::CriticalSuccess) => &[
            "The dice finally fell my way. {skill} came through on {context}. Case breaks wide open.",
            "Sometimes the city gives back. {skill} delivered on {context}. A rare win.",
            "{skill} cracked it. {context} — solved. Pour one out for the ones that got away.",
        ],
        (Genre::Noir, EntryType::CriticalFailure) => &[
            "The dice betrayed me. {skill} crumbled on {context}. Another dead end.",
            "{skill} went dark on {context}. The case file gets thicker, the leads get thinner.",
            "Failed. {skill} had nothing for {context}. Rain keeps falling.",
        ],
        (Genre::Noir, EntryType::GameOver) => &[
            "The case goes cold. The detective slumps over the keyboard.",
            "No more leads. No more strength. The city wins this round.",
            "Lights out. The case file closes. Someone else picks it up.",
        ],
        (Genre::Noir, EntryType::LevelUp) => &[
            "Level {level}. More scars, more skills. The city teaches those who survive.",
            "Another notch on the belt. Level {level}. Still standing.",
            "Level {level}. Harder, sharper. The streets can tell.",
        ],
        (Genre::Noir, EntryType::ThoughtInternalized) => &[
            "A new thought takes root: {thought}. It'll change how I work the case.",
            "{thought} — can't shake it. It's part of the method now.",
            "The thought crystallizes: {thought}. Everything looks different.",
        ],
        (Genre::Noir, EntryType::SubstanceUsed) => &[
            "Reached for the {substance}. A detective's best friend.",
            "The {substance} hits. The world narrows to what matters.",
            "{substance}. Not proud of it. But it works.",
        ],
        (Genre::Noir, EntryType::Rest) => &[
            "Sleep. Dreamless, merciful sleep. Tomorrow the case continues.",
            "Crashed at the desk. Dawn comes too soon.",
            "Rest. The body repairs what the mind broke.",
        ],
        (Genre::Noir, EntryType::Manual) => &[
            "{content}",
            "{content}",
            "{content}",
        ],

        // ── SCI-FI ────────────────────────────────────────────────────────────
        (Genre::SciFi, EntryType::CriticalSuccess) => &[
            "SHIP LOG: {skill} subsystem exceeded parameters on {context}. Anomalous performance.",
            "TELEMETRY: {skill} output nominal-plus during {context}. Flagged for analysis.",
            "SENSOR ARRAY: {skill} resonance peak detected. {context} resolved. Optimal.",
        ],
        (Genre::SciFi, EntryType::CriticalFailure) => &[
            "WARNING: {skill} subsystem failure during {context}. Hull integrity compromised.",
            "ALERT: {skill} cascade failure on {context}. Switching to backup.",
            "CRITICAL: {skill} offline. {context} — unresolved. Damage report pending.",
        ],
        (Genre::SciFi, EntryType::GameOver) => &[
            "EMERGENCY: All systems offline. Crew vitals flatlined. Mission aborted.",
            "FINAL LOG: Life support failure. The void claims another ship.",
            "DISTRESS BEACON: Activated. No response expected. Signing off.",
        ],
        (Genre::SciFi, EntryType::LevelUp) => &[
            "UPGRADE: Operative level {level}. Neural pathways recalibrated.",
            "SYSTEM UPDATE: Clearance elevated to tier {level}. New protocols unlocked.",
            "EVOLUTION: Cognitive firmware v{level}.0 installed. Reboot complete.",
        ],
        (Genre::SciFi, EntryType::ThoughtInternalized) => &[
            "COGNITIVE PATCH: {thought} integrated into neural matrix.",
            "SUBROUTINE: {thought} — compiled and loaded into active memory.",
            "NEURAL LINK: New pattern recognized: {thought}. Synaptic weight adjusted.",
        ],
        (Genre::SciFi, EntryType::SubstanceUsed) => &[
            "MEDICAL LOG: {substance} compound administered. Monitoring biometrics.",
            "PHARMA: {substance} injected. Biomarkers stabilizing.",
            "DOSAGE: {substance} — within tolerance. Side effects: acceptable.",
        ],
        (Genre::SciFi, EntryType::Rest) => &[
            "MAINTENANCE CYCLE: Systems entering standby. Recovery: 8 hours.",
            "STASIS: Crew pod sealed. Defrag cycle initiated.",
            "POWER DOWN: Non-essential systems offline. Regeneration in progress.",
        ],
        (Genre::SciFi, EntryType::Manual) => &[
            "{content}",
            "{content}",
            "{content}",
        ],

        // ── HORROR ────────────────────────────────────────────────────────────
        (Genre::Horror, EntryType::CriticalSuccess) => &[
            "{skill} guided you through the dark. {context} — survived. Something watched.",
            "You made it past {context}. {skill} held. But the shadows grew longer.",
            "{skill} flared like a match in the dark. {context} — for now, you're safe.",
        ],
        (Genre::Horror, EntryType::CriticalFailure) => &[
            "{skill} failed in the dark. {context} — something stirred in the shadows.",
            "The {skill} guttered out during {context}. You heard breathing. Not yours.",
            "{context}. {skill} abandoned you. The walls are closer now.",
        ],
        (Genre::Horror, EntryType::GameOver) => &[
            "The screen goes black. The cursor blinks. Nobody is typing anymore.",
            "Silence. The kind that comes after the screaming stops.",
            "It's over. The thing in the code found you. It was always going to.",
        ],
        (Genre::Horror, EntryType::LevelUp) => &[
            "Level {level}. You know more now. You wish you didn't.",
            "Level {level}. Each lesson costs something you can't name.",
            "Stronger? Level {level} says yes. But strength attracts attention.",
        ],
        (Genre::Horror, EntryType::ThoughtInternalized) => &[
            "{thought} — the thought won't leave. It scratches at your skull.",
            "You think: {thought}. The thought thinks back.",
            "{thought} lodges in your brain like a splinter. It festers.",
        ],
        (Genre::Horror, EntryType::SubstanceUsed) => &[
            "You swallow the {substance}. It tastes wrong. Everything tastes wrong.",
            "The {substance} burns going down. Something inside you settles. Something else wakes up.",
            "{substance}. Your hands stop shaking. Your shadow doesn't.",
        ],
        (Genre::Horror, EntryType::Rest) => &[
            "You close your eyes. The dreams are worse than waking.",
            "Sleep comes. It brings visitors.",
            "Rest. The darkness behind your eyelids has texture.",
        ],
        (Genre::Horror, EntryType::Manual) => &[
            "{content}",
            "{content}",
            "{content}",
        ],

        // ── FANTASY ───────────────────────────────────────────────────────────
        (Genre::Fantasy, EntryType::CriticalSuccess) => &[
            "The {skill} spell ignites! {context} — a triumph worthy of ancient coders.",
            "By the old algorithms! {skill} strikes true on {context}!",
            "{skill} blazes like dragonfire. {context} falls before your power.",
        ],
        (Genre::Fantasy, EntryType::CriticalFailure) => &[
            "The {skill} enchantment shatters on {context}. Dark magic lingers.",
            "Curse the fates! {skill} fails on {context}. The dungeon grows darker.",
            "{skill} fizzles on {context}. The code-dragons circle closer.",
        ],
        (Genre::Fantasy, EntryType::GameOver) => &[
            "The hero falls. The quest log closes. The kingdom mourns.",
            "Your mana is spent. Your HP is zero. The bards sing of what could have been.",
            "Defeated. The dark compiler claims another soul.",
        ],
        (Genre::Fantasy, EntryType::LevelUp) => &[
            "LEVEL {level} ATTAINED! New powers awaken in the developer-knight.",
            "The XP orbs shimmer. Level {level}! The guild celebrates.",
            "Level {level}. New spells available. The quest grows harder, but so do you.",
        ],
        (Genre::Fantasy, EntryType::ThoughtInternalized) => &[
            "The scroll of {thought} is bound to your grimoire.",
            "{thought} — ancient wisdom, now part of your spellbook.",
            "You meditate on {thought}. The runes rearrange themselves.",
        ],
        (Genre::Fantasy, EntryType::SubstanceUsed) => &[
            "You drink the {substance} potion. Arcane energy fills you.",
            "The {substance} elixir glows faintly. You drink. Power surges.",
            "{substance} — brewed by the dev-monks. It restores what was lost.",
        ],
        (Genre::Fantasy, EntryType::Rest) => &[
            "You rest at the inn. The fireplace crackles. Tomorrow, the quest.",
            "Camp made. Stars above. The code-forest sleeps.",
            "The tavern bed is hard but welcome. HP restored.",
        ],
        (Genre::Fantasy, EntryType::Manual) => &[
            "{content}",
            "{content}",
            "{content}",
        ],

        // ── CYBERPUNK ─────────────────────────────────────────────────────────
        (Genre::Cyberpunk, EntryType::CriticalSuccess) => &[
            "{skill} jack-in clean on {context}. Flatlined the ICE. Chrome and glory.",
            "Neural link: {skill} overclocked on {context}. The net bows.",
            "{skill} sliced through {context} like monofilament. Preem work, choom.",
        ],
        (Genre::Cyberpunk, EntryType::CriticalFailure) => &[
            "{skill} bricked on {context}. Black ICE almost got you. Jack out.",
            "Glitch in {skill} during {context}. Feedback loop. Taste of copper.",
            "{skill} flatlined on {context}. The corpo ICE wins this round.",
        ],
        (Genre::Cyberpunk, EntryType::GameOver) => &[
            "Flatlined. Meat and chrome on the floor. The net doesn't forgive.",
            "System crash. No respawn. The sprawl keeps spinning without you.",
            "Brain-death. Your deck sparks once, then goes dark. Game over, choom.",
        ],
        (Genre::Cyberpunk, EntryType::LevelUp) => &[
            "Street cred: level {level}. New chrome, new edges.",
            "Level {level}. The fixers know your name now.",
            "Upgraded to v{level}. More chrome than meat. The sprawl respects that.",
        ],
        (Genre::Cyberpunk, EntryType::ThoughtInternalized) => &[
            "{thought} — new software in your wetware. Reality shifts.",
            "Braindance echo: {thought}. Permanently etched in your neural lace.",
            "{thought} downloaded. Can't uninstall it. Wouldn't want to.",
        ],
        (Genre::Cyberpunk, EntryType::SubstanceUsed) => &[
            "Popped {substance}. The neon gets brighter. The code gets sharper.",
            "{substance} kicks in. Your chrome hums. Time to jack in.",
            "Slotted {substance}. Street pharma — dirty but effective.",
        ],
        (Genre::Cyberpunk, EntryType::Rest) => &[
            "Crashed in a coffin hotel. The city hums. Recharging.",
            "Flatlined voluntary. Eight hours offline. The meat needs it.",
            "Sleep mode. The neon bleeds through the blinds. Tomorrow's another run.",
        ],
        (Genre::Cyberpunk, EntryType::Manual) => &[
            "{content}",
            "{content}",
            "{content}",
        ],
    }
}

fn apply_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

pub fn generate_entry(
    genre: Genre,
    entry_type: EntryType,
    vars: &HashMap<String, String>,
) -> JournalEntry {
    let pool = templates(genre, entry_type);
    let idx = rand::rng().random_range(0..pool.len());
    let content = apply_vars(pool[idx], vars);
    JournalEntry {
        timestamp: Utc::now(),
        entry_type,
        content,
        details: None,
    }
}

pub fn generate_check_entry(
    genre: Genre,
    entry_type: EntryType,
    vars: &HashMap<String, String>,
    details: CheckDetails,
) -> JournalEntry {
    let mut entry = generate_entry(genre, entry_type, vars);
    entry.details = Some(details);
    entry
}

// ── Display ───────────────────────────────────────────────────────────────────

pub fn format_journal(entries: &[JournalEntry], limit: usize, verbose: bool) -> String {
    let mut lines = Vec::new();

    let slice: Vec<&JournalEntry> = entries.iter().rev().take(limit).collect();

    for entry in slice {
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M").to_string();
        let ts_str = ts.dimmed().to_string();

        let tag_str = match entry.entry_type {
            EntryType::CriticalSuccess => {
                format!("[{}]", entry.entry_type).green().bold().to_string()
            }
            EntryType::CriticalFailure => {
                format!("[{}]", entry.entry_type).red().bold().to_string()
            }
            EntryType::CheckPass => format!("[{}]", "PASS").green().to_string(),
            EntryType::CheckFail => format!("[{}]", "FAIL").red().to_string(),
            EntryType::GameOver => format!("[{}]", entry.entry_type).red().bold().to_string(),
            EntryType::LevelUp => format!("[{}]", entry.entry_type).yellow().to_string(),
            EntryType::ThoughtInternalized => format!("[{}]", entry.entry_type).cyan().to_string(),
            EntryType::SubstanceUsed => format!("[{}]", entry.entry_type).magenta().to_string(),
            EntryType::Rest => format!("[{}]", entry.entry_type).blue().to_string(),
            EntryType::Manual => format!("[{}]", entry.entry_type).white().to_string(),
        };

        lines.push(format!("{} {}", ts_str, tag_str));
        lines.push(format!("  {}", entry.content.italic()));
        if verbose {
            if let Some(ref d) = entry.details {
                let roll_str = format!(
                    "{}+{}+{}={} vs DC{}",
                    d.roll.0, d.roll.1, d.modifier, d.total, d.threshold
                );
                let result = if d.success {
                    "pass".green().to_string()
                } else {
                    "fail".red().to_string()
                };
                lines.push(format!(
                    "  {} {} | {} | {} → {}",
                    "▸".dimmed(),
                    d.skill.dimmed(),
                    d.tool.dimmed(),
                    roll_str.dimmed(),
                    result
                ));
            }
        }
        lines.push(String::new());
    }

    // Remove trailing blank line if present
    if lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}
