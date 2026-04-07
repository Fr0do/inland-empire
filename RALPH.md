# RALPH.md — Inland Empire autonomous tasks

Agent instructions: Read CLAUDE.md first. Build with `rtk cargo build`. Pick tasks in order. Mark `[x]` when done.

---

## [x] Add `ie history` command — paginated check history viewer

Show last N checks with filtering by skill/outcome.

```
ie history              # last 20 checks
ie history --skill Logic  # filter by skill
ie history --failed       # only failures
ie history -n 50          # last 50
```

Output format (one line per check):
```
2026-04-06 14:23  ✓ LOGIC        [4+2+3=9 vs DC8]   Read src/checks.rs
2026-04-06 14:21  ✗ HALF LIGHT   [1+3-1=3 vs DC14]  Bash rm -rf
```

Fields in `CheckRecord` (src/character.rs): skill, roll (u8,u8), modifier, total, difficulty, success, context, timestamp, check_color.

Add `History` variant to `Commands` in src/main.rs:
```rust
History {
    #[arg(short, long, default_value = "20")]
    n: usize,
    #[arg(short, long)]
    skill: Option<String>,
    #[arg(long)]
    failed: bool,
    #[arg(long)]
    passed: bool,
}
```

Use `colored` crate for output. Green ✓ / red ✗. Skill colored by attribute (blue=Intellect, magenta=Psyche, red=Physique, yellow=Motorics).

### Updates

---

## [x] Add morale/health bar to `ie status --oneline`

Currently: `Psychic L23 ❤🖤🖤🖤 ☀ 💭4/6 XP:545/2300 ★Inland Empire +1SP`

The `❤🖤🖤🖤` part shows health. Add morale separately with a different symbol (e.g. `🔵⚫⚫⚫` or `◆◇◇◇`).

Look at `cmd_status` and `display.rs` for the oneline format. The `character.morale` and `character.max_morale` fields exist. Add morale display after health in oneline output.

Also update the full `ie status` sheet to show morale with its own colored bar (use cyan/blue for morale vs red for health).

### Updates

---

## [x] Write tests for skill routing in src/skills.rs

The `Skill::for_tool(tool, context)` function maps tools+context to skills. It has no tests. Add a `#[cfg(test)]` module at the bottom of `src/skills.rs` with test cases:

- `for_tool("Bash", "git push origin main")` → `Electrochemistry`
- `for_tool("Bash", "git commit -m")` → `Rhetoric`
- `for_tool("Bash", "cargo test")` → `Logic`
- `for_tool("Bash", "rm -rf node_modules")` → `HalfLight`
- `for_tool("Read", "src/checks.rs")` → `Perception`
- `for_tool("Read", "config.toml")` → `VisualCalculus`
- `for_tool("Edit", "src/main_test.rs")` → `Logic`
- `for_tool("Glob", "**/*.rs")` → `VisualCalculus` (or whatever `for_tool("Glob",...)` returns)
- `for_tool("Bash", "docker-compose up")` → `Interfacing`
- `for_tool("Bash", "echo hello")` → `Drama`

Run with `rtk cargo test`. All tests must pass.

### Updates
- Added the missing `Glob` routing assertion and verified the requested cases match current routing behavior.

---

## [x] Export/Import characters — `ie export` / `ie import`

New module `src/multiplayer.rs`. Export active character to `{name}.ie.json`, import from JSON.

```rust
pub struct PortableCharacter { version: u32, exported_at: DateTime<Utc>, character: Character }
pub fn export_character(ch: &Character) -> Result<String, String>
pub fn import_character(json: &str) -> Result<Character, String>
fn resolve_name_conflict(name: &str, existing: &[String]) -> String  // name-2, name-3...
```

Add to `src/character.rs`: extract `pub fn migrate(&mut self)` so imported chars run the same migrations as disk-loaded ones.

Add to `src/main.rs`:
```rust
Export { name: Option<String>, output: Option<String> }
Import { file: String }
```

Build with `rtk cargo build`. No new deps needed (serde_json already present).

### Updates
- `multiplayer::import_character` now only deserializes; `cmd_import` applies `Character::migrate()` after resolving name conflicts.

---

## [x] Compare characters — `ie compare`

In `src/multiplayer.rs`:
```rust
pub struct Comparison { left: CharacterSummary, right: CharacterSummary, verdicts: Vec<Verdict> }
pub struct CharacterSummary { name, archetype, level, total_xp, pass_rate, best_streak, top_skills, attributes }
pub struct Verdict { category: String, left_val: String, right_val: String, winner: Winner }
pub enum Winner { Left, Right, Tie }
pub fn compare(left: &Character, right: &Character) -> Comparison
pub fn format_comparison(comp: &Comparison) -> String  // colored terminal output
```

DE-style: add random flavor text from attribute voices (AUTHORITY, INLAND EMPIRE, etc.) to the comparison output.

Add to `src/main.rs`:
```rust
Compare { name1: String, name2: Option<String> }  // name2=None → compare with active
```

Output: side-by-side terminal table, winner highlighted green.

### Updates

---

## [x] SVG character card — `ie card`

In `src/multiplayer.rs`:
```rust
pub fn generate_card(ch: &Character) -> String  // standalone SVG 600×400
```

Make `svg_radar_chart` and `svg_portrait` in `src/dashboard.rs` `pub(crate)`. Reuse them in the card.
Layout: portrait left, radar chart right, key stats (level, pass rate, streak, top skill) in center strip.
Save to `{name}-card.svg`.

Add to `src/main.rs`:
```rust
Card { name: Option<String>, output: Option<String> }
```

### Updates
- Implemented `generate_card` in `multiplayer.rs` (standalone 600x400 SVG).
- Added `Card` subcommand to `main.rs` with short/long flags.
- Reused existing portrait and radar chart SVG generators.


---

## [x] Dashboard leaderboard — `/leaderboard` route

In `src/dashboard.rs`:
1. Add "Leaderboard" link to nav
2. New route: `/leaderboard` → `page_leaderboard()`
3. `page_leaderboard()`: load all profiles via `list_profiles()` + `Character::load`, compute stats for each, render HTML table sorted by level (desc) with columns: rank, name, archetype, level, pass rate, checks, best streak, XP

No new deps needed.

### Updates

---

## [x] Improve `ie stats` output — add skill breakdown table

Currently `ie stats` shows aggregate pass rate and top skills. Add a full skill breakdown table:

```
SKILL BREAKDOWN (sorted by usage)
──────────────────────────────────────────────────────
Skill              Used   Pass%   Avg Roll   Last Used
──────────────────────────────────────────────────────
Interfacing         847   84%     9.2        2h ago
Logic               234   71%     8.8        1d ago
Perception          198   89%     10.1       3d ago  ⚠ atrophy risk
...
```

Show "⚠ atrophy risk" if `skill_last_used` entry is 5+ days old (warning before the 7-day threshold).

Data comes from `character.check_history` (Vec<CheckRecord>) and `character.skill_last_used`. Look at `src/stats.rs` for existing stats logic.

### Updates
