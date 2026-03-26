---
name: disco
description: Toggle Disco Elysium mode — replace permission prompts with 2d6 skill checks
user_invocable: true
---

# Disco Elysium Mode

Toggle the Disco Elysium skill check system for Claude Code.

## What to do

When the user invokes `/disco`, perform the following:

1. **Check if disco-claude binary exists** by running `./target/release/disco-claude --version` (or `./target/debug/disco-claude --version`). If not found, suggest running `cargo build --release`.

2. **If no arguments**: show current character status by running `./target/debug/disco-claude status`.

3. **If argument is `new <name> [-a archetype]`**: create a new character by running `./target/debug/disco-claude new "<name>" -a <archetype>`.

4. **If argument is `check <tool> [-c context]`**: perform a skill check by running `./target/debug/disco-claude check <tool> -c "<context>"`. Show the result in DE-style formatting.

5. **If argument is `develop <skill>`**: spend a skill point by running `./target/debug/disco-claude develop <skill>`.

6. **If argument is `think <name> [-d desc] [-m modifiers]`**: internalize a thought by running `./target/debug/disco-claude think "<name>" -d "<desc>" -m "<modifiers>"`.

7. **If argument is `roll <tool> <context>`**: perform a hook-style check by running `./target/debug/disco-claude hook-check <tool> -c "<context>"` and parse the JSON output. If `allow` is true, proceed with the action. If false, tell the user the check failed and they cannot perform the action.

8. **If argument is `archetypes`**: list archetypes via `./target/debug/disco-claude archetypes`.

9. **If argument is `skills`**: list all skills via `./target/debug/disco-claude skills`.

## Disco Elysium Flavor

When reporting check results, adopt the voice of the relevant skill as an inner monologue, just like in Disco Elysium. Each skill is a voice in the detective's head:

- **Logic**: cold, analytical, sometimes condescending
- **Encyclopedia**: pedantic, trivia-obsessed, helpful but tangential
- **Rhetoric**: persuasive, debate-focused, sees arguments everywhere
- **Drama**: theatrical, paranoid about deception, over-the-top
- **Volition**: the voice of reason, encouraging, tries to keep you grounded
- **Inland Empire**: mystical, dream-like, sees meaning in everything
- **Empathy**: warm, understanding, sometimes too sensitive
- **Authority**: commanding, domineering, demands respect
- **Electrochemistry**: thrill-seeking, reckless, loves danger
- **Half Light**: paranoid, aggressive, fight-or-flight
- **Interfacing**: technical, precise, loves gadgets and systems
- **Perception**: observant, detail-oriented, misses nothing
- **Composure**: zen-like, unflappable, always cool

Format the inner monologue as:

```
SKILL_NAME [ATTRIBUTE] — "The inner monologue text here."
```

## Integration as Hook

To use as a Claude Code pre-tool hook that replaces permission prompts with skill checks, the user should configure their `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash|Edit|Write",
        "command": "./target/release/disco-claude hook-check \"$CLAUDE_TOOL_NAME\" -c \"$CLAUDE_TOOL_INPUT\""
      }
    ]
  }
}
```

The hook outputs JSON with `{"allow": true/false, ...}` to stdout.
