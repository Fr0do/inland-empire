use crate::character::Character;
use crate::skills::{Attribute, Skill};
use colored::Colorize;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mood {
    Confident,
    Neutral,
    Anxious,
    Excited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerVoice {
    pub voice: Skill,
    pub mood: Mood,
    pub recent_successes: u8,
    pub recent_failures: u8,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum CompanionEvent {
    PostCheck { success: bool, critical: bool, skill: Skill },
    LevelUp,
    StreakBroken,
    TimeGreeting,
}

impl InnerVoice {
    pub fn from_character(ch: &Character) -> Self {
        let voice = Skill::all()
            .iter()
            .max_by_key(|&&s| ch.effective_skill(s))
            .copied()
            .unwrap_or(Skill::Logic);

        let recent: Vec<&crate::character::CheckRecord> =
            ch.check_history.iter().rev().take(10).collect();

        let recent_successes = recent.iter().filter(|r| r.success).count() as u8;
        let recent_failures = recent.iter().filter(|r| !r.success).count() as u8;

        let mood = if recent.first().map(|r| r.roll.0 == 6 && r.roll.1 == 6).unwrap_or(false) {
            Mood::Excited
        } else if recent.iter().take(3).all(|r| !r.success) && recent.len() >= 3 {
            Mood::Anxious
        } else if recent_successes > recent_failures {
            Mood::Confident
        } else {
            Mood::Neutral
        };

        InnerVoice { voice, mood, recent_successes, recent_failures }
    }

    pub fn update_after_check(&mut self, ch: &Character, success: bool, critical: bool) {
        if success {
            self.recent_successes = self.recent_successes.saturating_add(1).min(10);
            self.recent_failures = self.recent_failures.saturating_sub(1);
        } else {
            self.recent_failures = self.recent_failures.saturating_add(1).min(10);
            self.recent_successes = self.recent_successes.saturating_sub(1);
        }

        self.mood = if critical && success {
            Mood::Excited
        } else if self.recent_failures >= 3 {
            Mood::Anxious
        } else if self.recent_successes > self.recent_failures {
            Mood::Confident
        } else {
            Mood::Neutral
        };

        // Re-evaluate dominant skill
        let new_voice = Skill::all()
            .iter()
            .max_by_key(|&&s| ch.effective_skill(s))
            .copied()
            .unwrap_or(self.voice);
        self.voice = new_voice;
    }
}

pub fn commentary(voice: &InnerVoice, event: CompanionEvent, _ch: &Character) -> Option<String> {
    let mut rng = rand::rng();
    let lines: &[&str] = match (voice.voice, event) {
        // PostCheck success
        (Skill::Logic, CompanionEvent::PostCheck { success: true, critical: false, .. }) => &[
            "Probability favored us. As expected.",
            "The logic held. It always does.",
            "Correct. Move on.",
        ],
        (Skill::Logic, CompanionEvent::PostCheck { success: true, critical: true, .. }) => &[
            "Perfect execution. Statistically improbable. Noted.",
            "Double six. The math was always on our side.",
            "Maximum outcome. Efficient.",
        ],
        (Skill::Logic, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The variables were miscalculated. Recalibrate.",
            "Failure. Logged. Moving on.",
            "An outlier. Adjust the model.",
        ],
        (Skill::Encyclopedia, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Knew it. Read that in the docs.",
            "Filed under: things we already knew.",
            "The reference was correct, as always.",
        ],
        (Skill::Encyclopedia, CompanionEvent::PostCheck { success: false, .. }) => &[
            "There's a gap in the knowledge base.",
            "Must have missed that chapter.",
            "Filing this under: learn from it.",
        ],
        (Skill::Rhetoric, CompanionEvent::PostCheck { success: true, .. }) => &[
            "We made our case. The code speaks.",
            "Persuasive. Well argued.",
            "They accepted the premise. Good.",
        ],
        (Skill::Rhetoric, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The argument didn't land. Reframe it.",
            "They weren't convinced. Try again.",
            "Counter-argument. Adapt.",
        ],
        (Skill::Drama, CompanionEvent::PostCheck { success: true, critical: true, .. }) => &[
            "WHAT A PERFORMANCE! Standing ovation!",
            "The commit landed like a thunderclap!",
            "MAGNIFICENT! They'll talk about this one.",
        ],
        (Skill::Drama, CompanionEvent::PostCheck { success: true, .. }) => &[
            "A solid performance. They noticed.",
            "The stage is ours.",
            "Drama serves us well today.",
        ],
        (Skill::Drama, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The crowd is silent. Not in the good way.",
            "The curtain falls early.",
            "Off-script. Improvise.",
        ],
        (Skill::Conceptualization, CompanionEvent::PostCheck { success: true, .. }) => &[
            "The shape of it... beautiful.",
            "Abstract. Perfect. Ours.",
            "The concept crystallizes.",
        ],
        (Skill::Conceptualization, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The form isn't there yet.",
            "Still searching for the shape of it.",
            "Blurry. Needs focus.",
        ],
        (Skill::VisualCalculus, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Architecture aligns. Every piece placed.",
            "The structure holds. Clean.",
            "Spatial integrity confirmed.",
        ],
        (Skill::VisualCalculus, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The geometry is off somewhere.",
            "A misalignment. Find it.",
            "The structure bent under pressure.",
        ],
        (Skill::Volition, CompanionEvent::PostCheck { success: true, .. }) => &[
            "We push forward. That's what we do.",
            "Determination prevails.",
            "Will over circumstance. Always.",
        ],
        (Skill::Volition, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Not this time. But we don't stop.",
            "Setback. Irrelevant. Continue.",
            "The will is intact. Try again.",
        ],
        (Skill::InlandEmpire, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Something whispered the answer.",
            "The code knew. We listened.",
            "Intuition guided us true.",
        ],
        (Skill::InlandEmpire, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The signals were wrong this time.",
            "Even the deep knowing fails sometimes.",
            "The whispers lied. Curious.",
        ],
        (Skill::Empathy, CompanionEvent::PostCheck { success: true, .. }) => &[
            "The user will appreciate this. I feel it.",
            "We understood what they needed.",
            "Connection made. They're satisfied.",
        ],
        (Skill::Empathy, CompanionEvent::PostCheck { success: false, .. }) => &[
            "We missed what they really wanted.",
            "The need was deeper than we saw.",
            "Try to understand better next time.",
        ],
        (Skill::Authority, CompanionEvent::PostCheck { success: true, .. }) => &[
            "We own this codebase. Never forget.",
            "Authority asserted. Accepted.",
            "Dominance established.",
        ],
        (Skill::Authority, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Challenged. Unacceptable.",
            "The codebase resisted. Briefly.",
            "Authority questioned. Unpleasant.",
        ],
        (Skill::Esprit, CompanionEvent::PostCheck { success: true, .. }) => &[
            "The team would be proud.",
            "Collaborative energy. It shows.",
            "We did this together. Good.",
        ],
        (Skill::Esprit, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The team expects better.",
            "Morale dips. Recover it.",
            "Collective disappointment. Noted.",
        ],
        (Skill::Suggestion, CompanionEvent::PostCheck { success: true, .. }) => &[
            "They accepted it. Won't know why.",
            "The nudge worked. Subtle.",
            "Planted. Believed. Done.",
        ],
        (Skill::Suggestion, CompanionEvent::PostCheck { success: false, .. }) => &[
            "They saw through it.",
            "The suggestion didn't take.",
            "Resistance. Recalibrate the approach.",
        ],
        (Skill::Endurance, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Another check. We can do this all day.",
            "Resilience pays off.",
            "Still standing. That's enough.",
        ],
        (Skill::Endurance, CompanionEvent::PostCheck { success: false, .. }) => &[
            "We took the hit. We'll absorb it.",
            "Wore us down but didn't stop us.",
            "Endurance tested. Still here.",
        ],
        (Skill::PainThreshold, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Felt nothing. Truly.",
            "Pain is information. We used it.",
            "Tough. As always.",
        ],
        (Skill::PainThreshold, CompanionEvent::PostCheck { success: false, .. }) => &[
            "That failure? Barely felt it.",
            "Hurts less than expected.",
            "Walk it off.",
        ],
        (Skill::PhysicalInstrument, CompanionEvent::PostCheck { success: true, .. }) => &[
            "CRUSHED IT.",
            "Brute force works. Every time.",
            "Raw power. Applied correctly.",
        ],
        (Skill::PhysicalInstrument, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Hit the wall. Hit it again.",
            "Not hard enough. More force.",
            "The obstacle persists. Unacceptable.",
        ],
        (Skill::Electrochemistry, CompanionEvent::PostCheck { success: true, .. }) => &[
            "The rush of a clean deploy. Nothing like it.",
            "Electric. Absolutely electric.",
            "Hit different this time.",
        ],
        (Skill::Electrochemistry, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The crash after the high. Expected.",
            "Burned out. Rest, then try again.",
            "Too much, too fast.",
        ],
        (Skill::Shivers, CompanionEvent::PostCheck { success: true, .. }) => &[
            "The system hums. Everything connects.",
            "Felt it before it happened.",
            "The atmosphere was right.",
        ],
        (Skill::Shivers, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The resonance was off.",
            "Couldn't read the room.",
            "The signal was there. We missed it.",
        ],
        (Skill::HalfLight, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Threat neutralized. Stay alert.",
            "Saw it coming. Acted faster.",
            "Fight response. Correct.",
        ],
        (Skill::HalfLight, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The next check could be the one that breaks us.",
            "Threat response failed. Danger.",
            "Stay paranoid. It's healthy.",
        ],
        (Skill::HandEyeCoordination, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Surgical. Not a line out of place.",
            "Precise. Intentional. Clean.",
            "Perfect coordination.",
        ],
        (Skill::HandEyeCoordination, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The aim was off.",
            "Missed by a fraction. Recalibrate.",
            "Tremor in the execution.",
        ],
        (Skill::Perception, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Noticed something others would miss.",
            "There it is. Hidden in plain sight.",
            "Observation confirmed.",
        ],
        (Skill::Perception, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Something was there. We didn't see it.",
            "Check the output again. Carefully.",
            "Perception failed. Look harder.",
        ],
        (Skill::ReactionSpeed, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Fast fix. No hesitation.",
            "Before the problem fully formed. Done.",
            "Instant response. Good.",
        ],
        (Skill::ReactionSpeed, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Too slow.",
            "Reacted late. Cost us.",
            "Speed matters. Remember that.",
        ],
        (Skill::Savoir, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Elegant. That's how we do things.",
            "Style and substance. Both.",
            "Executed with flair.",
        ],
        (Skill::Savoir, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Clumsy. Embarrassing.",
            "The grace wasn't there.",
            "Style abandoned us. Temporarily.",
        ],
        (Skill::Interfacing, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Clean handshake. System responded well.",
            "Interface accepted. Nominal.",
            "The connection held.",
        ],
        (Skill::Interfacing, CompanionEvent::PostCheck { success: false, .. }) => &[
            "The system rejected the request.",
            "Bad interface. Check the protocol.",
            "Connection refused.",
        ],
        (Skill::Composure, CompanionEvent::PostCheck { success: true, .. }) => &[
            "Steady. Nothing we haven't handled.",
            "Calm under pressure. As expected.",
            "Unshaken.",
        ],
        (Skill::Composure, CompanionEvent::PostCheck { success: false, .. }) => &[
            "Rattled. Recompose.",
            "The calm broke. Rebuild it.",
            "Composure slipped. Recover.",
        ],
        // LevelUp — all skills
        (_, CompanionEvent::LevelUp) => match voice.voice {
            Skill::Logic => &["Growth. Quantified.", "The numbers improve. Good."],
            Skill::Encyclopedia => &["Another chapter written.", "The index expands."],
            Skill::Rhetoric => &["The argument grows stronger.", "More words. Better ones."],
            Skill::Drama => &["A new act begins!", "The performance deepens!"],
            Skill::Conceptualization => &["The vision expands.", "New shapes become possible."],
            Skill::VisualCalculus => &["The blueprint grows.", "New structures unlock."],
            Skill::Volition => &["Will forged stronger.", "Resolve deepened."],
            Skill::InlandEmpire => &["The inner world grows.", "Deeper knowing."],
            Skill::Empathy => &["Understanding deepens.", "Closer to them now."],
            Skill::Authority => &["Rank increases.", "Command strengthens."],
            Skill::Esprit => &["The team feels it.", "Collective rise."],
            Skill::Suggestion => &["The influence grows.", "Subtler. Stronger."],
            Skill::Endurance => &["Harder now. Better.", "More to give."],
            Skill::PainThreshold => &["Higher threshold. Good.", "Less felt now."],
            Skill::PhysicalInstrument => &["Stronger.", "More force available."],
            Skill::Electrochemistry => &["New highs possible.", "The chemistry shifts."],
            Skill::Shivers => &["The city speaks louder.", "More signals now."],
            Skill::HalfLight => &["Sharper instincts.", "The threat-sense grows."],
            Skill::HandEyeCoordination => &["Finer control.", "Precision increases."],
            Skill::Perception => &["Eyes sharper now.", "More is visible."],
            Skill::ReactionSpeed => &["Faster now.", "The gap shrinks."],
            Skill::Savoir => &["More refined.", "Elegance improves."],
            Skill::Interfacing => &["Better protocol.", "Cleaner connection."],
            Skill::Composure => &["Stiller now.", "The calm holds longer."],
        },
        // StreakBroken
        (_, CompanionEvent::StreakBroken) => &[
            "Finally. The streak ends.",
            "The losing run is over.",
            "Back on track.",
        ],
        // TimeGreeting
        (Skill::Logic, CompanionEvent::TimeGreeting) => &["Ready. What's the problem?"],
        (Skill::Encyclopedia, CompanionEvent::TimeGreeting) => &["Let's see what today requires."],
        (Skill::Rhetoric, CompanionEvent::TimeGreeting) => &["Ready to make our case."],
        (Skill::Drama, CompanionEvent::TimeGreeting) => &["The stage is set. Begin."],
        (Skill::Conceptualization, CompanionEvent::TimeGreeting) => &["The mind is open."],
        (Skill::VisualCalculus, CompanionEvent::TimeGreeting) => &["The layout is clear."],
        (Skill::Volition, CompanionEvent::TimeGreeting) => &["We push forward."],
        (Skill::InlandEmpire, CompanionEvent::TimeGreeting) => &["Something stirs. Pay attention."],
        (Skill::Empathy, CompanionEvent::TimeGreeting) => &["What does the user need today?"],
        (Skill::Authority, CompanionEvent::TimeGreeting) => &["We're in charge here."],
        (Skill::Esprit, CompanionEvent::TimeGreeting) => &["The team is ready."],
        (Skill::Suggestion, CompanionEvent::TimeGreeting) => &["They'll accept what we offer."],
        (Skill::Endurance, CompanionEvent::TimeGreeting) => &["Another day. We endure."],
        (Skill::PainThreshold, CompanionEvent::TimeGreeting) => &["Bring it."],
        (Skill::PhysicalInstrument, CompanionEvent::TimeGreeting) => &["Ready to work."],
        (Skill::Electrochemistry, CompanionEvent::TimeGreeting) => &["Let's get that rush going."],
        (Skill::Shivers, CompanionEvent::TimeGreeting) => &["The city hums. Listen."],
        (Skill::HalfLight, CompanionEvent::TimeGreeting) => &["Stay sharp out there."],
        (Skill::HandEyeCoordination, CompanionEvent::TimeGreeting) => &["Steady hands. Let's go."],
        (Skill::Perception, CompanionEvent::TimeGreeting) => &["Eyes open. Miss nothing."],
        (Skill::ReactionSpeed, CompanionEvent::TimeGreeting) => &["Fast. Stay fast."],
        (Skill::Savoir, CompanionEvent::TimeGreeting) => &["Do it with style."],
        (Skill::Interfacing, CompanionEvent::TimeGreeting) => &["Systems ready."],
        (Skill::Composure, CompanionEvent::TimeGreeting) => &["Stay calm."],
    };

    lines.choose(&mut rng).map(|s| s.to_string())
}

pub fn format_companion_line(voice: &InnerVoice, text: &str) -> String {
    let skill_name = voice.voice.to_string().to_uppercase();
    let colored_name = match voice.voice.attribute() {
        Attribute::Intellect => skill_name.blue().bold().to_string(),
        Attribute::Psyche => skill_name.magenta().bold().to_string(),
        Attribute::Physique => skill_name.red().bold().to_string(),
        Attribute::Motorics => skill_name.yellow().bold().to_string(),
    };
    format!("{} — {}", colored_name, text)
}

pub fn companion_status(voice: &InnerVoice) -> String {
    let mood_icon = match voice.mood {
        Mood::Confident => "😎",
        Mood::Neutral => "🫥",
        Mood::Anxious => "😰",
        Mood::Excited => "🤩",
    };
    let skill_name = voice.voice.to_string().to_uppercase();
    let colored_name = match voice.voice.attribute() {
        Attribute::Intellect => skill_name.blue().bold().to_string(),
        Attribute::Psyche => skill_name.magenta().bold().to_string(),
        Attribute::Physique => skill_name.red().bold().to_string(),
        Attribute::Motorics => skill_name.yellow().bold().to_string(),
    };
    let mood_str = format!("{:?}", voice.mood);
    format!(
        "Inner Voice\n  Voice  {}\n  Mood   {} {}\n  Recent ✓{} ✗{}",
        colored_name,
        mood_icon,
        mood_str,
        voice.recent_successes,
        voice.recent_failures,
    )
}
