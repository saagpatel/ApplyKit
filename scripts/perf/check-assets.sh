#!/usr/bin/env bash
set -euo pipefail

# codex-os-managed
max_bytes="${ASSET_MAX_BYTES:-350000}"

asset_roots=()
if [[ -d "public" ]]; then
  asset_roots+=("public")
fi
if [[ -d "ui/public" ]]; then
  asset_roots+=("ui/public")
fi
if [[ -d "dist/assets" ]]; then
  asset_roots+=("dist/assets")
fi
if [[ -d "ui/dist/assets" ]]; then
  asset_roots+=("ui/dist/assets")
fi
if [[ ${#asset_roots[@]} -eq 0 ]]; then
  echo "No public or built asset directories found; skipping asset check."
  exit 0
fi

fail=0
for root in "${asset_roots[@]}"; do
  while IFS= read -r file; do
    size=$(wc -c < "$file")
    if (( size > max_bytes )); then
      echo "Asset too large (>${max_bytes} bytes): $file"
      fail=1
    fi
  done < <(find "$root" -type f \( -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" -o -name "*.webp" -o -name "*.avif" -o -name "*.svg" -o -name "*.woff2" -o -name "*.ttf" \))
done

exit $fail
