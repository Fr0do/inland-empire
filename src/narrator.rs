use crate::character::Character;
use crate::checks::CheckResult;
use crate::time::TimeOfDay;
use chrono::Utc;
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub enum NarratorTrigger {
    CriticalSuccess,
    CriticalFailure,
    SessionIntro,
    FailStreak(usize),
    TimeChange(String),
    LevelUp(u32),
}

#[derive(Debug, Clone)]
pub struct NarratorEvent {
    pub trigger: NarratorTrigger,
    pub context: String,
    pub prompt: String,
}

impl NarratorEvent {
    pub fn to_json(&self) -> Value {
        let (trigger_str, extra) = match &self.trigger {
            NarratorTrigger::CriticalSuccess => ("critical_success", None),
            NarratorTrigger::CriticalFailure => ("critical_failure", None),
            NarratorTrigger::SessionIntro => ("session_intro", None),
            NarratorTrigger::FailStreak(n) => ("fail_streak", Some(json!(n))),
            NarratorTrigger::TimeChange(t) => ("time_change", Some(json!(t))),
            NarratorTrigger::LevelUp(lvl) => ("level_up", Some(json!(lvl))),
        };
        let mut obj = json!({
            "narrator": true,
            "trigger": trigger_str,
            "context": self.context,
            "prompt": self.prompt,
        });
        if let Some(val) = extra {
            let key = match &self.trigger {
                NarratorTrigger::FailStreak(_) => "streak",
                NarratorTrigger::TimeChange(_) => "time_of_day",
                NarratorTrigger::LevelUp(_) => "level",
                _ => "",
            };
            if !key.is_empty() {
                obj[key] = val;
            }
        }
        obj
    }
}

pub fn narrator_triggers(
    result: &CheckResult,
    character: &Character,
    tool: &str,
    context: &str,
) -> Vec<NarratorEvent> {
    let _ = tool;
    let mut events: Vec<NarratorEvent> = Vec::new();
    let skill_name = result.skill.to_string();
    let current_time = TimeOfDay::current();

    // Level up — return immediately with just this event
    if let Some(level) = result.level_up {
        events.push(NarratorEvent {
            trigger: NarratorTrigger::LevelUp(level),
            context: format!("{} on {}", skill_name, context),
            prompt: format!(
                "LEVEL UP to {}! Describe an epic moment of growth and revelation.",
                level
            ),
        });
        return events;
    }

    // Critical success or failure (mutually exclusive)
    if result.critical_success {
        events.push(NarratorEvent {
            trigger: NarratorTrigger::CriticalSuccess,
            context: format!("{} on {}", skill_name, context),
            prompt: format!(
                "Describe a dramatic moment of clarity as {} critically succeeds on {}. The dice show double sixes.",
                skill_name, context
            ),
        });
    } else if result.critical_failure {
        events.push(NarratorEvent {
            trigger: NarratorTrigger::CriticalFailure,
            context: format!("{} on {}", skill_name, context),
            prompt: format!(
                "Describe a devastating failure as {} critically fails on {}. Snake eyes. The detective's confidence shatters.",
                skill_name, context
            ),
        });
    }

    // Session intro — no checks in last 30 minutes
    // perform_check already appended the current check, so history.len() >= 1
    let history = &character.check_history;
    let is_first_of_session = if history.len() <= 1 {
        true
    } else {
        let prev = &history[history.len() - 2];
        let elapsed = Utc::now().signed_duration_since(prev.timestamp);
        elapsed.num_minutes() >= 30
    };

    if is_first_of_session {
        let checks_total = history.len();
        events.push(NarratorEvent {
            trigger: NarratorTrigger::SessionIntro,
            context: format!("{} checks lifetime", checks_total),
            prompt: format!(
                "The detective returns to work. A new session begins. Set the scene — {}, {} checks lifetime.",
                current_time.label(),
                checks_total
            ),
        });
        // Session intro + critical can coexist; skip time-change on fresh session
        events.truncate(2);
        return events;
    }

    // Time of day change
    if history.len() >= 2 {
        let prev = &history[history.len() - 2];
        let prev_time = TimeOfDay::from_timestamp(prev.timestamp);
        if prev_time.label() != current_time.label() {
            events.push(NarratorEvent {
                trigger: NarratorTrigger::TimeChange(current_time.label().to_string()),
                context: format!("was {}, now {}", prev_time.label(), current_time.label()),
                prompt: format!(
                    "The light has changed. It's now {}. Describe the shift in atmosphere.",
                    current_time.label()
                ),
            });
        }
    }

    // Fail streak — 3+ consecutive failures from end of check_history (current check included)
    let streak = consecutive_failures_from_end(history);
    if streak >= 3 {
        events.push(NarratorEvent {
            trigger: NarratorTrigger::FailStreak(streak),
            context: format!("{} consecutive failures", streak),
            prompt: format!(
                "The detective has failed {} checks in a row. Describe the mounting frustration, the doubt creeping in.",
                streak
            ),
        });
    }

    events.truncate(2);
    events
}

fn consecutive_failures_from_end(history: &[crate::character::CheckRecord]) -> usize {
    let mut count = 0;
    for record in history.iter().rev() {
        if !record.success {
            count += 1;
        } else {
            break;
        }
    }
    count
}

impl TimeOfDay {
    pub fn from_timestamp(ts: chrono::DateTime<chrono::Utc>) -> Self {
        use chrono::Timelike;
        let local: chrono::DateTime<chrono::Local> = ts.into();
        match local.hour() {
            6..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=23 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }
}
