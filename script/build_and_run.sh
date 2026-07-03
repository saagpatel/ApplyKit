#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
APP_NAME="ApplyKit"
BUNDLE_ID="com.applykit.desktop"
PROCESS_NAME="applykit_tauri"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BUNDLE="$ROOT_DIR/target/debug/bundle/macos/$APP_NAME.app"
APP_BINARY="$ROOT_DIR/target/debug/applykit_tauri"

usage() {
  cat <<'EOF'
Usage: ./script/build_and_run.sh [run|--verify|--logs|--telemetry|--debug|--stop]

Builds the debug Tauri macOS app bundle, stops an existing ApplyKit process,
and launches the freshly built app. Use --verify for non-interactive smoke.
EOF
}

stop_app() {
  pkill -x "$APP_NAME" >/dev/null 2>&1 || true
  pkill -x "$PROCESS_NAME" >/dev/null 2>&1 || true
}

build_bundle() {
  cargo tauri build --debug --bundles app
}

open_app() {
  /usr/bin/open -n "$APP_BUNDLE"
}

verify_app() {
  for _ in $(seq 1 20); do
    if pgrep -x "$PROCESS_NAME" >/dev/null 2>&1; then
      echo "ok: $APP_NAME launched as $PROCESS_NAME"
      return 0
    fi
    sleep 0.25
  done

  echo "error: $APP_NAME did not launch" >&2
  return 1
}

case "$MODE" in
  run)
    stop_app
    build_bundle
    open_app
    ;;
  --verify|verify)
    stop_app
    build_bundle
    open_app
    verify_app
    ;;
  --logs|logs)
    stop_app
    build_bundle
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$PROCESS_NAME\""
    ;;
  --telemetry|telemetry)
    stop_app
    build_bundle
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$PROCESS_NAME\" OR subsystem == \"$BUNDLE_ID\""
    ;;
  --debug|debug)
    stop_app
    cargo build -p applykit_tauri
    lldb -- "$APP_BINARY"
    ;;
  --stop|stop)
    stop_app
    ;;
  --help|-h|help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
