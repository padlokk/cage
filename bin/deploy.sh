#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BINARY_NAME="cage"
PROFILE="release"
PREFIX=""
LIB_DIR=""
BIN_DIR=""
SKIP_BUILD=0

usage() {
    cat <<'USAGE'
Usage: ./bin/deploy.sh [options]

Options:
  --prefix <PATH>      Install under the given prefix (default: ~/.local)
  --lib-dir <PATH>     Override library install directory (default: <prefix>/lib/odx/cage)
  --bin-dir <PATH>     Override binary symlink directory (default: <prefix>/bin)
  --profile <NAME>     Build profile to deploy (release|debug, default: release)
  --skip-build         Reuse existing build artifact (skip cargo build)
  --help               Show this help message

Examples:
  ./bin/deploy.sh                         # Build release and install into ~/.local
  ./bin/deploy.sh --prefix /usr/local     # Install into /usr/local/{bin,lib/odx/cage}
  ./bin/deploy.sh --profile debug         # Deploy debug build (target/debug/cage)
USAGE
}

has_boxy() {
    command -v boxy >/dev/null 2>&1
}

ceremony_msg() {
    local msg="$1"
    local theme="${2:-info}"
    if has_boxy; then
        echo "$msg" | boxy --theme "$theme" --width max
    else
        echo "$msg"
    fi
}

step_msg() {
    local step="$1"
    local desc="$2"
    if has_boxy; then
        printf "%s\n%s\n" "$step" "• $desc" | boxy --style rounded --width max --title "📦 Deploy Step"
    else
        echo "$step: $desc"
    fi
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --lib-dir)
            LIB_DIR="$2"
            shift 2
            ;;
        --lib-dir=*)
            LIB_DIR="${1#*=}"
            shift
            ;;
        --bin-dir)
            BIN_DIR="$2"
            shift 2
            ;;
        --bin-dir=*)
            BIN_DIR="${1#*=}"
            shift
            ;;
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --profile=*)
            PROFILE="${1#*=}"
            shift
            ;;
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
done

case "$PROFILE" in
    release|debug) ;;
    *)
        echo "Invalid profile '$PROFILE'. Use 'release' or 'debug'." >&2
        exit 1
        ;;
esac

if [[ -z "$PREFIX" ]]; then
    PREFIX="$HOME/.local"
fi

if [[ -z "$LIB_DIR" ]]; then
    LIB_DIR="$PREFIX/lib/odx/cage"
fi

if [[ -z "$BIN_DIR" ]]; then
    BIN_DIR="$PREFIX/bin"
fi

VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)
ARTIFACT="$ROOT_DIR/target/$PROFILE/${BINARY_NAME}"

ceremony_msg "🔒 CAGE DEPLOYMENT CEREMONY v$VERSION" "success"
echo

step_msg "Step 1" "Preparing build artifact (profile: $PROFILE)"
cd "$ROOT_DIR"
if [[ $SKIP_BUILD -eq 0 ]]; then
    if [[ "$PROFILE" == "release" ]]; then
        cargo build --release --bin "$BINARY_NAME"
    else
        cargo build --bin "$BINARY_NAME"
    fi
fi

if [[ ! -f "$ARTIFACT" ]]; then
    ceremony_msg "❌ Binary not found at $ARTIFACT" "error"
    exit 1
fi

if [[ ! -s "$ARTIFACT" ]]; then
    ceremony_msg "❌ Binary at $ARTIFACT appears empty" "error"
    exit 1
fi

step_msg "Step 2" "Ensuring install directories exist"
mkdir -p "$LIB_DIR"
mkdir -p "$BIN_DIR"

step_msg "Step 3" "Installing cage to $LIB_DIR"
install -m 755 "$ARTIFACT" "$LIB_DIR/${BINARY_NAME}"

step_msg "Step 4" "Updating symlink in $BIN_DIR"
ln -sf "$LIB_DIR/${BINARY_NAME}" "$BIN_DIR/${BINARY_NAME}"
echo "  Symlink: $BIN_DIR/${BINARY_NAME} → $LIB_DIR/${BINARY_NAME}"

step_msg "Step 5" "Running smoke check"
if ! "$BIN_DIR/${BINARY_NAME}" --help >/dev/null 2>&1; then
    ceremony_msg "❌ cage command test failed" "error"
    exit 1
fi
echo "✅ cage --help responded successfully"

ceremony_msg "✅ CAGE v$VERSION DEPLOYED" "success"
cat <<SUMMARY
📍 Library: $LIB_DIR/${BINARY_NAME}
📍 Binary:  $BIN_DIR/${BINARY_NAME}

Next steps:
  cage stream encrypt --input file --output file.cage --recipient age1...
  cage init    # hydrate XDG config/data directories
SUMMARY
