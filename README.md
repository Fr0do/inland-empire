<p align="center">
  <img src="assets/logo.svg" width="200" alt="Inland Empire">
</p>

<h1 align="center">Inland Empire</h1>

<p align="center">
  Disco Elysium skill checks for Claude Code — your tools have voices now.
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/Fr0do/inland-empire?label=release" alt="Latest release">
  <img src="https://img.shields.io/badge/rust-1.70%2B-orange" alt="Rust">
  <img src="https://img.shields.io/github/license/Fr0do/inland-empire" alt="License">
</p>

---

Every time Claude Code reaches for a tool, **Inland Empire** rolls 2d6 + your skill modifier against a difficulty check. Critical success (⚅⚅) always passes. Critical failure (⚀⚀) always fails. Your character earns XP, levels up, and develops skills over time.

24 skills across 4 attributes — each mapped to a Claude Code domain:

| Attribute | Skills | Claude Code Domain |
|---|---|---|
| **Intellect** | Logic, Encyclopedia, Rhetoric, Drama, Conceptualization, Visual Calculus | Reasoning, API recall, architecture arguments, lie detection in logs |
| **Psyche** | Volition, Inland Empire, Empathy, Authority, Esprit, Suggestion | Scope discipline, code smells, user intent, standards enforcement |
| **Physique** | Endurance, Pain Threshold, Physical Instrument, Electrochemistry, Shivers, Half Light | Large codebases, legacy tolerance, brute-force, deploy thrill, system sense |
| **Motorics** | Hand/Eye, Perception, Reaction Speed, Savoir Faire, Interfacing, Composure | Surgical edits, bug detection, hotfixes, elegant code, API work, CI calm |

## Install

```bash
cargo install --git https://github.com/Fr0do/inland-empire
```

Or build from source:

```bash
git clone https://github.com/Fr0do/inland-empire
cd inland-empire
cargo install --path .
```

## Quick Start

```bash
# Create your character (archetypes: thinker, sensitive, bruiser, operator, generalist)
ie new "Harry" -a sensitive

# View character sheet
ie status

# Roll a skill check
ie check Bash -c "git push origin main"

# Level up a skill (when you have skill points)
ie develop logic

# Add a thought to the Thought Cabinet (persistent modifier)
ie think "Rust Evangelist" -d "Everything should be in Rust" -m "int:+2,log:+1"

# List all skills and archetypes
ie skills
ie archetypes
```

## Claude Code Hook Integration

Replace permission prompts with skill checks. Add to `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash|Edit|Write",
        "command": "ie hook-check \"$CLAUDE_TOOL_NAME\" -c \"$CLAUDE_TOOL_INPUT\""
      }
    ]
  }
}
```

Now every Bash, Edit, and Write action rolls a check. Failed checks block the action. The DE-style flavor text appears in your terminal:

```
INTERFACING [MOTORICS] — Formidable check (DC 14)
  3 + 2 +3 = 8  vs  14
  FAILURE
  git push --force origin main
  INTERFACING mutters: Not this time. working with external APIs and systems eludes you.
```

## Mechanics

- **2d6 + modifier vs DC** — standard Disco Elysium resolution
- **Critical success** (double 6) — always passes, regardless of DC
- **Critical failure** (double 1) — always fails, even on Trivial checks
- **Auto-difficulty** — tool + context keywords determine DC (e.g., `rm` → Formidable, `git status` → Easy)
- **Skill routing** — each Claude tool maps to the most relevant skill
- **XP** — 10 for success, 5 for failure. Level up every `level × 100` XP
- **Thought Cabinet** — internalize thoughts for persistent skill modifiers

## Difficulty Scale

| DC | Label | Example |
|---|---|---|
| 6 | Trivial | Read, Glob, Grep |
| 8 | Easy | Edit, safe git commands |
| 10 | Medium | Write, Bash, Agent |
| 12 | Challenging | git push, deploy |
| 14 | Formidable | rm, reset --hard, force push |
| 16 | Legendary | — |
| 18 | Heroic | — |
| 20 | Godly | — |

## License

MIT
