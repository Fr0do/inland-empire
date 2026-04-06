# RALPH.md — Inland Empire autonomous tasks

Agent instructions: Read CLAUDE.md first. Build with `rtk cargo build`. Pick tasks in order. Mark `[x]` when done.

---

## [ ] Add `ie history` command — paginated check history viewer

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

## [ ] Add morale/health bar to `ie status --oneline`

Currently: `Psychic L23 ❤🖤🖤🖤 ☀ 💭4/6 XP:545/2300 ★Inland Empire +1SP`

The `❤🖤🖤🖤` part shows health. Add morale separately with a different symbol (e.g. `🔵⚫⚫⚫` or `◆◇◇◇`).

Look at `cmd_status` and `display.rs` for the oneline format. The `character.morale` and `character.max_morale` fields exist. Add morale display after health in oneline output.

Also update the full `ie status` sheet to show morale with its own colored bar (use cyan/blue for morale vs red for health).

### Updates

---

## [ ] Write tests for skill routing in src/skills.rs

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

---

## [ ] Improve `ie stats` output — add skill breakdown table

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
