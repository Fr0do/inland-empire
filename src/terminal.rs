use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    ITerm2,
    Kitty,
    WezTerm,
    Alacritty,
    Generic,
}

impl Terminal {
    pub fn detect() -> Self {
        // Manual override takes priority
        if let Ok(val) = std::env::var("IE_TERMINAL") {
            return match val.as_str() {
                "iterm2" | "ITerm2" | "iTerm2" => Terminal::ITerm2,
                "kitty" | "Kitty" => Terminal::Kitty,
                "wezterm" | "WezTerm" => Terminal::WezTerm,
                "alacritty" | "Alacritty" => Terminal::Alacritty,
                _ => Terminal::Generic,
            };
        }
        // Auto-detect via $TERM_PROGRAM
        match std::env::var("TERM_PROGRAM").as_deref() {
            Ok("iTerm.app") => Terminal::ITerm2,
            Ok("kitty") => Terminal::Kitty,
            Ok("WezTerm") => Terminal::WezTerm,
            Ok("Alacritty") => Terminal::Alacritty,
            _ => Terminal::Generic,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Terminal::ITerm2 => "iTerm2",
            Terminal::Kitty => "Kitty",
            Terminal::WezTerm => "WezTerm",
            Terminal::Alacritty => "Alacritty",
            Terminal::Generic => "Generic",
        }
    }

    #[allow(dead_code)]
    pub fn supports_color(&self) -> bool {
        true
    }

    #[allow(dead_code)]
    pub fn supports_images(&self) -> bool {
        matches!(self, Terminal::ITerm2 | Terminal::Kitty | Terminal::WezTerm)
    }

    #[allow(dead_code)]
    pub fn image_protocol(&self) -> Option<&'static str> {
        match self {
            Terminal::ITerm2 => Some("iterm2"),
            Terminal::Kitty => Some("kitty"),
            Terminal::WezTerm => Some("sixel"),
            Terminal::Alacritty | Terminal::Generic => None,
        }
    }

    #[allow(dead_code)]
    pub fn notify(&self, title: &str, message: &str) {
        match self {
            Terminal::ITerm2 => eprint!("\x1b]9;{}\x07", message),
            Terminal::Kitty => eprint!("\x1b]99;i=1:d=0;{}: {}\x1b\\", title, message),
            Terminal::WezTerm => eprint!("\x1b]9;{}\x07", message),
            Terminal::Alacritty | Terminal::Generic => eprint!("\x07"),
        }
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
