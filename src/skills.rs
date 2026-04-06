use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attribute {
    Intellect,
    Psyche,
    Physique,
    Motorics,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Intellect => write!(f, "INTELLECT"),
            Attribute::Psyche => write!(f, "PSYCHE"),
            Attribute::Physique => write!(f, "PHYSIQUE"),
            Attribute::Motorics => write!(f, "MOTORICS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    // Intellect
    Logic,
    Encyclopedia,
    Rhetoric,
    Drama,
    Conceptualization,
    VisualCalculus,
    // Psyche
    Volition,
    InlandEmpire,
    Empathy,
    Authority,
    Esprit,
    Suggestion,
    // Physique
    Endurance,
    PainThreshold,
    PhysicalInstrument,
    Electrochemistry,
    Shivers,
    HalfLight,
    // Motorics
    HandEyeCoordination,
    Perception,
    ReactionSpeed,
    Savoir,
    Interfacing,
    Composure,
}

impl Skill {
    pub fn attribute(&self) -> Attribute {
        match self {
            Skill::Logic
            | Skill::Encyclopedia
            | Skill::Rhetoric
            | Skill::Drama
            | Skill::Conceptualization
            | Skill::VisualCalculus => Attribute::Intellect,
            Skill::Volition
            | Skill::InlandEmpire
            | Skill::Empathy
            | Skill::Authority
            | Skill::Esprit
            | Skill::Suggestion => Attribute::Psyche,
            Skill::Endurance
            | Skill::PainThreshold
            | Skill::PhysicalInstrument
            | Skill::Electrochemistry
            | Skill::Shivers
            | Skill::HalfLight => Attribute::Physique,
            Skill::HandEyeCoordination
            | Skill::Perception
            | Skill::ReactionSpeed
            | Skill::Savoir
            | Skill::Interfacing
            | Skill::Composure => Attribute::Motorics,
        }
    }

    pub fn all() -> &'static [Skill] {
        &[
            Skill::Logic,
            Skill::Encyclopedia,
            Skill::Rhetoric,
            Skill::Drama,
            Skill::Conceptualization,
            Skill::VisualCalculus,
            Skill::Volition,
            Skill::InlandEmpire,
            Skill::Empathy,
            Skill::Authority,
            Skill::Esprit,
            Skill::Suggestion,
            Skill::Endurance,
            Skill::PainThreshold,
            Skill::PhysicalInstrument,
            Skill::Electrochemistry,
            Skill::Shivers,
            Skill::HalfLight,
            Skill::HandEyeCoordination,
            Skill::Perception,
            Skill::ReactionSpeed,
            Skill::Savoir,
            Skill::Interfacing,
            Skill::Composure,
        ]
    }

    pub fn claude_domain(&self) -> &'static str {
        match self {
            Skill::Logic => "reasoning through complex code paths",
            Skill::Encyclopedia => "recalling APIs, docs, and obscure knowledge",
            Skill::Rhetoric => "arguing for architectural decisions",
            Skill::Drama => "detecting lies in error messages and logs",
            Skill::Conceptualization => "creative problem-solving and abstractions",
            Skill::VisualCalculus => "spatial reasoning over code structure",
            Skill::Volition => "resisting scope creep and staying on task",
            Skill::InlandEmpire => "gut feelings about code smells",
            Skill::Empathy => "understanding user intent and UX",
            Skill::Authority => "enforcing code standards and conventions",
            Skill::Esprit => "team morale and collaborative coding",
            Skill::Suggestion => "persuading the user to accept refactors",
            Skill::Endurance => "handling massive codebases without fatigue",
            Skill::PainThreshold => "tolerating legacy code and tech debt",
            Skill::PhysicalInstrument => "brute-force solutions and raw throughput",
            Skill::Electrochemistry => "the thrill of deploying to production",
            Skill::Shivers => "sensing the state of the system holistically",
            Skill::HalfLight => "fight-or-flight on critical production bugs",
            Skill::HandEyeCoordination => "precise surgical edits to code",
            Skill::Perception => "noticing subtle bugs and edge cases",
            Skill::ReactionSpeed => "quick hotfixes and incident response",
            Skill::Savoir => "writing elegant, stylish code",
            Skill::Interfacing => "working with external APIs and systems",
            Skill::Composure => "staying calm under failing CI pipelines",
        }
    }

    pub fn for_tool(tool: &str, context: &str) -> Self {
        let ctx = context.to_lowercase();
        match tool {
            "Bash" | "bash" => bash_skill(&ctx),
            "Read" | "read" => {
                if ctx.contains("test") || ctx.contains("spec") {
                    Skill::Logic
                } else if ctx.contains(".lock") {
                    Skill::Composure
                } else if ctx.contains("readme") || ctx.contains("doc") || ctx.contains(".md") {
                    Skill::Encyclopedia
                } else if ctx.contains("config")
                    || ctx.contains(".toml")
                    || ctx.contains(".yaml")
                    || ctx.contains(".json")
                    || ctx.contains(".env")
                {
                    Skill::VisualCalculus
                } else {
                    Skill::Perception
                }
            }
            "Edit" | "edit" => {
                if ctx.contains("test") || ctx.contains("spec") {
                    Skill::Logic
                } else if ctx.contains("config")
                    || ctx.contains(".toml")
                    || ctx.contains(".yaml")
                {
                    Skill::VisualCalculus
                } else if ctx.contains("css")
                    || ctx.contains("style")
                    || ctx.contains("ui")
                    || ctx.contains("html")
                {
                    Skill::Conceptualization
                } else if ctx.contains("error")
                    || ctx.contains("catch")
                    || ctx.contains("panic")
                {
                    Skill::PainThreshold
                } else {
                    Skill::HandEyeCoordination
                }
            }
            "Write" | "write" => {
                if ctx.contains("test") || ctx.contains("spec") {
                    Skill::Logic
                } else if ctx.contains("config")
                    || ctx.contains(".toml")
                    || ctx.contains(".yaml")
                {
                    Skill::VisualCalculus
                } else {
                    Skill::Conceptualization
                }
            }
            "Glob" | "glob" => Skill::VisualCalculus,
            "Grep" | "grep" => {
                if ctx.contains("error") || ctx.contains("bug") || ctx.contains("fix") {
                    Skill::Perception
                } else if ctx.contains("todo") || ctx.contains("hack") || ctx.contains("fixme") {
                    Skill::InlandEmpire
                } else {
                    Skill::Encyclopedia
                }
            }
            "WebFetch" | "web_fetch" => Skill::Interfacing,
            "WebSearch" | "web_search" => Skill::Encyclopedia,
            "Agent" | "agent" => Skill::Authority,
            _ => Skill::Logic,
        }
    }
}

fn bash_skill(ctx: &str) -> Skill {
    if ctx.contains("git push") {
        Skill::Electrochemistry
    } else if ctx.contains("git commit") {
        Skill::Rhetoric
    } else if ctx.contains("git log")
        || ctx.contains("git blame")
        || ctx.contains("git diff")
        || ctx.contains("git show")
    {
        Skill::Encyclopedia
    } else if ctx.contains("git reset") || ctx.contains("git checkout --") {
        Skill::HalfLight
    } else if ctx.contains("git merge") || ctx.contains("git rebase") {
        Skill::Authority
    } else if ctx.contains("git stash") || ctx.contains("git branch") {
        Skill::Savoir
    } else if ctx.contains("cargo test") || ctx.contains("pytest") || ctx.contains("npm test") {
        Skill::Logic
    } else if ctx.contains("cargo build") || ctx.contains("make") || ctx.contains("npm run build") {
        Skill::Endurance
    } else if ctx.contains("cargo clippy") || ctx.contains("lint") {
        Skill::Perception
    } else if ctx.contains("rm ") || ctx.contains("del ") || ctx.contains("drop ") || ctx.contains("truncate") {
        Skill::HalfLight
    } else if ctx.contains("docker")
        || ctx.contains("kubectl")
        || ctx.contains("k8s")
        || ctx.contains("terraform")
    {
        Skill::Interfacing
    } else if ctx.contains("curl") || ctx.contains("wget") || ctx.contains("http") || ctx.contains("fetch") {
        Skill::Esprit
    } else if ctx.contains("npm install") || ctx.contains("pip install") || ctx.contains("cargo add") {
        Skill::Electrochemistry
    } else if ctx.contains("ssh") || ctx.contains("scp") {
        Skill::Shivers
    } else if ctx.contains("env")
        || ctx.contains("secret")
        || ctx.contains("token")
        || ctx.contains("key")
        || ctx.contains("credential")
    {
        Skill::Composure
    } else if ctx.contains("sed") || ctx.contains("awk") || ctx.contains("grep") {
        Skill::VisualCalculus
    } else if ctx.contains("chmod") || ctx.contains("chown") || ctx.contains("sudo") {
        Skill::Authority
    } else if ctx.contains("echo") || ctx.contains("printf") || ctx.contains("cat") {
        Skill::Drama
    } else if ctx.contains("sleep") || ctx.contains("wait") {
        Skill::Volition
    } else {
        Skill::Interfacing
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Skill::Logic => "Logic",
            Skill::Encyclopedia => "Encyclopedia",
            Skill::Rhetoric => "Rhetoric",
            Skill::Drama => "Drama",
            Skill::Conceptualization => "Conceptualization",
            Skill::VisualCalculus => "Visual Calculus",
            Skill::Volition => "Volition",
            Skill::InlandEmpire => "Inland Empire",
            Skill::Empathy => "Empathy",
            Skill::Authority => "Authority",
            Skill::Esprit => "Esprit de Corps",
            Skill::Suggestion => "Suggestion",
            Skill::Endurance => "Endurance",
            Skill::PainThreshold => "Pain Threshold",
            Skill::PhysicalInstrument => "Physical Instrument",
            Skill::Electrochemistry => "Electrochemistry",
            Skill::Shivers => "Shivers",
            Skill::HalfLight => "Half Light",
            Skill::HandEyeCoordination => "Hand/Eye Coordination",
            Skill::Perception => "Perception",
            Skill::ReactionSpeed => "Reaction Speed",
            Skill::Savoir => "Savoir Faire",
            Skill::Interfacing => "Interfacing",
            Skill::Composure => "Composure",
        };
        write!(f, "{}", name)
    }
}

impl std::str::FromStr for Skill {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
            "logic" | "log" => Ok(Skill::Logic),
            "encyclopedia" | "enc" => Ok(Skill::Encyclopedia),
            "rhetoric" | "rhe" => Ok(Skill::Rhetoric),
            "drama" | "dra" => Ok(Skill::Drama),
            "conceptualization" | "con" => Ok(Skill::Conceptualization),
            "visualcalculus" | "vis" => Ok(Skill::VisualCalculus),
            "volition" | "vol" => Ok(Skill::Volition),
            "inlandempire" | "inl" => Ok(Skill::InlandEmpire),
            "empathy" | "emp" => Ok(Skill::Empathy),
            "authority" | "aut" => Ok(Skill::Authority),
            "espritdecorps" | "esprit" | "esp" => Ok(Skill::Esprit),
            "suggestion" | "sug" => Ok(Skill::Suggestion),
            "endurance" | "end" => Ok(Skill::Endurance),
            "painthreshold" | "pai" => Ok(Skill::PainThreshold),
            "physicalinstrument" | "phy" => Ok(Skill::PhysicalInstrument),
            "electrochemistry" | "ele" => Ok(Skill::Electrochemistry),
            "shivers" | "shi" => Ok(Skill::Shivers),
            "halflight" | "hal" => Ok(Skill::HalfLight),
            "handeyecoordination" | "han" => Ok(Skill::HandEyeCoordination),
            "perception" | "per" => Ok(Skill::Perception),
            "reactionspeed" | "rea" => Ok(Skill::ReactionSpeed),
            "savoirfaire" | "savoir" | "sav" => Ok(Skill::Savoir),
            "interfacing" | "int" => Ok(Skill::Interfacing),
            "composure" | "com" => Ok(Skill::Composure),
            _ => Err(format!("Unknown skill: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_tool() {
        assert_eq!(Skill::for_tool("Bash", "git push origin main"), Skill::Electrochemistry);
        assert_eq!(Skill::for_tool("Bash", "git commit -m 'fix'"), Skill::Rhetoric);
        assert_eq!(Skill::for_tool("Bash", "cargo test"), Skill::Logic);
        assert_eq!(Skill::for_tool("Bash", "rm -rf node_modules"), Skill::HalfLight);
        assert_eq!(Skill::for_tool("Read", "src/checks.rs"), Skill::Perception);
        assert_eq!(Skill::for_tool("Read", "config.toml"), Skill::VisualCalculus);
        assert_eq!(Skill::for_tool("Edit", "src/main_test.rs"), Skill::Logic);
        assert_eq!(Skill::for_tool("Bash", "docker-compose up"), Skill::Interfacing);
        assert_eq!(Skill::for_tool("Bash", "echo hello"), Skill::Drama);
    }
}
