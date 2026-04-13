#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

echo "Building ApplyKit release bundle for macOS..."
echo "This script builds unsigned release artifacts by default."
echo "Signing/notarization remain credential-dependent follow-up steps."

pnpm -C ui install --frozen-lockfile
pnpm -C ui build
cargo tauri build --bundles app,dmg "$@"

echo
echo "Release artifacts:"
find "$repo_root/target/release/bundle" -maxdepth 2 \( -name "*.app" -o -name "*.dmg" \) -print | sort
