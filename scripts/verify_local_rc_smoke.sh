#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
receipt_path=""
keep_app=false

usage() {
  cat <<'EOF'
Usage: ./scripts/verify_local_rc_smoke.sh [--receipt PATH] [--keep-app]

Runs the local release-candidate smoke path and writes a compact JSON receipt.

Checks:
- frozen pnpm install from the repo root
- deterministic CLI packet generation from fixtures/jd_support_ops_01.txt
- expected packet artifacts, including ReviewData.json
- native debug Tauri app bundle launch via ./script/build_and_run.sh --verify

By default, evidence is written under /tmp and the launched app is stopped.
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
    --keep-app)
      keep_app=true
      shift
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
  echo "error: local RC smoke requires a clean git worktree before writing evidence" >&2
  echo "$dirty_status" >&2
  exit 1
fi

run_id="$(date -u +%Y%m%dT%H%M%SZ)"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/applykit-rc-smoke.XXXXXX")"
receipt_path="${receipt_path:-${TMPDIR:-/tmp}/applykit-local-rc-smoke-${run_id}.json}"
packet_base="$tmp_root/packets"
jd_path="$tmp_root/jd_support_ops_01.txt"
cli_log="$tmp_root/cli-generate.log"
app_bundle="$repo_root/target/debug/bundle/macos/ApplyKit.app"

cleanup() {
  if [[ "$keep_app" != true ]]; then
    ./script/build_and_run.sh --stop >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

step() {
  local label="$1"
  shift
  printf '==> %s\n' "$label"
  "$@"
}

required_packet_files=(
  "JD.txt"
  "Extracted.json"
  "FitScore.md"
  "TailorPlan.md"
  "Resume_1pg_Tailored.md"
  "RecruiterMessage.md"
  "HiringManagerMessage.md"
  "CoverNote_Short.md"
  "Diff.md"
  "TrackerRow.csv"
  "Meta.json"
  "ReviewData.json"
)

cp "$repo_root/fixtures/jd_support_ops_01.txt" "$jd_path"
mkdir -p "$packet_base" "$(dirname "$receipt_path")"

step "frozen install" pnpm -C . install --frozen-lockfile

step "generate deterministic packet" cargo run -p applykit_cli -- generate \
  --company "Acme" \
  --role "Support Operations Lead" \
  --source "LinkedIn" \
  --baseline 1pg \
  --date 2026-02-14 \
  --jd "$jd_path" \
  --outdir "$packet_base" | tee "$cli_log"

packet_dir="$(awk -F'Output Dir: ' '/^Output Dir: / {print $2}' "$cli_log" | tail -n1)"
if [[ -z "$packet_dir" || ! -d "$packet_dir" ]]; then
  echo "error: generated packet directory was not found" >&2
  exit 1
fi

for file in "${required_packet_files[@]}"; do
  if [[ ! -s "$packet_dir/$file" ]]; then
    echo "error: missing or empty packet artifact: $file" >&2
    exit 1
  fi
done

step "verify native debug launch" ./script/build_and_run.sh --verify

if [[ ! -d "$app_bundle" ]]; then
  echo "error: expected app bundle missing: $app_bundle" >&2
  exit 1
fi

git_head="$(git rev-parse HEAD)"

node - "$receipt_path" "$repo_root" "$git_head" "$packet_dir" "$app_bundle" "${required_packet_files[@]}" <<'NODE'
const fs = require("node:fs");
const path = require("node:path");

const [
  receiptPath,
  repoRoot,
  gitHead,
  packetDir,
  appBundle,
  ...packetFiles
] = process.argv.slice(2);

const statSize = (fileName) => fs.statSync(path.join(packetDir, fileName)).size;
const receipt = {
  schema: "applykit-local-rc-smoke/v1",
  generatedAt: new Date().toISOString(),
  repoRoot,
  gitHead,
  checks: [
    {
      name: "frozen-install",
      command: "pnpm -C . install --frozen-lockfile",
      status: "pass",
    },
    {
      name: "deterministic-packet-generation",
      command: "cargo run -p applykit_cli -- generate --date 2026-02-14",
      status: "pass",
    },
    {
      name: "native-debug-launch",
      command: "./script/build_and_run.sh --verify",
      status: "pass",
    },
  ],
  packet: {
    sourceFixture: "fixtures/jd_support_ops_01.txt",
    runDate: "2026-02-14",
    outputDir: packetDir,
    files: packetFiles.map((name) => ({
      name,
      bytes: statSize(name),
    })),
  },
  nativeApp: {
    bundle: appBundle,
    processName: "applykit_tauri",
    status: "launched",
  },
};

fs.writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
NODE

printf 'ok: local RC smoke receipt written to %s\n' "$receipt_path"
