#!/bin/bash
TOOL_NAME="${CLAUDE_TOOL_NAME:-Bash}"
TOOL_INPUT="${CLAUDE_TOOL_INPUT:-}"

# Run ie hook-check, capture stdout (JSON) and stderr (DE flavor)
JSON_OUT=$(ie hook-check "$TOOL_NAME" -c "$TOOL_INPUT" 2>/dev/null)

# Parse all scalar fields in two jq calls: TSV for numbers/flags, raw for free-text reason
read -r ALLOW SKILL ROLL MODIFIER TOTAL THRESHOLD CHECK_COLOR <<< $(echo "$JSON_OUT" | jq -r '[.allow, .skill, ((.roll[0] // 0)+(.roll[1] // 0)), .modifier, .total, .threshold, (.check_color // "White")] | @tsv')
REASON=$(echo "$JSON_OUT" | jq -r '.reason')

# Build display
if [ "$ALLOW" = "true" ]; then
    DECISION="allow"
    DISPLAY="✓ ${SKILL} [${ROLL}+${MODIFIER}=${TOTAL} vs DC${THRESHOLD}] ${CHECK_COLOR}"
else
    DECISION="deny"
    DISPLAY="✗ ${SKILL} [${ROLL}+${MODIFIER}=${TOTAL} vs DC${THRESHOLD}] ${CHECK_COLOR}"
fi

# Output hookSpecificOutput JSON to stdout for Claude Code
jq -n \
  --arg d "$DECISION" \
  --arg r "$DISPLAY" \
  --arg c "$REASON" \
  '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:$d,permissionDecisionReason:$r,additionalContext:$c}}'

exit 0
