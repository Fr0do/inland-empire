#!/bin/bash
set -e

echo "═══════════════════════════════════════"
echo "  INLAND EMPIRE — Installation"
echo "═══════════════════════════════════════"
echo ""

# Build and install ie binary
echo "Building ie binary..."
cargo install --path .
echo "✓ ie binary installed"

# Install hook
HOOK_DIR="$HOME/.claude/hooks"
mkdir -p "$HOOK_DIR"
cp hooks/ie-check.sh "$HOOK_DIR/ie-check.sh"
chmod +x "$HOOK_DIR/ie-check.sh"
echo "✓ Hook installed to $HOOK_DIR/ie-check.sh"

# Check if settings already configured
SETTINGS="$HOME/.claude/settings.json"
if [ -f "$SETTINGS" ] && grep -q "ie-check" "$SETTINGS"; then
    echo "✓ Claude Code settings already configured"
else
    echo ""
    echo "Add this to $SETTINGS (or merge with existing):"
    echo ""
    cat hooks/settings.example.json
    echo ""
fi

echo ""
echo "═══════════════════════════════════════"
echo "  Done! Create a character:"
echo "  ie new <name> -a <archetype>"
echo ""
echo "  Archetypes: thinker, sensitive,"
echo "  bruiser, operator, generalist"
echo "═══════════════════════════════════════"
