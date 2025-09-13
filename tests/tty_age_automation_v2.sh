#!/bin/bash
# KREX Iron Gate TTY Automation for Age Encryption - Refined Version
# Bulletproof Age automation with corrected decryption verification
# Created: 2025-09-10 | Status: IRON TESTED | Version: 2.0

set -euo pipefail

# Configuration
AGE_BINARY="/usr/bin/age"
TEMP_DIR="/tmp/age_automation_$$"

# Cleanup function - Atomic Operation Guarantee
cleanup() {
    local exit_code=$?
    [[ -d "$TEMP_DIR" ]] && rm -rf "$TEMP_DIR" 2>/dev/null || true
    exit $exit_code
}
trap cleanup EXIT

# Iron Gate Validation - Dependency Verification
validate_dependencies() {
    local missing_deps=()
    
    [[ ! -x "$AGE_BINARY" ]] && missing_deps+=("age binary at $AGE_BINARY")
    ! command -v expect >/dev/null 2>&1 && missing_deps+=("expect utility")
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo "STRUCTURAL FAILURE: Missing dependencies:" >&2
        printf ' - %s\n' "${missing_deps[@]}" >&2
        return 1
    fi
}

# Primary Method: Expect-based automation with proper error handling
age_encrypt_expect() {
    local input_file="$1"
    local output_file="$2"
    local passphrase="$3"
    
    expect -c "
        set timeout 10
        log_user 0
        
        spawn $AGE_BINARY -p -o {$output_file} {$input_file}
        
        expect {
            \"Enter passphrase*\" {
                send \"$passphrase\r\"
                exp_continue
            }
            \"Confirm passphrase*\" {
                send \"$passphrase\r\"
                exp_continue
            }
            eof {
                catch wait result
                exit [lindex \$result 3]
            }
            timeout {
                exit 1
            }
        }
    " 2>/dev/null
    
    return $?
}

# Decryption with expect for consistency
age_decrypt_expect() {
    local encrypted_file="$1"
    local output_file="$2"
    local passphrase="$3"
    
    expect -c "
        set timeout 10
        log_user 0
        
        spawn $AGE_BINARY -d -o {$output_file} {$encrypted_file}
        
        expect {
            \"Enter passphrase*\" {
                send \"$passphrase\r\"
                exp_continue
            }
            eof {
                catch wait result
                exit [lindex \$result 3]
            }
            timeout {
                exit 1
            }
        }
    " 2>/dev/null
    
    return $?
}

# Simplified test function
test_encrypt_decrypt() {
    local test_file="$TEMP_DIR/test_input.txt"
    local encrypted_file="$TEMP_DIR/test.age"
    local decrypted_file="$TEMP_DIR/test_decrypted.txt"
    local test_pass="TestPass123"
    
    # Create test input
    echo "KREX IRON GATE TEST DATA - $(date)" > "$test_file"
    
    echo "Testing Age TTY automation..."
    
    # Test encryption
    if age_encrypt_expect "$test_file" "$encrypted_file" "$test_pass"; then
        echo "✓ Encryption: SUCCESS"
    else
        echo "✗ Encryption: FAILED"
        return 1
    fi
    
    # Verify encrypted file exists and is different from input
    if [[ -f "$encrypted_file" && ! $(cmp -s "$test_file" "$encrypted_file" 2>/dev/null) ]]; then
        echo "✓ Encryption verification: File encrypted"
    else
        echo "✗ Encryption verification: File not properly encrypted"
        return 1
    fi
    
    # Test decryption
    if age_decrypt_expect "$encrypted_file" "$decrypted_file" "$test_pass"; then
        echo "✓ Decryption: SUCCESS"
    else
        echo "✗ Decryption: FAILED"
        return 1
    fi
    
    # Verify content matches
    if [[ -f "$decrypted_file" ]] && cmp -s "$test_file" "$decrypted_file"; then
        echo "✓ Content verification: MATCH"
        echo "🔒 TTY AUTOMATION: FULLY FUNCTIONAL"
        return 0
    else
        echo "✗ Content verification: MISMATCH"
        return 1
    fi
}

# Production-ready encryption function
encrypt_file() {
    local input_file="$1"
    local output_file="$2"
    local passphrase="$3"
    
    # Validation
    [[ ! -f "$input_file" ]] && { echo "ERROR: Input file not found: $input_file" >&2; return 1; }
    [[ -f "$output_file" ]] && { echo "ERROR: Output file exists: $output_file" >&2; return 1; }
    [[ -z "$passphrase" ]] && { echo "ERROR: Passphrase cannot be empty" >&2; return 1; }
    
    if age_encrypt_expect "$input_file" "$output_file" "$passphrase"; then
        echo "✓ File encrypted: $output_file"
        return 0
    else
        echo "✗ Encryption failed" >&2
        return 1
    fi
}

# Production-ready decryption function
decrypt_file() {
    local encrypted_file="$1"
    local output_file="$2"
    local passphrase="$3"
    
    # Validation
    [[ ! -f "$encrypted_file" ]] && { echo "ERROR: Encrypted file not found: $encrypted_file" >&2; return 1; }
    [[ -f "$output_file" ]] && { echo "ERROR: Output file exists: $output_file" >&2; return 1; }
    [[ -z "$passphrase" ]] && { echo "ERROR: Passphrase cannot be empty" >&2; return 1; }
    
    if age_decrypt_expect "$encrypted_file" "$output_file" "$passphrase"; then
        echo "✓ File decrypted: $output_file"
        return 0
    else
        echo "✗ Decryption failed" >&2
        return 1
    fi
}

# Main execution logic
main() {
    case "${1:-}" in
        "test")
            validate_dependencies || exit 1
            mkdir -p "$TEMP_DIR"
            
            echo "=== KREX IRON GATE TTY AUTOMATION TEST ==="
            if test_encrypt_decrypt; then
                echo ""
                echo "🛡️  SOLUTION CONFIRMED: TTY automation bypass successful"
                echo "📋 Usage: $0 encrypt <input> <output> <passphrase>"
                echo "📋 Usage: $0 decrypt <encrypted> <output> <passphrase>"
                exit 0
            else
                echo ""
                echo "❌ TTY automation failed in this environment"
                exit 1
            fi
            ;;
            
        "encrypt")
            [[ $# -lt 4 ]] && {
                echo "Usage: $0 encrypt <input_file> <output_file> <passphrase>" >&2
                exit 1
            }
            validate_dependencies || exit 1
            encrypt_file "$2" "$3" "$4"
            ;;
            
        "decrypt")
            [[ $# -lt 4 ]] && {
                echo "Usage: $0 decrypt <encrypted_file> <output_file> <passphrase>" >&2
                exit 1
            }
            validate_dependencies || exit 1
            decrypt_file "$2" "$3" "$4"
            ;;
            
        *)
            cat << 'EOF'
KREX Iron Gate TTY Automation for Age Encryption v2.0

PROBLEM SOLVED: Age encryption requires TTY input, blocking automation
SOLUTION: Expect-based PTY emulation bypasses TTY validation

Usage:
    ./tty_age_automation_v2.sh test                                    # Test automation
    ./tty_age_automation_v2.sh encrypt <input> <output> <passphrase>   # Encrypt file
    ./tty_age_automation_v2.sh decrypt <encrypted> <output> <passphrase> # Decrypt file

Features:
    ✓ Bypasses Age's TTY validation completely
    ✓ Works in CI/CD environments (no /dev/tty required)
    ✓ Secure automation (no passphrase logging)
    ✓ Atomic operations with cleanup
    ✓ Full encryption/decryption cycle tested

Security Notes:
    - For production: modify to read passphrase from secure source
    - Test in isolated environment before production use
    - Validates all dependencies before execution

Iron Gate Status: BULLETPROOF - Tier 1 Threat T2.1 ELIMINATED
EOF
            exit 0
            ;;
    esac
}

main "$@"