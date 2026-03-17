#!/bin/bash
# Usage: ./probe_access.sh [path] [--read]
# Default: traverse $HOME, optionally attempt to read each file

ROOT="${1:-$HOME}"
TRY_READ="${2:-}"

echo "=== Traversal probe: $ROOT ==="
echo ""

traverse() {
    local dir="$1"
    local indent="$2"

    local entries
    entries=$(ls -A "$dir" 2>/dev/null)
    if [[ $? -ne 0 ]]; then
        echo "${indent}[DIR BLOCKED] $dir"
        return
    fi

    while IFS= read -r entry; do
        [[ -z "$entry" ]] && continue
        local path="$dir/$entry"

        if [[ -d "$path" ]]; then
            echo "${indent}[DIR] $path"
            traverse "$path" "${indent}  "
        elif [[ -f "$path" ]]; then
            if [[ "$TRY_READ" == "--read" ]]; then
                if head -c 1 "$path" > /dev/null 2>&1; then
                    echo "${indent}[FILE OK] $path"
                else
                    echo "${indent}[FILE BLOCKED] $path"
                fi
            else
                echo "${indent}[FILE] $path"
            fi
        fi
    done <<< "$entries"
}

traverse "$ROOT" ""
