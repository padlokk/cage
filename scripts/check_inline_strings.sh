#!/bin/bash
#
# String Literal Audit Script (SEC-01)
#
# This script helps identify inline string literals in the codebase
# that should potentially be moved to src/cage/strings.rs
#
# Usage: ./scripts/check_inline_strings.sh [--verbose]

set -euo pipefail

VERBOSE=0
if [[ "${1:-}" == "--verbose" ]]; then
    VERBOSE=1
fi

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Directories to scan (critical modules)
CRITICAL_DIRS=(
    "src/cage/adapter_v2.rs"
    "src/cage/lifecycle/crud_manager.rs"
    "src/cage/pty_wrap.rs"
    "src/cage/error.rs"
    "src/bin/cli_age.rs"
)

# Patterns to exclude (acceptable inline strings)
EXCLUDE_PATTERNS=(
    '""'                    # Empty strings
    '" "'                   # Single space
    '"\n"'                  # Newline
    '"\t"'                  # Tab
    '"/"'                   # Path separator
    '"."'                   # Current directory
    '".."'                  # Parent directory
    '".age"'                # File extension
    '".bak"'                # Backup extension
    '"age"'                 # Binary name (might be okay)
    '"cage"'                # Tool name
    '"ssh-'                 # SSH key prefixes
    '"ecdsa-'              # ECDSA key prefixes
    'env!('                 # Compile-time env macros
    'include_str!'          # Include macros
    'format!'               # Format macros with variables
    'println!'              # Debug output
    'eprintln!'             # Error output
    'debug!'                # Log macros
    'trace!'                # Log macros
    'info!'                 # Log macros
    'warn!'                 # Log macros
    'error!'                # Log macros
)

echo -e "${YELLOW}=== String Literal Audit Report ===${NC}"
echo -e "Scanning for inline strings that could be centralized..."
echo ""

total_strings=0
candidate_strings=0

for file in "${CRITICAL_DIRS[@]}"; do
    if [[ ! -f "$file" ]]; then
        continue
    fi

    echo -e "${GREEN}Scanning: $file${NC}"

    # Find all string literals (basic regex for strings in quotes)
    # This catches most cases but isn't perfect (doesn't handle escaped quotes well)
    strings=$(grep -n '"[^"]*"' "$file" 2>/dev/null || true)

    if [[ -z "$strings" ]]; then
        echo "  No string literals found"
        continue
    fi

    file_total=0
    file_candidates=0

    while IFS= read -r line; do
        file_total=$((file_total + 1))

        # Check if this line should be excluded
        excluded=0
        for pattern in "${EXCLUDE_PATTERNS[@]}"; do
            if echo "$line" | grep -q "$pattern"; then
                excluded=1
                break
            fi
        done

        # Skip test assertions and debug output
        if echo "$line" | grep -q -E 'assert|test|#\[cfg\(test\)\]|mod tests'; then
            excluded=1
        fi

        # Skip comments
        if echo "$line" | grep -q -E '^\s*//'; then
            excluded=1
        fi

        if [[ $excluded -eq 0 ]]; then
            file_candidates=$((file_candidates + 1))
            if [[ $VERBOSE -eq 1 ]]; then
                echo -e "  ${YELLOW}Line $line${NC}"
            fi
        fi
    done <<< "$strings"

    echo "  Found: $file_total strings ($file_candidates potential candidates)"
    total_strings=$((total_strings + file_total))
    candidate_strings=$((candidate_strings + file_candidates))
done

echo ""
echo -e "${YELLOW}=== Summary ===${NC}"
echo "Total string literals found: $total_strings"
echo "Potential candidates for centralization: $candidate_strings"

if [[ $candidate_strings -gt 50 ]]; then
    echo -e "${RED}⚠️  High number of inline strings detected${NC}"
    echo "Consider moving user-facing messages to src/cage/strings.rs"
elif [[ $candidate_strings -gt 20 ]]; then
    echo -e "${YELLOW}⚠️  Moderate number of inline strings detected${NC}"
    echo "Review and centralize user-facing messages"
else
    echo -e "${GREEN}✅ Inline string usage is reasonable${NC}"
fi

echo ""
echo "To see detailed results, run: $0 --verbose"
echo "To add more exclusions, edit the EXCLUDE_PATTERNS array in this script"