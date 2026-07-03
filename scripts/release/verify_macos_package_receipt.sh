#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
receipt_path=""

usage() {
  cat <<'EOF'
Usage: ./scripts/release/verify_macos_package_receipt.sh [--receipt PATH]

Builds unsigned macOS release artifacts and writes a compact JSON receipt.

Checks:
- clean git worktree before writing release evidence
- unsigned macOS release build via ./scripts/release/build_macos_release.sh
- expected .app bundle and .dmg package exist and are non-empty
- app executable exists inside the release bundle

By default, the receipt is written under /tmp.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --receipt)
      if [[ $# -lt 2 ]]; then
        echo "error: --receipt requires a path" >&2
        exit 2
      fi
      receipt_path="$2"
      shift 2
      ;;
    --help|-h|help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
done

cd "$repo_root"

dirty_status="$(git status --porcelain --untracked-files=normal)"
if [[ -n "$dirty_status" ]]; then
  echo "error: macOS package receipt requires a clean git worktree before writing evidence" >&2
  echo "$dirty_status" >&2
  exit 1
fi

run_id="$(date -u +%Y%m%dT%H%M%SZ)"
receipt_path="${receipt_path:-${TMPDIR:-/tmp}/applykit-macos-package-${run_id}.json}"
build_log="${TMPDIR:-/tmp}/applykit-macos-package-${run_id}.log"
bundle_root="$repo_root/target/release/bundle"
app_binary="$bundle_root/macos/ApplyKit.app/Contents/MacOS/applykit_tauri"

step() {
  local label="$1"
  shift
  printf '==> %s\n' "$label"
  "$@"
}

mkdir -p "$(dirname "$receipt_path")"

step "build unsigned macOS release package" ./scripts/release/build_macos_release.sh | tee "$build_log"

app_bundles=()
while IFS= read -r artifact; do
  app_bundles+=("$artifact")
done < <(find "$bundle_root" -maxdepth 3 -type d -name "*.app" -print | sort)

dmg_packages=()
while IFS= read -r artifact; do
  dmg_packages+=("$artifact")
done < <(find "$bundle_root" -maxdepth 3 -type f -name "*.dmg" -print | sort)

if [[ "${#app_bundles[@]}" -eq 0 ]]; then
  echo "error: no release .app bundle found under $bundle_root" >&2
  exit 1
fi

if [[ "${#dmg_packages[@]}" -eq 0 ]]; then
  echo "error: no release .dmg package found under $bundle_root" >&2
  exit 1
fi

if [[ ! -x "$app_binary" ]]; then
  echo "error: expected app executable missing or not executable: $app_binary" >&2
  exit 1
fi

for dmg in "${dmg_packages[@]}"; do
  if [[ ! -s "$dmg" ]]; then
    echo "error: release .dmg package is empty: $dmg" >&2
    exit 1
  fi
done

git_head="$(git rev-parse HEAD)"

node - "$receipt_path" "$repo_root" "$git_head" "$build_log" "$app_binary" "${app_bundles[@]}" "--" "${dmg_packages[@]}" <<'NODE'
const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const args = process.argv.slice(2);
const separator = args.indexOf("--");
const [receiptPath, repoRoot, gitHead, buildLog, appBinary, ...appBundles] =
  args.slice(0, separator);
const dmgPackages = args.slice(separator + 1);

const fileHash = (filePath) =>
  crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");

const fileReceipt = (filePath) => ({
  path: filePath,
  bytes: fs.statSync(filePath).size,
  sha256: fileHash(filePath),
});

const dirReceipt = (dirPath) => ({
  path: dirPath,
  entries: fs.readdirSync(dirPath).length,
});

const receipt = {
  schema: "applykit-macos-package/v1",
  generatedAt: new Date().toISOString(),
  repoRoot,
  gitHead,
  signing: {
    status: "unsigned",
    notarization: "not-run",
    reason: "local package receipt does not require Apple signing credentials",
  },
  checks: [
    {
      name: "clean-worktree",
      command: "git status --porcelain --untracked-files=normal",
      status: "pass",
    },
    {
      name: "unsigned-macos-package-build",
      command: "./scripts/release/build_macos_release.sh",
      status: "pass",
    },
    {
      name: "release-artifacts-present",
      command: "find target/release/bundle -maxdepth 3",
      status: "pass",
    },
  ],
  buildLog,
  app: {
    bundles: appBundles.map(dirReceipt),
    executable: fileReceipt(appBinary),
  },
  dmg: {
    packages: dmgPackages.map(fileReceipt),
  },
};

fs.writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
NODE

printf 'ok: macOS package receipt written to %s\n' "$receipt_path"
