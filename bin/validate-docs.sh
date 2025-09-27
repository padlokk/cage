#!/bin/bash
#===============================================================================
# 🔍 CAGE DOCUMENTATION VALIDATOR
#===============================================================================
#
# Purpose: Silent success, noisy failure validation
# Usage: ./bin/validate-docs.sh
# Output: Only shows problems - hides successful validations
#
# Created: 2025-09-27 (META_PROCESS v2 implementation)
#===============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

ERROR_COUNT=0
WARN_COUNT=0

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Helper functions
error() {
    echo -e "${RED}ERROR${NC}: $1" >&2
    ((ERROR_COUNT++))
}

warn() {
    echo -e "${YELLOW}WARN${NC}: $1" >&2
    ((WARN_COUNT++))
}

success() {
    if [[ "${VERBOSE:-}" == "true" ]]; then
        echo -e "${GREEN}OK${NC}: $1"
    fi
}

check_file_exists() {
    local file="$1"
    local description="$2"

    if [[ -f "$file" ]]; then
        success "$description exists: $file"
        return 0
    else
        error "$description missing: $file"
        return 1
    fi
}

check_directory_exists() {
    local dir="$1"
    local description="$2"

    if [[ -d "$dir" ]]; then
        success "$description exists: $dir"
        return 0
    else
        error "$description missing: $dir"
        return 1
    fi
}

check_file_age() {
    local file="$1"
    local max_age_days="$2"
    local description="$3"

    if [[ ! -f "$file" ]]; then
        return 1 # File doesn't exist, already reported elsewhere
    fi

    local file_age_days
    if [[ "$(uname)" == "Darwin" ]]; then
        # macOS
        file_age_days=$(( ($(date +%s) - $(stat -f %m "$file")) / 86400 ))
    else
        # Linux
        file_age_days=$(( ($(date +%s) - $(stat -c %Y "$file")) / 86400 ))
    fi

    if [[ $file_age_days -gt $max_age_days ]]; then
        warn "$description is $file_age_days days old (max: $max_age_days): $file"
        return 1
    else
        success "$description is fresh ($file_age_days days old): $file"
        return 0
    fi
}

check_internal_references() {
    local file="$1"
    local description="$2"

    if [[ ! -f "$file" ]]; then
        return 1 # File doesn't exist, already reported elsewhere
    fi

    # Extract file references (simple patterns)
    local broken_refs=0

    # Check for docs/ references
    while IFS= read -r line; do
        if [[ -n "$line" && ! -f "$line" ]]; then
            error "Broken reference in $description: $line"
            ((broken_refs++))
        fi
    done < <(grep -oE 'docs/[a-zA-Z0-9_/.-]+\.(md|txt)' "$file" 2>/dev/null || true)

    # Check for bin/ references
    while IFS= read -r line; do
        if [[ -n "$line" && ! -f "$line" ]]; then
            error "Broken reference in $description: $line"
            ((broken_refs++))
        fi
    done < <(grep -oE 'bin/[a-zA-Z0-9_.-]+\.sh' "$file" 2>/dev/null || true)

    if [[ $broken_refs -eq 0 ]]; then
        success "$description has no broken references"
        return 0
    else
        return 1
    fi
}

#===============================================================================
# VALIDATION CHECKS
#===============================================================================

echo "🔍 Validating Cage documentation integrity..."
echo

# Core entry points
echo "📋 Checking core entry points..."
check_file_exists "START.txt" "Entry point"
check_file_exists "README.md" "Project README"
check_file_exists "LICENSE" "License file"

# Process documentation structure
echo
echo "📁 Checking process documentation structure..."
check_directory_exists "docs/procs" "Process documentation directory"
check_file_exists "docs/procs/PROCESS.txt" "Master workflow guide"
check_file_exists "docs/procs/CONTINUE.md" "Session status document"
check_file_exists "docs/procs/TASKS.txt" "Task breakdown"
check_file_exists "docs/procs/QUICK_REF.txt" "Quick reference"
check_file_exists "docs/procs/SPRINT.txt" "Sprint planning"
check_file_exists "docs/procs/DONE.txt" "Completed work archive"

# Reference documentation
echo
echo "📚 Checking reference documentation..."
check_directory_exists "docs/ref" "Reference documentation directory"
check_file_exists "docs/ref/ROADMAP.md" "Development roadmap"
check_file_exists "docs/ref/RSB_LESSONS.md" "RSB lessons learned"
check_file_exists "docs/ref/SAFETY_DESIGN.md" "Safety design guide"

# Miscellaneous documentation
echo
echo "📄 Checking miscellaneous documentation..."
check_directory_exists "docs/misc" "Miscellaneous documentation directory"
check_file_exists "docs/misc/CAGE_PTY_FIX.md" "PTY fix documentation"

# Analysis directory (if using agents)
echo
echo "🧠 Checking analysis consolidation..."
if [[ -d ".analysis" ]]; then
    success "Analysis directory exists"
    check_file_exists ".analysis/consolidated_wisdom.txt" "Consolidated wisdom"
    check_file_exists ".analysis/technical_debt.txt" "Technical debt analysis"
else
    warn "Analysis directory missing (optional for manual projects)"
fi

# Validation tools
echo
echo "🔧 Checking validation tools..."
check_file_exists "bin/validate-docs.sh" "Documentation validator"
if [[ -f "bin/validate-docs.sh" ]]; then
    if [[ -x "bin/validate-docs.sh" ]]; then
        success "Documentation validator is executable"
    else
        error "Documentation validator is not executable"
    fi
fi

# Staleness checks (critical docs should be fresh)
echo
echo "⏰ Checking document freshness..."
check_file_age "docs/procs/CONTINUE.md" 7 "Session status"
check_file_age "docs/procs/QUICK_REF.txt" 30 "Quick reference"
check_file_age "docs/procs/TASKS.txt" 14 "Task breakdown"

# Internal reference integrity
echo
echo "🔗 Checking internal references..."
check_internal_references "START.txt" "Entry point"
check_internal_references "docs/procs/PROCESS.txt" "Master workflow"
check_internal_references "docs/procs/QUICK_REF.txt" "Quick reference"

# Project structure validation
echo
echo "🏗️ Checking project structure..."
check_directory_exists "src" "Source code directory"
check_directory_exists "docs" "Documentation directory"
check_directory_exists "bin" "Binary/script directory"

# Optional: Check for common development files (warnings only)
echo
echo "⚙️ Checking development infrastructure..."
if [[ ! -f "Cargo.toml" ]]; then
    warn "Cargo.toml missing (Rust project manifest)"
fi

if [[ ! -f "bin/test.sh" ]]; then
    warn "bin/test.sh missing (test runner)"
fi

if [[ ! -d "tests" ]]; then
    warn "tests/ directory missing (test suite)"
fi

if [[ ! -f "META_PROCESS.txt" ]]; then
    warn "META_PROCESS.txt missing (workflow documentation)"
fi

#===============================================================================
# SUMMARY
#===============================================================================

echo
echo "═══════════════════════════════════════════"
if [[ $ERROR_COUNT -eq 0 && $WARN_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✅ All documentation validations passed!${NC}"
    echo "Documentation system is healthy and ready for use."
    exit 0
elif [[ $ERROR_COUNT -eq 0 ]]; then
    echo -e "${YELLOW}⚠️ Documentation validation completed with warnings${NC}"
    echo "Errors: $ERROR_COUNT | Warnings: $WARN_COUNT"
    echo "System is functional but has minor issues to address."
    exit 1
else
    echo -e "${RED}❌ Documentation validation failed${NC}"
    echo "Errors: $ERROR_COUNT | Warnings: $WARN_COUNT"
    echo "Critical documentation issues must be resolved."
    exit 2
fi