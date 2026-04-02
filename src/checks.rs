use serde::{Serialize, Deserialize};
use crate::character::{Character, CheckRecord};
use crate::journal::{self, EntryType};
use crate::skills::{Attribute, Skill};
use crate::substances::Substance;
use crate::types::CheckColor;
use chrono::Utc;
use colored::Colorize;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Trivial,
    Easy,
    Medium,
    Challenging,
    Formidable,
    Legendary,
    Heroic,
    Godly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DifficultyTier {
    Casual,
    Normal,
    Hardcore,
}

impl Default for DifficultyTier {
    fn default() -> Self {
        DifficultyTier::Normal
    }
}

impl std::fmt::Display for DifficultyTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyTier::Casual => write!(f, "Casual"),
            DifficultyTier::Normal => write!(f, "Normal"),
            DifficultyTier::Hardcore => write!(f, "Hardcore"),
        }
    }
}

impl std::str::FromStr for DifficultyTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "casual" | "easy" => Ok(DifficultyTier::Casual),
            "normal" | "medium" => Ok(DifficultyTier::Normal),
            "hardcore" | "hard" => Ok(DifficultyTier::Hardcore),
            _ => Err(format!("Unknown tier: {}. Use casual/normal/hardcore", s)),
        }
    }
}

impl Difficulty {
    pub fn threshold_for_level(&self, level: u32, tier: DifficultyTier) -> u8 {
        let base: u8 = match self {
            Difficulty::Trivial => 6,
            Difficulty::Easy => 8,
            Difficulty::Medium => 10,
            Difficulty::Challenging => 12,
            Difficulty::Formidable => 14,
            Difficulty::Legendary => 16,
            Difficulty::Heroic => 18,
            Difficulty::Godly => 20,
        };
        let divisor = match tier {
            DifficultyTier::Casual => 5,
            DifficultyTier::Normal => 3,
            DifficultyTier::Hardcore => 2,
        };
        let scaling = (level.saturating_sub(1) / divisor) as u8;
        base.saturating_add(scaling).min(20)
    }
    pub fn from_threshold(n: u8) -> Self {
        match n {
            0..=6 => Difficulty::Trivial,
            7..=8 => Difficulty::Easy,
            9..=10 => Difficulty::Medium,
            11..=12 => Difficulty::Challenging,
            13..=14 => Difficulty::Formidable,
            15..=16 => Difficulty::Legendary,
            17..=18 => Difficulty::Heroic,
            _ => Difficulty::Godly,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Difficulty::Trivial => "Trivial",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Challenging => "Challenging",
            Difficulty::Formidable => "Formidable",
            Difficulty::Legendary => "Legendary",
            Difficulty::Heroic => "Heroic",
            Difficulty::Godly => "Godly",
        }
    }
    pub fn for_action(tool: &str, context: &str) -> Self {
        let ctx = context.to_lowercase();
        match tool {
            "Read" | "read" => {
                if ctx.contains(".lock") || ctx.contains("/usr/") || ctx.contains("/proc/") {
                    Difficulty::Easy
                } else {
                    Difficulty::Trivial
                }
            }
            "Glob" | "glob" => Difficulty::Trivial,
            "Grep" | "grep" => {
                if ctx.contains("secret")
                    || ctx.contains("key")
                    || ctx.contains("token")
                    || ctx.contains("password")
                    || ctx.contains("credential")
                {
                    Difficulty::Easy
                } else {
                    Difficulty::Trivial
                }
            }
            "Edit" | "edit" => {
                if ctx.contains("config")
                    || ctx.contains(".toml")
                    || ctx.contains(".yaml")
                    || ctx.contains(".yml")
                    || ctx.contains(".github")
                {
                    Difficulty::Medium
                } else {
                    Difficulty::Easy
                }
            }
            "Write" | "write" => Difficulty::Medium,
            "Bash" | "bash" => {
                if ctx.contains("rm ")
                    || ctx.contains("reset --hard")
                    || ctx.contains("force")
                    || ctx.contains("drop")
                {
                    Difficulty::Formidable
                } else if ctx.contains("git push") || ctx.contains("deploy") {
                    Difficulty::Challenging
                } else if ctx.contains("git") || ctx.contains("npm") || ctx.contains("cargo") {
                    Difficulty::Easy
                } else {
                    Difficulty::Medium
                }
            }
            "Agent" | "agent" => Difficulty::Medium,
            _ => Difficulty::Medium,
        }
    }
}

#[derive(Debug)]
pub struct CheckResult {
    pub skill: Skill,
    pub die1: u8,
    pub die2: u8,
    pub modifier: i8,
    pub total: i8,
    pub threshold: u8,
    pub success: bool,
    pub critical_success: bool,
    pub critical_failure: bool,
    pub check_color: CheckColor,
    pub is_signature: bool,
    pub game_over: bool,
    pub level_up: Option<u32>,
    pub loot: Option<Substance>,
}

impl CheckResult {
    pub fn format_de_style(&self, context: &str) -> String {
        let skill_name = self.skill.to_string().to_uppercase();
        let attr = self.skill.attribute();
        let difficulty = Difficulty::from_threshold(self.threshold);
        let mod_str = if self.modifier >= 0 {
            format!("+{}", self.modifier)
        } else {
            format!("{}", self.modifier)
        };
        let color_tag = match self.check_color {
            CheckColor::White => "○".dimmed().to_string(),
            CheckColor::Red => "●".red().to_string(),
        };
        let colored_skill = match attr {
            Attribute::Intellect => skill_name.blue().bold().to_string(),
            Attribute::Psyche => skill_name.magenta().bold().to_string(),
            Attribute::Physique => skill_name.red().bold().to_string(),
            Attribute::Motorics => skill_name.yellow().bold().to_string(),
        };
        let header = format!(
            "{} {} [{}] — {} check (DC {})",
            color_tag,
            colored_skill,
            attr,
            difficulty.label(),
            self.threshold
        );
        let dice_line = format!(
            "  {} + {} {} = {}  vs  {}",
            self.die1, self.die2, mod_str, self.total, self.threshold
        );
        let result_line = if self.critical_success {
            "  CRITICAL SUCCESS".green().bold().to_string()
        } else if self.critical_failure {
            "  CRITICAL FAILURE".red().bold().to_string()
        } else if self.success {
            "  SUCCESS".green().to_string()
        } else {
            "  FAILURE".red().to_string()
        };
        let domain = self.skill.claude_domain();
        let flavor = if self.is_signature && self.success {
            format!(
                "  {} whispers: This is what you were MADE for. {}.",
                skill_name, domain
            )
        } else if self.is_signature && !self.success {
            format!(
                "  {} mutters: Even your strongest skill falters... {} slips away.",
                skill_name, domain
            )
        } else if self.success {
            format!("  {} whispers: You've got this. {}.", skill_name, domain)
        } else {
            format!(
                "  {} mutters: Not this time. {} eludes you.",
                skill_name, domain
            )
        };
        let retry_hint = if !self.success && self.check_color == CheckColor::White {
            format!(
                "\n  {}",
                "(White check — can retry after developing this skill)".dimmed()
            )
        } else if !self.success {
            format!("\n  {}", "(Red check — no second chances)".red().dimmed())
        } else {
            String::new()
        };
        let level_up_msg = if let (Some(lvl), Some(loot)) = (self.level_up, self.loot) {
            format!(
                "\n\n  {} Level {}! +1 skill point. Found: {}",
                "LEVEL UP!".yellow().bold(),
                lvl,
                loot.to_string().magenta()
            )
        } else {
            String::new()
        };
        let game_over_msg = if self.game_over {
            format!(
                "\n\n{}\n{}\n{}\n",
                "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                    .red()
                    .bold(),
                "  GAME OVER".red().bold(),
                "  The detective collapses. The case goes cold. You were so close."
                    .red()
                    .italic()
            )
        } else {
            String::new()
        };
        format!(
            "\n{}\n{}\n{}\n  {}\n{}{}{}{}\n",
            header.bold(),
            dice_line,
            result_line,
            context.dimmed(),
            flavor.italic(),
            retry_hint,
            level_up_msg,
            game_over_msg
        )
    }
}

pub fn roll_check(
    character: &Character,
    skill: Skill,
    threshold: u8,
    _context: &str,
    check_color: CheckColor,
    is_signature: bool,
) -> CheckResult {
    let mut rng = rand::rng();
    let die1: u8 = rng.random_range(1..=6);
    let die2: u8 = rng.random_range(1..=6);
    let modifier = character.effective_skill(skill);
    let total = die1 as i8 + die2 as i8 + modifier;
    let critical_success = die1 == 6 && die2 == 6;
    let critical_failure = die1 == 1 && die2 == 1;
    let success = if critical_success {
        true
    } else if critical_failure {
        false
    } else {
        total >= threshold as i8
    };
    CheckResult {
        skill,
        die1,
        die2,
        modifier,
        total,
        threshold,
        success,
        critical_success,
        critical_failure,
        check_color,
        is_signature,
        game_over: false,
        level_up: None,
        loot: None,
    }
}

pub fn perform_check(
    character: &mut Character,
    skill: Skill,
    threshold: u8,
    context: &str,
    check_color: CheckColor,
    is_signature: bool,
) -> CheckResult {
    let mut result = roll_check(
        character,
        skill,
        threshold,
        context,
        check_color,
        is_signature,
    );
    let skill_level = character.effective_skill(skill);
    let record = CheckRecord {
        skill,
        difficulty: threshold,
        roll: (result.die1, result.die2),
        modifier: result.modifier,
        total: result.total,
        success: result.success,
        context: context.to_string(),
        timestamp: Utc::now(),
        check_color,
        skill_level_at_check: skill_level,
    };
    if let Some((level, loot)) = character.record_check(record) {
        result.level_up = Some(level);
        result.loot = Some(loot);
    }
    if result.critical_failure {
        let dead = match skill.attribute() {
            Attribute::Physique | Attribute::Motorics => character.take_physical_damage(1),
            Attribute::Intellect | Attribute::Psyche => character.take_morale_damage(1),
        };
        if dead || character.is_dead() {
            result.game_over = true;
            let entry =
                journal::generate_entry(character.genre, EntryType::GameOver, &HashMap::new());
            character.journal.push(entry);
        }
    }
    result
}

#[allow(dead_code)]
pub fn passive_check(character: &Character, skill: Skill, threshold: u8) -> bool {
    let effective = character.effective_skill(skill);
    (effective + 6) >= threshold as i8
}

#[derive(Debug)]
pub struct Interjection {
    pub skill: Skill,
    pub message: String,
}

impl Interjection {
    pub fn format_de_style(&self) -> String {
        let name = self.skill.to_string().to_uppercase();
        format!(
            "{} {} — {}",
            name.bold(),
            "[Passive: Success]".green().dimmed(),
            self.message.italic()
        )
    }
}

fn interjection_messages(skill: Skill) -> &'static [&'static str] {
    match skill {
        Skill::Logic => &[
            "Wait. If A implies B, and B is on fire... check the premises.",
            "There's a contradiction hiding in this logic. I can feel it.",
            "The branching here follows a pattern. You've seen it before.",
            "This reduces to a simpler problem. Think.",
            "One of these conditions is redundant. Guaranteed.",
        ],
        Skill::Encyclopedia => &[
            "Actually, this API was deprecated in version 3.2...",
            "Fun fact: this pattern is called a 'Schlemiel the Painter' algorithm.",
            "According to the RFC, this header is technically optional.",
            "The original author of this library wrote it in a weekend. In 2009.",
            "This is the same bug that caused the 2017 npm incident.",
        ],
        Skill::Rhetoric => &[
            "You could argue this is a feature, not a bug. Convincingly.",
            "The commit message should frame this as an improvement, not a fix.",
            "Make the case. The old approach was fundamentally unsound.",
            "This refactor needs a manifesto, not a PR description.",
            "Position it as tech debt reduction. Nobody argues against that.",
        ],
        Skill::Drama => &[
            "That error message is lying to you. It always lies.",
            "Something about this stack trace feels... performative.",
            "The logs say 'success' but they don't mean it.",
            "This config is hiding something. Read between the lines.",
            "Notice how the test passes? Too easily. Suspiciously easily.",
        ],
        Skill::Conceptualization => &[
            "What if this entire module was actually just... a monad?",
            "There's a beautiful abstraction here, waiting to be born.",
            "Forget the implementation. What does this *mean*?",
            "This could be restructured as a pipeline. Elegant. Clean.",
            "The architecture is trying to become something. Let it.",
        ],
        Skill::VisualCalculus => &[
            "The dependency graph forms a cycle. I can see it.",
            "Trace the data flow. It converges right... there.",
            "The nesting depth here is exactly three too many.",
            "If you squint, the directory structure tells you everything.",
            "The shape of this diff is wrong. Asymmetric.",
        ],
        Skill::Volition => &[
            "Stay focused. This yak doesn't need shaving.",
            "You were supposed to fix ONE bug. Remember?",
            "Resist the urge to refactor. Ship it.",
            "The scope is creeping. Hold the line.",
            "Close those 47 browser tabs. You know what you need to do.",
        ],
        Skill::InlandEmpire => &[
            "The codebase is trying to tell you something...",
            "Something stirs in the dependency tree. Ancient. Waiting.",
            "This function has seen things. Terrible things.",
            "The ghost of a deleted file lingers in the git history.",
            "You can almost hear the server humming. It's worried.",
        ],
        Skill::Empathy => &[
            "The person who wrote this was exhausted. Be gentle.",
            "Think about who has to maintain this after you leave.",
            "The user won't understand that error message. Nobody would.",
            "Whoever opens this PR will need context. Leave comments.",
            "This variable name tells you nothing. The next dev will suffer.",
        ],
        Skill::Authority => &[
            "This code WILL follow the style guide. Non-negotiable.",
            "Enforce the linting rules. No exceptions. Not even for you.",
            "Who approved this without tests? Find them.",
            "The convention exists for a reason. Follow it.",
            "Review your own code as if someone else wrote it. Harshly.",
        ],
        Skill::Esprit => &[
            "The team will appreciate this cleanup. Morale matters.",
            "Pair programming energy: you're not alone in this.",
            "Someone left a helpful comment here. Acknowledge it.",
            "The build is green. Everyone can relax. For now.",
            "This is the kind of fix that earns respect at standup.",
        ],
        Skill::Suggestion => &[
            "Slip the refactor in alongside the bugfix. They won't mind.",
            "If you phrase it as a 'quick improvement,' it'll get approved.",
            "Plant the seed: mention the tech debt casually in the PR.",
            "Suggest the migration gently. Make them think it's their idea.",
            "The reviewer is tired. A small, clean diff will slide right through.",
        ],
        Skill::Endurance => &[
            "It's a 10,000-line file. You've handled worse. Keep going.",
            "The build takes 20 minutes. You can wait. You will wait.",
            "Another dependency conflict. Breathe. Resolve it. Move on.",
            "The monorepo is vast. But you are vaster.",
            "Legacy code cannot break you. You are unbreakable.",
        ],
        Skill::PainThreshold => &[
            "Yes, it's jQuery in 2026. You'll survive. Barely.",
            "The indentation mixes tabs and spaces. Absorb the pain.",
            "PHP 5.3. You can do this. Your eyes will heal.",
            "The entire app state is in a single global variable. Press on.",
            "No types. No tests. No docs. You've seen worse. Maybe.",
        ],
        Skill::PhysicalInstrument => &[
            "Just grep the entire codebase. Brute force works.",
            "Replace all instances. Every single one. No regex needed.",
            "Sometimes the answer is: delete everything and start over.",
            "Throw more compute at it. Elegance is for the weak.",
            "The O(n^2) solution will work fine. Ship it.",
        ],
        Skill::Electrochemistry => &[
            "Oh yes. Deploy it. DEPLOY IT NOW.",
            "Push to main. Feel the rush. The sweet, sweet dopamine.",
            "One more commit. Just one more. Then you'll stop.",
            "The green checkmark. You need it. You NEED it.",
            "Force push. Live dangerously. You know you want to.",
        ],
        Skill::Shivers => &[
            "A cold wind blows through the server rack. Something has changed.",
            "The network is quiet. Too quiet.",
            "Somewhere, a container just restarted. Can you feel it?",
            "The latency spike ripples outward. Something upstream shifted.",
            "Every microservice in the cluster holds its breath.",
        ],
        Skill::HalfLight => &[
            "Don't do it. Something about this command feels deeply wrong.",
            "Your palms are sweating. This is a destructive operation.",
            "DANGER. Step away from the production database.",
            "That rm command is one flag away from disaster.",
            "The fight-or-flight response is correct here. Choose flight.",
        ],
        Skill::HandEyeCoordination => &[
            "Careful. One character off and the whole thing breaks.",
            "The edit needs to be surgical. Precise. No collateral damage.",
            "Line 47, column 12. That's where the fix goes. Exactly.",
            "Thread the needle between the opening and closing brace.",
            "Your cursor knows where to go. Trust the muscle memory.",
        ],
        Skill::Perception => &[
            "There. Line 203. An off-by-one error. Almost invisible.",
            "The semicolon is missing. Right there. Do you see it?",
            "That variable is shadowed. Three scopes up. Subtle.",
            "The test coverage has a gap. Right at the boundary condition.",
            "Notice the trailing whitespace. It'll fail the linter.",
        ],
        Skill::ReactionSpeed => &[
            "The build is failing — hotfix NOW before anyone notices.",
            "Revert first, investigate later. Speed over understanding.",
            "The alert just fired. You have seconds, not minutes.",
            "Quick — the CI window closes in two minutes.",
            "Muscle memory. Fix it before the conscious mind catches up.",
        ],
        Skill::Savoir => &[
            "This code should not merely work. It should be *beautiful*.",
            "Add a blank line there. Aesthetic whitespace matters.",
            "The function name should roll off the tongue. Rename it.",
            "Elegance isn't optional. It's the whole point.",
            "Anyone can solve the problem. Solve it with *style*.",
        ],
        Skill::Interfacing => &[
            "The API expects camelCase, not snake_case. Watch the boundary.",
            "Check the rate limit before you fire off that request.",
            "The third-party SDK has an undocumented quirk. I've seen it.",
            "That webhook payload format changed last month. Verify it.",
            "The auth token expires in 15 minutes. Time your calls.",
        ],
        Skill::Composure => &[
            "Stay calm. The CI will pass. Probably.",
            "The production error rate is spiking. Keep your composure.",
            "Panicking won't fix the build. Steady hands.",
            "The deadline is tomorrow. You are made of stone. Unshakeable.",
            "Three merge conflicts. So what. Resolve them methodically.",
        ],
    }
}

pub fn passive_interjections(
    character: &Character,
    tool: &str,
    context: &str,
) -> Vec<Interjection> {
    let primary = Skill::for_tool(tool, context);
    let mut rng = rand::rng();
    let mut results: Vec<Interjection> = Vec::new();
    for skill in Skill::all() {
        if *skill == primary {
            continue;
        }
        let effective = character.effective_skill(*skill);
        if effective <= 4 {
            continue;
        }
        let chance = (effective - 4) as f64 * 0.10;
        if rng.random::<f64>() >= chance {
            continue;
        }
        let messages = interjection_messages(*skill);
        let msg = messages[rng.random_range(0..messages.len())];
        results.push(Interjection {
            skill: *skill,
            message: msg.to_string(),
        });
        if results.len() >= 3 {
            break;
        }
    }
    results
}
