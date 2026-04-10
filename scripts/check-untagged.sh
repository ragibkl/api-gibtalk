#!/usr/bin/env bash

# Check for media files that are not listed in any symbols.yaml

MEDIA_DIR="./media"
untagged=0
total=0

for library in "$MEDIA_DIR"/*/; do
    library_name=$(basename "$library")
    yaml_file="$library/symbols.yaml"

    # Find all png files in this library
    while IFS= read -r png; do
        total=$((total + 1))
        relative="${png#$library}"

        if [ ! -f "$yaml_file" ]; then
            echo "[untagged] $library_name/$relative (no symbols.yaml)"
            untagged=$((untagged + 1))
        elif ! grep -q "$relative" "$yaml_file"; then
            echo "[untagged] $library_name/$relative"
            untagged=$((untagged + 1))
        fi
    done < <(find "$library" -name "*.png" -type f | sort)
done

echo ""
echo "Total: $total files, $untagged untagged"
