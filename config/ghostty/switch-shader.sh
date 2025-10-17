#!/usr/bin/env bash
# Ghostty Shader Switcher
# Interactive shader selection with hot-reload

SHADER_DIR="$HOME/.config/ghostty/shaders"
CONFIG_FILE="$HOME/.config/ghostty/config"
CURRENT_SHADER="$SHADER_DIR/current.glsl"

# Get list of shader files (exclude README, git files, etc.)
SHADERS=$(find "$SHADER_DIR" -maxdepth 1 -name "*.glsl" -type f | sort)

if [ -z "$SHADERS" ]; then
    echo "No shaders found in $SHADER_DIR"
    exit 1
fi

# Check if fzf is available
if ! command -v fzf &> /dev/null; then
    echo "Error: fzf is not installed. Install with: brew install fzf"
    exit 1
fi

# Get currently active shader (if any)
CURRENT_NAME=""
if [ -L "$CURRENT_SHADER" ] && [ -e "$CURRENT_SHADER" ]; then
    CURRENT_NAME=$(basename "$(readlink "$CURRENT_SHADER")" .glsl)
elif ! grep -q "^custom-shader" "$CONFIG_FILE"; then
    CURRENT_NAME="none (disable shader)"
fi

# Format shader names nicely for fzf, mark the current one
SHADER_NAMES=$(echo "$SHADERS" | xargs -n1 basename | sed 's/\.glsl$//')
if [ -n "$CURRENT_NAME" ] && [ "$CURRENT_NAME" != "none (disable shader)" ]; then
    SHADER_NAMES=$(echo "$SHADER_NAMES" | sed "s/^${CURRENT_NAME}$/${CURRENT_NAME} ★/")
fi

# Add "none (disable shader)" option
NONE_OPTION="none (disable shader)"
if [ "$CURRENT_NAME" = "none (disable shader)" ]; then
    NONE_OPTION="none (disable shader) ★"
fi

SELECTION=$(echo -e "$NONE_OPTION\n$SHADER_NAMES" | fzf \
    --prompt="Select shader: " \
    --height=40% \
    --reverse \
    --border \
    --preview="SHADER=\$(echo '{}' | sed 's/ ★$//'); [ \"\$SHADER\" != 'none (disable shader)' ] && cat $SHADER_DIR/\$SHADER.glsl | head -20 || echo 'Disables custom shader'" \
    --preview-window=right:60%:wrap)

# Remove the star marker from selection
SELECTION=$(echo "$SELECTION" | sed 's/ ★$//')

if [ -z "$SELECTION" ]; then
    echo "No shader selected"
    exit 0
fi

# Update or add custom-shader line in config
if [ "$SELECTION" = "none (disable shader)" ]; then
    # Comment out or remove the custom-shader line
    if grep -q "^custom-shader" "$CONFIG_FILE"; then
        sed -i '' 's/^custom-shader/# custom-shader/' "$CONFIG_FILE"
        echo "Shader disabled"
    fi
    rm -f "$CURRENT_SHADER"
else
    SELECTED_SHADER="$SHADER_DIR/${SELECTION}.glsl"

    # Create symlink to current shader for reference
    ln -sf "$SELECTED_SHADER" "$CURRENT_SHADER"

    # Update config file (using BSD sed syntax for macOS)
    if grep -q "^custom-shader" "$CONFIG_FILE"; then
        # Replace existing uncommented line
        sed -i '' "s|^custom-shader.*|custom-shader = $CURRENT_SHADER|" "$CONFIG_FILE"
    elif grep -q "^# custom-shader" "$CONFIG_FILE"; then
        # Replace existing commented line
        sed -i '' "s|^# custom-shader.*|custom-shader = $CURRENT_SHADER|" "$CONFIG_FILE"
    else
        # Add new line
        echo "" >> "$CONFIG_FILE"
        echo "# --- Custom Shader ---" >> "$CONFIG_FILE"
        echo "custom-shader = $CURRENT_SHADER" >> "$CONFIG_FILE"
    fi

    # Verify the change was applied
    if grep -q "^custom-shader = $CURRENT_SHADER" "$CONFIG_FILE"; then
        echo "Shader switched to: $SELECTION"
    else
        echo "ERROR: Failed to update config file. Line may still be commented."
        exit 1
    fi
fi

# Display success message
echo ""
echo "✓ Shader updated! Press Cmd+Shift+R to reload Ghostty."
