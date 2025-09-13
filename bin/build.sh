#!/bin/bash
# Padlock Build Script - CLI Subtools for Age Automation
# Builds independent CLI tools for Age automation testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build configuration
BUILD_TYPE="${1:-cli}"
BUILD_MODE="${2:-debug}"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Padlock CLI Build System v0.1.0                ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

case "$BUILD_TYPE" in
    cli|subtools)
        echo -e "${YELLOW}🔧 Building CLI subtools (cage, cli_auth)...${NC}"
        if [ "$BUILD_MODE" = "release" ]; then
            cargo build --release --bin cage --bin cli_auth
        else
            cargo build --bin cage --bin cli_auth
        fi
        echo -e "${GREEN}✅ CLI subtools build complete${NC}"
        echo -e "${YELLOW}   Tools: cage (Age automation), cli_auth (Authority chain)${NC}"
        echo -e "${YELLOW}   PTY automation enabled${NC}"
        ;;

    all)
        echo -e "${YELLOW}⚡ Building ALL binaries (padlock, cage, cli_auth)...${NC}"
        if [ "$BUILD_MODE" = "release" ]; then
            cargo build --release
        else
            cargo build
        fi
        echo -e "${GREEN}✅ Full build complete${NC}"
        echo -e "${YELLOW}   Binaries: padlock, cage, cli_auth${NC}"
        echo -e "${YELLOW}   Age automation with PTY${NC}"
        ;;

    padlock)
        echo -e "${YELLOW}🔐 Building main padlock binary...${NC}"
        if [ "$BUILD_MODE" = "release" ]; then
            cargo build --release --bin padlock
        else
            cargo build --bin padlock
        fi
        echo -e "${GREEN}✅ Padlock main binary complete${NC}"
        echo -e "${YELLOW}   Features: Complete cryptographic repository management${NC}"
        ;;

    test)
        echo -e "${YELLOW}🧪 Building and testing all configurations...${NC}"

        # Test CLI tools build
        echo -e "${BLUE}Testing CLI tools build...${NC}"
        cargo build --bin cage --bin cli_auth
        cargo test

        # Test full build
        echo -e "${BLUE}Testing complete build...${NC}"
        cargo build

        # Test PTY automation
        echo -e "${BLUE}Testing PTY automation...${NC}"
        if [ -f "src/age_driver.rs" ]; then
            cargo build --bin age_driver
            echo -e "${GREEN}✅ PTY driver test build successful${NC}"
        fi

        echo -e "${GREEN}✅ All configurations tested successfully${NC}"
        ;;

    clean)
        echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
        cargo clean
        echo -e "${GREEN}✅ Build directory cleaned${NC}"
        ;;

    *)
        echo -e "${RED}❌ Unknown build type: $BUILD_TYPE${NC}"
        echo ""
        echo "Usage: $0 [build-type] [build-mode]"
        echo ""
        echo "Build types:"
        echo "  cli|subtools  - Build CLI subtools only (default)"
        echo "  all           - Build all binaries"
        echo "  padlock       - Build main padlock binary only"
        echo "  test          - Test all build configurations"
        echo "  clean         - Clean build artifacts"
        echo ""
        echo "Build modes:"
        echo "  debug         - Debug build (default)"
        echo "  release       - Release build with optimizations"
        echo ""
        echo "Examples:"
        echo "  $0              # CLI subtools debug build"
        echo "  $0 cli release  # CLI subtools release build"
        echo "  $0 all          # All binaries debug build"
        echo "  $0 test         # Test all configurations"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"

# Show binary locations
if [ "$BUILD_MODE" = "release" ]; then
    BINARY_DIR="target/release"
else
    BINARY_DIR="target/debug"
fi

# Check which binaries were built
echo -e "${GREEN}📦 Built binaries:${NC}"

if [ -f "$BINARY_DIR/cage" ]; then
    SIZE=$(du -h "$BINARY_DIR/cage" | cut -f1)
    echo -e "${GREEN}   cage: $BINARY_DIR/cage ($SIZE)${NC}"
fi

if [ -f "$BINARY_DIR/cli_auth" ]; then
    SIZE=$(du -h "$BINARY_DIR/cli_auth" | cut -f1)
    echo -e "${GREEN}   cli_auth: $BINARY_DIR/cli_auth ($SIZE)${NC}"
fi

if [ -f "$BINARY_DIR/padlock" ]; then
    SIZE=$(du -h "$BINARY_DIR/padlock" | cut -f1)
    echo -e "${GREEN}   padlock: $BINARY_DIR/padlock ($SIZE)${NC}"
fi

if [ -f "$BINARY_DIR/age_driver" ]; then
    SIZE=$(du -h "$BINARY_DIR/age_driver" | cut -f1)
    echo -e "${GREEN}   age_driver: $BINARY_DIR/age_driver ($SIZE) [test]${NC}"
fi

echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo -e "${YELLOW}   • Deploy CLI tools: ./bin/deploy.sh${NC}"
echo -e "${YELLOW}   • Test PTY automation: $BINARY_DIR/age_driver${NC}"
echo -e "${YELLOW}   • Test CLI tools: $BINARY_DIR/cage help${NC}"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"