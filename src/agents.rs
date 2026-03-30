use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent { ClaudeCode, Cursor, Aider, Windsurf, Copilot, Unknown }

impl Agent {
    pub fn detect() -> Self {
        if std::env::var("CLAUDE_TOOL_NAME").is_ok() { return Agent::ClaudeCode; }
        if std::env::var("CURSOR_TOOL").is_ok() { return Agent::Cursor; }
        if std::env::var("AIDER_TOOL").is_ok() { return Agent::Aider; }
        Agent::Unknown
    }
    pub fn name(&self) -> &'static str {
        match self { Agent::ClaudeCode => "Claude Code", Agent::Cursor => "Cursor", Agent::Aider => "Aider", Agent::Windsurf => "Windsurf", Agent::Copilot => "Copilot", Agent::Unknown => "Unknown" }
    }
}
impl fmt::Display for Agent { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.name()) } }
