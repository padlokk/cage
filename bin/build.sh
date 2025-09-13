#!/bin/bash
# Cage Build Script - Age Encryption Automation CLI
# Builds the standalone cage CLI tool for Age encryption automation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build configuration
BUILD_TYPE="${1:-release}"
BUILD_MODE="${2:-optimized}"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}       Cage Build System v0.1.0                    ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Check if age is available, install if missing
check_age_installation() {
    if ! command -v age >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Age not found on system${NC}"
        echo -e "${YELLOW}🔧 Installing age encryption tool...${NC}"

        # Detect OS and install age accordingly
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if command -v apt-get >/dev/null 2>&1; then
                # Ubuntu/Debian
                sudo apt-get update && sudo apt-get install -y age
            elif command -v yum >/dev/null 2>&1; then
                # RHEL/CentOS
                sudo yum install -y age
            elif command -v pacman >/dev/null 2>&1; then
                # Arch Linux
                sudo pacman -S --noconfirm age
            else
                echo -e "${RED}❌ Unable to auto-install age. Please install manually:${NC}"
                echo "Visit: https://github.com/FiloSottile/age/releases"
                exit 1
            fi
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            if command -v brew >/dev/null 2>&1; then
                brew install age
            else
                echo -e "${RED}❌ Homebrew not found. Please install age manually:${NC}"
                echo "Visit: https://github.com/FiloSottile/age/releases"
                exit 1
            fi
        else
            echo -e "${RED}❌ Unsupported OS. Please install age manually:${NC}"
            echo "Visit: https://github.com/FiloSottile/age/releases"
            exit 1
        fi

        # Verify installation
        if command -v age >/dev/null 2>&1; then
            AGE_VERSION=$(age --version 2>&1 | head -n1)
            echo -e "${GREEN}✅ Age installed successfully: ${AGE_VERSION}${NC}"
        else
            echo -e "${RED}❌ Age installation failed${NC}"
            exit 1
        fi
    else
        AGE_VERSION=$(age --version 2>&1 | head -n1)
        echo -e "${GREEN}✅ Age found: ${AGE_VERSION}${NC}"
    fi
}

# Check age installation first
check_age_installation

case "$BUILD_TYPE" in
    release|prod)
        echo -e "${YELLOW}🔒 Building cage release version...${NC}"
        cargo build --release --bin cage
        echo -e "${GREEN}✅ Cage release build complete${NC}"
        echo -e "${YELLOW}   Binary: cage (Age encryption automation)${NC}"
        echo -e "${YELLOW}   Features: PTY automation, batch processing${NC}"
        ;;

    debug|dev)
        echo -e "${YELLOW}🔧 Building cage debug version...${NC}"
        cargo build --bin cage
        echo -e "${GREEN}✅ Cage debug build complete${NC}"
        echo -e "${YELLOW}   Binary: cage (Age encryption automation)${NC}"
        echo -e "${YELLOW}   Features: PTY automation, verbose logging${NC}"
        ;;

    test)
        echo -e "${YELLOW}🧪 Building and testing cage...${NC}"

        # Test build
        echo -e "${BLUE}Testing cage build...${NC}"
        cargo build --bin cage
        cargo test

        echo -e "${GREEN}✅ All tests passed successfully${NC}"
        ;;

    clean)
        echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
        cargo clean
        echo -e "${GREEN}✅ Build directory cleaned${NC}"
        ;;

    *)
        echo -e "${RED}❌ Unknown build type: $BUILD_TYPE${NC}"
        echo ""
        echo "Usage: $0 [build-type]"
        echo ""
        echo "Build types:"
        echo "  release|prod  - Release build with optimizations (default)"
        echo "  debug|dev     - Debug build with verbose output"
        echo "  test          - Build and run tests"
        echo "  clean         - Clean build artifacts"
        echo ""
        echo "Examples:"
        echo "  $0              # Release build"
        echo "  $0 debug        # Debug build"
        echo "  $0 test         # Test build"
        echo "  $0 clean        # Clean build"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"

# Show binary locations
if [[ "$BUILD_TYPE" == "release" || "$BUILD_TYPE" == "prod" ]]; then
    BINARY_DIR="target/release"
else
    BINARY_DIR="target/debug"
fi

# Check if cage binary was built
echo -e "${GREEN}📦 Built binary:${NC}"

if [ -f "$BINARY_DIR/cage" ]; then
    SIZE=$(du -h "$BINARY_DIR/cage" | cut -f1)
    echo -e "${GREEN}   cage: $BINARY_DIR/cage ($SIZE)${NC}"
    echo -e "${GREEN}   ✅ Cage is ready for deployment${NC}"
else
    echo -e "${RED}   ❌ Cage binary not found${NC}"
fi

echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo -e "${YELLOW}   • Deploy cage: ./bin/deploy.sh${NC}"
echo -e "${YELLOW}   • Test cage: $BINARY_DIR/cage --help${NC}"
echo -e "${YELLOW}   • Try demo: $BINARY_DIR/cage demo${NC}"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"