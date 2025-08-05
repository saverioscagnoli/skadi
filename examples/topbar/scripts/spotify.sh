#!/bin/bash

player="spotify"

playerctl --player=$player metadata --format '{{ artist }};{{ title }}' --follow | while IFS=';;;' read -r artist title; do
    status=$(playerctl --player=$player status 2>/dev/null)
    if [ "$status" = "Playing" ]; then
        echo "{\"artist\": \"$artist\", \"title\": \"$title\"}"
    else
        echo "null"
    fi
done
