use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CheckColor {
    #[default]
    White,
    Red,
}

impl CheckColor {
    pub fn for_action(tool: &str, context: &str) -> Self {
        let ctx = context.to_lowercase();
        match tool {
            "Bash" | "bash"
                if ctx.contains("rm ")
                    || ctx.contains("reset --hard")
                    || ctx.contains("--force")
                    || ctx.contains("drop ")
                    || ctx.contains("push") =>
            {
                CheckColor::Red
            }
            "Bash" | "bash" if ctx.contains("deploy") => CheckColor::Red,
            _ => CheckColor::White,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            CheckColor::White => "White",
            CheckColor::Red => "Red",
        }
    }
}

impl std::fmt::Display for CheckColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
