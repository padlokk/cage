#!/bin/bash
# Documentation Validation Script for Cage Project
# META_PROCESS v2 compliant - Silent success, noisy failure

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0

# Check if file exists
check_file() {
    local file="$1"
    local desc="$2"

    if [[ ! -f "$file" ]]; then
        echo -e "${RED}✗ MISSING: $desc${NC}"
        echo "  Expected: $file"
        ((ERRORS++))
        return 1
    fi
    return 0
}

# Check file staleness (age in days)
check_staleness() {
    local file="$1"
    local desc="$2"
    local threshold_days="$3"

    if [[ ! -f "$file" ]]; then
        return 0  # Already reported as missing
    fi

    local file_age_days=$(( ($(date +%s) - $(stat -c %Y "$file" 2>/dev/null || stat -f %m "$file" 2>/dev/null || echo 0)) / 86400 ))

    if [[ $file_age_days -gt $threshold_days ]]; then
        echo -e "${YELLOW}⚠ STALE: $desc (${file_age_days} days old)${NC}"
        echo "  File: $file"
        ((WARNINGS++))
    fi
}

# Check internal reference exists
check_reference() {
    local source_file="$1"
    local ref_pattern="$2"
    local ref_file="$3"

    if [[ ! -f "$source_file" ]]; then
        return 0  # Already reported as missing
    fi

    if grep -q "$ref_pattern" "$source_file" 2>/dev/null; then
        if [[ ! -f "$ref_file" ]]; then
            echo -e "${RED}✗ BROKEN REFERENCE: $source_file references missing file${NC}"
            echo "  Reference: $ref_pattern"
            echo "  Expected: $ref_file"
            ((ERRORS++))
        fi
    fi
}

echo "🔍 Validating Cage Documentation..."
echo

# === CRITICAL FILES (Must exist) ===
echo "Checking critical files..."
check_file "START.txt" "Entry point"
check_file "README.md" "Project README"
check_file "docs/procs/PROCESS.txt" "Master workflow"
check_file "docs/procs/CONTINUE.md" "Session handoff"
check_file "docs/procs/TASKS.txt" "Task breakdown"
check_file "docs/procs/QUICK_REF.txt" "Quick reference"

# === PROCESS FILES (Staleness: 7 days) ===
echo "Checking process document staleness..."
check_staleness "docs/procs/CONTINUE.md" "Session handoff" 7
check_staleness "docs/procs/TASKS.txt" "Task breakdown" 30

# === REFERENCE FILES ===
echo "Checking reference documentation..."
check_file "docs/ref/ROADMAP.md" "Development roadmap"
check_file "docs/ref/RSB_LESSONS.md" "RSB lessons"
check_file "docs/ref/SAFETY_DESIGN.md" "Safety design"

# === INTERNAL REFERENCES ===
echo "Checking internal references..."
check_reference "START.txt" "docs/procs/PROCESS.txt" "docs/procs/PROCESS.txt"
check_reference "START.txt" "docs/procs/CONTINUE.md" "docs/procs/CONTINUE.md"
check_reference "docs/procs/PROCESS.txt" "docs/procs/TASKS.txt" "docs/procs/TASKS.txt"
check_reference "docs/procs/PROCESS.txt" "docs/ref/ROADMAP.md" "docs/ref/ROADMAP.md"

# === DIRECTORY STRUCTURE ===
echo "Checking directory structure..."
for dir in docs/procs docs/ref docs/misc docs/lics .analysis; do
    if [[ ! -d "$dir" ]]; then
        echo -e "${RED}✗ MISSING DIRECTORY: $dir${NC}"
        ((ERRORS++))
    fi
done

# === SUMMARY ===
echo
if [[ $ERRORS -eq 0 && $WARNINGS -eq 0 ]]; then
    echo "✅ All documentation checks passed!"
    exit 0
elif [[ $ERRORS -eq 0 ]]; then
    echo -e "${YELLOW}⚠ Documentation valid with $WARNINGS warning(s)${NC}"
    exit 0
else
    echo -e "${RED}❌ Documentation validation failed with $ERRORS error(s) and $WARNINGS warning(s)${NC}"
    echo
    echo "Please fix errors before proceeding."
    exit 1
fi