# Age CLI Reference Documentation

**Purpose**: Document expected age CLI behavior to detect compatibility issues during upgrades.
**Created**: 2025-09-29
**Current Age Version**: 1.1.1

## Expected Command Outputs

### `age --version`

```
1.1.1
```

**Notes**:
- Returns just the version number without "age" prefix
- Exit code: 0

### `age --help`

```
Usage:
    age [--encrypt] (-r RECIPIENT | -R PATH)... [--armor] [-o OUTPUT] [INPUT]
    age [--encrypt] --passphrase [--armor] [-o OUTPUT] [INPUT]
    age --decrypt [-i PATH]... [-o OUTPUT] [INPUT]

Options:
    -e, --encrypt               Encrypt the input to the output. Default if omitted.
    -d, --decrypt               Decrypt the input to the output.
    -o, --output OUTPUT         Write the result to the file at path OUTPUT.
    -a, --armor                 Encrypt to a PEM encoded format.
    -p, --passphrase            Encrypt with a passphrase.
    -r, --recipient RECIPIENT   Encrypt to the specified RECIPIENT. Can be repeated.
    -R, --recipients-file PATH  Encrypt to recipients listed at PATH. Can be repeated.
    -i, --identity PATH         Use the identity file at PATH. Can be repeated.
```

**Notes**:
- May output to stdout or stderr depending on version
- Exit code: 0 or 2

### `age-keygen --version` (optional)

```
age-keygen 1.1.1
```

**Notes**:
- Optional tool, may not be present
- Includes "age-keygen" prefix unlike main age binary

## Critical Flags to Monitor

These flags are essential for Cage operation and must be present:

| Flag | Purpose | Critical |
|------|---------|----------|
| `--encrypt` / `-e` | Encryption mode | ✓ |
| `--decrypt` / `-d` | Decryption mode | ✓ |
| `--passphrase` / `-p` | Passphrase encryption | ✓ |
| `--identity` / `-i` | Identity file for decryption | ✓ |
| `--recipient` / `-r` | Recipient for encryption | ✓ |
| `--output` / `-o` | Output file specification | ✓ |
| `--armor` / `-a` | ASCII armor format | ✓ |
| `--recipients-file` / `-R` | Recipient file | ✓ |

## SSH Key Support

Age accepts these SSH key formats as recipients:
- `ssh-rsa AAAA...`
- `ssh-ed25519 AAAA...`
- `ecdsa-sha2-nistp256 AAAA...`
- `ecdsa-sha2-nistp384 AAAA...`
- `ecdsa-sha2-nistp521 AAAA...`

## Test Coverage

Sanity tests are located in: `tests/test_age_cli_sanity.rs`

The tests verify:
1. Age binary availability
2. Version output format
3. Help output contains critical flags
4. Optional age-keygen availability

## Upgrade Checklist

When upgrading age version:

1. [ ] Run `cargo test --test test_age_cli_sanity`
2. [ ] Check for new/deprecated flags in help output
3. [ ] Verify SSH key format support unchanged
4. [ ] Update this document with any changes
5. [ ] Test core encryption/decryption operations
6. [ ] Check PTY automation still works

## Known Version Differences

### age 1.0.0 vs 1.1.0+
- Added plugin support
- Enhanced error messages
- No breaking CLI changes

### Future Compatibility
- Monitor for changes to flag names
- Watch for new required parameters
- Check for deprecated features