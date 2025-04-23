#!/usr/bin/env bash

dependencies=("mk" "fzf" "bat")

for prog in "${dependencies[@]}"; do
    if ! command -v "$prog" &> /dev/null; then
        echo "mk, fzf, and bat are necessary to use this script. Install them first."
        exit 1
    fi
done

subcommand="${1:-list}"

case "$subcommand" in
    evoke|list|delete);;
    *)
        echo "Invalid subcommand ${subcommand}"
        exit 1
        ;;
esac

selection=$(mk list --type plain \
    | fzf --layout reverse-list --height=40% --border=rounded \
    --preview 'mk list {} | bat --style=numbers --color=always')

if [[ -z "$selection" ]]; then
    exit 0
fi

mk "$subcommand" "$selection"
