use crate::character::Character;
use crate::equipment::EquipSlot;
use crate::substances::Substance;
use colored::Colorize;

// ── Face state ────────────────────────────────────────────────────────────────

struct Face {
    eyes: &'static str,
    mouth: &'static str,
    smoke: bool,
    color_fn: fn(&str) -> colored::ColoredString,
}

fn identity(s: &str) -> colored::ColoredString {
    s.normal()
}

fn face_for_character(ch: &Character) -> Face {
    let active: Vec<Substance> = ch.active_effects.iter().map(|e| e.substance).collect();

    for sub in &active {
        match sub {
            Substance::Lsd => return Face { eyes: "✿ ✿", mouth: " ◊ ", smoke: false, color_fn: |s| s.magenta().bold() },
            Substance::Mushrooms => return Face { eyes: "☯ ☯", mouth: " ≋ ", smoke: false, color_fn: |s| s.green().bold() },
            Substance::Pyrholidon => return Face { eyes: "◈ ◈", mouth: " ▼ ", smoke: false, color_fn: |s| s.blue().bold() },
            Substance::Speed => return Face { eyes: "◎ ◎", mouth: " △ ", smoke: false, color_fn: |s| s.cyan().bold() },
            Substance::Alcohol => return Face { eyes: "≈ ≈", mouth: " ∪ ", smoke: false, color_fn: |s| s.yellow() },
            _ => {}
        }
    }
    for sub in &active {
        match sub {
            Substance::Cigarette => return Face { eyes: "° °", mouth: " ─╮", smoke: true, color_fn: identity },
            Substance::Coffee => return Face { eyes: "• •", mouth: " ‿ ", smoke: false, color_fn: identity },
            _ => {}
        }
    }

    let health_crit = ch.health <= 1;
    let health_half = ch.health <= ch.max_health / 2;
    let morale_crit = ch.morale <= 1;
    let morale_half = ch.morale <= ch.max_morale / 2;

    if health_crit {
        Face { eyes: "x x", mouth: " ▿ ", smoke: false, color_fn: |s| s.red() }
    } else if morale_crit {
        Face { eyes: "· ·", mouth: " ∩ ", smoke: false, color_fn: |s| s.bright_black() }
    } else if health_half {
        Face { eyes: "- -", mouth: " ~ ", smoke: false, color_fn: |s| s.yellow() }
    } else if morale_half {
        Face { eyes: "° °", mouth: " _ ", smoke: false, color_fn: identity }
    } else {
        Face { eyes: "° °", mouth: " ▽ ", smoke: false, color_fn: identity }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn slot_name(ch: &Character, slot: EquipSlot) -> Option<String> {
    ch.loadout.get(slot).map(|item| item.name.clone())
}

fn slot_color(slot: EquipSlot, text: &str) -> String {
    match slot {
        EquipSlot::Hat => text.blue().bold().to_string(),
        EquipSlot::Jacket => text.magenta().bold().to_string(),
        EquipSlot::Shirt => text.cyan().to_string(),
        EquipSlot::Pants => text.yellow().bold().to_string(),
        EquipSlot::Shoes => text.green().bold().to_string(),
        EquipSlot::Accessory => text.bright_cyan().bold().to_string(),
    }
}

fn slot_fill(slot: EquipSlot, text: &str) -> String {
    match slot {
        EquipSlot::Hat => text.on_blue().white().to_string(),
        EquipSlot::Jacket => text.magenta().to_string(),
        EquipSlot::Shirt => text.cyan().to_string(),
        EquipSlot::Pants => text.yellow().to_string(),
        EquipSlot::Shoes => text.green().to_string(),
        EquipSlot::Accessory => text.bright_cyan().to_string(),
    }
}

// ── Full body portrait ──────────────────────────────────────────────────────

pub fn render_character(ch: &Character) -> String {
    let face = face_for_character(ch);
    let cf = face.color_fn;

    let has_hat = ch.loadout.get(EquipSlot::Hat).is_some();
    let has_jacket = ch.loadout.get(EquipSlot::Jacket).is_some();
    let has_shirt = ch.loadout.get(EquipSlot::Shirt).is_some();
    let has_pants = ch.loadout.get(EquipSlot::Pants).is_some();
    let has_shoes = ch.loadout.get(EquipSlot::Shoes).is_some();

    // ── Hat ──
    let hat_line = if has_hat {
        format!("      {}", slot_fill(EquipSlot::Hat, "╔═════════╗"))
    } else {
        format!("      {}", "┌─────────┐".dimmed())
    };

    // ── Head ──
    let head_l = if has_hat { slot_fill(EquipSlot::Hat, "║") } else { "│".dimmed().to_string() };
    let head_r = if has_hat { slot_fill(EquipSlot::Hat, "║") } else { "│".dimmed().to_string() };
    let head_bot = if has_hat {
        format!("      {}", slot_fill(EquipSlot::Hat, "╚════╤════╝"))
    } else {
        format!("      {}", "└────┬────┘".dimmed())
    };

    let smoke = if face.smoke {
        format!("           {}", "~~~".bright_black())
    } else {
        String::new()
    };

    let eyes_line = format!("      {}  {}  {}", head_l, cf(face.eyes), head_r);
    let mouth_line = format!("      {} {} {}", head_l, cf(face.mouth), head_r);

    // ── Torso ──
    let (torso_tl, torso_tr, torso_fill, torso_bl, torso_br) = if has_jacket {
        (slot_fill(EquipSlot::Jacket, "╔═══"), slot_fill(EquipSlot::Jacket, "═══╗"),
         slot_fill(EquipSlot::Jacket, "║"), slot_fill(EquipSlot::Jacket, "╚═══"), slot_fill(EquipSlot::Jacket, "═══╝"))
    } else {
        ("┌───".dimmed().to_string(), "───┐".dimmed().to_string(),
         "│".dimmed().to_string(), "└───".dimmed().to_string(), "───┘".dimmed().to_string())
    };

    let shirt_inner = if has_shirt {
        slot_fill(EquipSlot::Shirt, "═══════")
    } else {
        "       ".to_string()
    };

    let torso_top = format!("     {}╧{}", torso_tl, torso_tr);
    let torso1 = format!("     {} {} {}", torso_fill, shirt_inner, torso_fill);
    let torso2 = format!("     {} {} {}", torso_fill, shirt_inner, torso_fill);
    let torso_bot = format!("     {}╤{}", torso_bl, torso_br);

    // ── Legs ──
    let (leg_l, leg_r, leg_fill) = if has_pants {
        (slot_fill(EquipSlot::Pants, "┌────"), slot_fill(EquipSlot::Pants, "────┐"),
         slot_fill(EquipSlot::Pants, "│"))
    } else {
        ("┌────".dimmed().to_string(), "────┐".dimmed().to_string(),
         "│".dimmed().to_string())
    };
    let leg_bl = if has_pants { slot_fill(EquipSlot::Pants, "└──") } else { "└──".dimmed().to_string() };
    let leg_br = if has_pants { slot_fill(EquipSlot::Pants, "──┘") } else { "──┘".dimmed().to_string() };

    let leg_top = format!("     {}┴{}", leg_l, leg_r);
    let leg1 = format!("     {}         {}", leg_fill, leg_fill);
    let leg2 = format!("     {}         {}", leg_fill, leg_fill);
    let leg_bot = format!("      {}┬───┬{}", leg_bl, leg_br);

    // ── Shoes ──
    let (shoe_l, shoe_r) = if has_shoes {
        (slot_fill(EquipSlot::Shoes, "╔══╧╗"), slot_fill(EquipSlot::Shoes, "╔╧══╗"))
    } else {
        ("┌──┴┐".dimmed().to_string(), "┌┴──┐".dimmed().to_string())
    };
    let shoe_bl = if has_shoes { slot_fill(EquipSlot::Shoes, "╚═══╝") } else { "└───┘".dimmed().to_string() };
    let shoe_br = if has_shoes { slot_fill(EquipSlot::Shoes, "╚═══╝") } else { "└───┘".dimmed().to_string() };

    let shoes_top = format!("     {} {}", shoe_l, shoe_r);
    let shoes_bot = format!("     {} {}", shoe_bl, shoe_br);

    // ── Labels on the right ──
    let arrow = "◄─".dimmed().to_string();
    let hat_lbl = slot_name(ch, EquipSlot::Hat).map(|n| slot_color(EquipSlot::Hat, &n)).unwrap_or("(none)".dimmed().to_string());
    let jacket_lbl = slot_name(ch, EquipSlot::Jacket).map(|n| slot_color(EquipSlot::Jacket, &n)).unwrap_or("(none)".dimmed().to_string());
    let shirt_lbl = slot_name(ch, EquipSlot::Shirt).map(|n| slot_color(EquipSlot::Shirt, &n)).unwrap_or("(none)".dimmed().to_string());
    let pants_lbl = slot_name(ch, EquipSlot::Pants).map(|n| slot_color(EquipSlot::Pants, &n)).unwrap_or("(none)".dimmed().to_string());
    let shoes_lbl = slot_name(ch, EquipSlot::Shoes).map(|n| slot_color(EquipSlot::Shoes, &n)).unwrap_or("(none)".dimmed().to_string());
    let acc_lbl = slot_name(ch, EquipSlot::Accessory).map(|n| slot_color(EquipSlot::Accessory, &n)).unwrap_or("(none)".dimmed().to_string());

    // ── Health/Morale bars ──
    let hearts: String = "❤".repeat(ch.health as usize).red().to_string() + &"♡".repeat((ch.max_health - ch.health) as usize).bright_black().to_string();
    let morale_bar: String = "◆".repeat(ch.morale as usize).magenta().to_string() + &"◇".repeat((ch.max_morale - ch.morale) as usize).bright_black().to_string();

    // ── Compose ──
    let mut out = String::new();
    out.push_str(&format!("\n"));
    if !smoke.is_empty() { out.push_str(&format!("{}\n", smoke)); }
    out.push_str(&format!("{}   {} {}\n", hat_line, arrow, hat_lbl));
    out.push_str(&format!("{}   {} {}\n", eyes_line, arrow, acc_lbl));
    out.push_str(&format!("{}\n", mouth_line));
    out.push_str(&format!("{}\n", head_bot));
    out.push_str(&format!("          {}\n", "│".dimmed()));
    out.push_str(&format!("{}  {} {}\n", torso_top, arrow, jacket_lbl));
    out.push_str(&format!("{}  {} {}\n", torso1, arrow, shirt_lbl));
    out.push_str(&format!("{}\n", torso2));
    out.push_str(&format!("{}\n", torso_bot));
    out.push_str(&format!("          {}\n", "│".dimmed()));
    out.push_str(&format!("{}  {} {}\n", leg_top, arrow, pants_lbl));
    out.push_str(&format!("{}\n", leg1));
    out.push_str(&format!("{}\n", leg2));
    out.push_str(&format!("{}\n", leg_bot));
    out.push_str(&format!("{}  {} {}\n", shoes_top, arrow, shoes_lbl));
    out.push_str(&format!("{}\n", shoes_bot));
    out.push_str(&format!("\n      {}  {}\n", hearts, morale_bar));
    out.push_str(&format!("\n"));
    out
}

// ── Compact portrait (head only) ──────────────────────────────────────────────

pub fn render_portrait(ch: &Character) -> String {
    let face = face_for_character(ch);
    let cf = face.color_fn;
    let has_hat = ch.loadout.get(EquipSlot::Hat).is_some();

    let (tl, tr, bl, br, side) = if has_hat {
        (slot_fill(EquipSlot::Hat, "╔═══════╗"), String::new(),
         slot_fill(EquipSlot::Hat, "╚═══════╝"), String::new(),
         slot_fill(EquipSlot::Hat, "║"))
    } else {
        ("┌───────┐".to_string(), String::new(),
         "└───────┘".to_string(), String::new(),
         "│".to_string())
    };

    let mut out = String::new();
    if face.smoke {
        out.push_str(&format!("       {}\n", "~~~".bright_black()));
    }
    out.push_str(&format!("  {}\n", tl));
    out.push_str(&format!("  {}  {}  {}\n", side, cf(face.eyes), side));
    out.push_str(&format!("  {} {} {}\n", side, cf(face.mouth), side));
    out.push_str(&format!("  {}\n", bl));
    out
}
