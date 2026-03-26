# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What Is This

**Inland Empire** — Rust CLI that replaces Claude Code permission prompts with Disco Elysium-style 2d6 skill checks. Binary name: `ie`.

## Build & Run

```bash
cargo build --release
cargo install --path .
ie new "Name" -a thinker
ie status
ie check Bash -c "git push"
ie hook-check Bash -c "rm -rf"   # JSON output for hooks
```

## Architecture

```
src/
  main.rs       — CLI (clap) with all subcommands
  skills.rs     — 24 skills, 4 attributes, tool→skill mapping, claude_domain descriptions
  character.rs  — Character struct, archetypes, save/load to ~/Library/Application Support/inland-empire/
  checks.rs     — 2d6 roll engine, difficulty classification, check recording
  display.rs    — DE-style terminal rendering (character sheet, status line)
```

**Data flow**: CLI → load active character → perform check/mutation → save. Hook mode (`hook-check`) outputs JSON to stdout + DE flavor to stderr.

## Key Mechanics

- **2d6 + modifier vs DC**: double-6 = critical success, double-1 = critical failure
- **Difficulty auto-classification**: `Difficulty::for_action(tool, context)` maps tool + keywords to DC 6–20
- **Skill routing**: `Skill::for_tool(tool)` maps Claude tools to skills
- **Thought Cabinet**: persistent skill modifiers via `think` command, format: `"skill:+N,skill:-N"`
