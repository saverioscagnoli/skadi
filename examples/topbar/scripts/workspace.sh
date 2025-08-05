#!/bin/bash

# Function to get detailed workspace info
get_workspace_info() {
    local workspace_id="$1"
    hyprctl workspaces -j | jq -r ".[] | select(.id == $workspace_id) | .name, .windows"
}

# Track last workspace to avoid duplicates
last_workspace=""

# Main event loop
socat -U - "UNIX-CONNECT:$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock" | while read -r event; do
    if [[ "$event" == workspace* ]]; then
        # Extract workspace ID and clean it up
        workspace_id=$(echo "$event" | cut -d',' -f2 | sed 's/workspace>>//')
        
        # Only output if workspace changed
        if [[ "$workspace_id" != "$last_workspace" ]]; then
            # Get total workspaces count
            total_workspaces=$(hyprctl workspaces -j | jq '. | length')
            
            # Output JSON format
            echo "{\"active\": $workspace_id, \"total\": $total_workspaces}"
            
            last_workspace="$workspace_id"
        fi
    fi
done