#!/bin/bash
#
# Check for todo!() macros in the codebase
#
# This script helps ensure no todo!() macros are left in production code
# Usage: ./scripts/check_todos.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Checking for todo!() macros ===${NC}"
echo ""

# Search for todo!() macros in Rust files
# Exclude test files and examples
todos=$(grep -r "todo!()" --include="*.rs" src/ 2>/dev/null | grep -v -E "test|example" || true)

if [[ -z "$todos" ]]; then
    echo -e "${GREEN}✅ No todo!() macros found in production code${NC}"
    exit 0
else
    echo -e "${RED}❌ Found todo!() macros in production code:${NC}"
    echo "$todos"
    echo ""
    echo -e "${YELLOW}Please replace todo!() macros with proper implementations or unimplemented!()${NC}"
    exit 1
fi