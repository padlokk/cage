# CLI RSB Usage - Important Flag Order Requirements

## ⚠️ **CRITICAL: RSB Flag Order**

**RSB requires flags to come LAST after all command arguments.**

### ✅ **CORRECT Usage:**
```bash
# Command + Arguments + Flags
meteor parse "global:main:button=click" --verbose
meteor validate "app:ui:theme=dark" --verbose
meteor parse "button=click; theme=dark" --format=json
```

### ❌ **INCORRECT Usage (Will Not Work):**
```bash
# Flags before arguments - WRONG!
meteor parse --verbose "global:main:button=click"     # ❌ BROKEN
meteor --verbose parse "global:main:button=click"     # ❌ BROKEN
meteor validate --format=json "app:ui:theme=dark"     # ❌ BROKEN
```

## Why This Happens

The RSB `options!` macro processes flags from the argument list, but it expects them to appear at the end. When flags appear before arguments:

1. `options!` doesn't recognize them as flags
2. They get included in `args.remaining()`
3. They become part of the input string
4. Parsing fails or produces wrong results

## RSB Architecture Pattern

```rust
fn main() {
    let args = bootstrap!();    // Parse command line
    options!(&args);           // Process flags (expects them last)
    dispatch!(&args, {         // Route to command handlers
        "parse" => parse_command
    });
}

fn parse_command(args: Args) -> i32 {
    // Flags are now in global context via get_var()
    let verbose = get_var("opt_verbose") == "true";

    // Arguments are in args.remaining()
    let input = args.remaining().join(" ");
}
```

## Current CLI Commands

### Parse Command
```bash
# Basic parsing
meteor parse "global:main:button=click"

# With verbose output
meteor parse "global:main:button=click" --verbose

# With JSON format
meteor parse "app:ui:theme=dark" --format=json

# Complex meteor shower
meteor parse "app:ui:button=click; user:settings:theme=dark" --verbose
```

### Validate Command
```bash
# Basic validation
meteor validate "global:main:button=click"

# With verbose output
meteor validate "app:ui:theme=dark" --verbose
```

## Expected Output Format

With the modernized CLI, you'll see detailed meteor component breakdown:

```
=== Meteor Parse Results ===
Current cursor: app:main

=== Parsed Data ===
Meteor 1:
  Context: global
  Namespace: main
  Key: button
  Value: click

Total: 1 meteors across 1 contexts
```

## Troubleshooting

**Problem**: Flags appearing in the parsed value
```
Key: button
Value: click --verbose    # ❌ Flag included in value
```

**Solution**: Move flags to the end
```bash
meteor parse "global:main:button=click" --verbose  # ✅ Correct order
```

**Problem**: "No input provided" error when flags are present
```
Error: No input provided
```

**Solution**: This means flags were processed as the input - move them to the end.

## Development Note

This issue has occurred multiple times during development. The RSB pattern requires:

1. ✅ `bootstrap!()` - Parse command line
2. ✅ `options!()` - Extract flags (expects them last)
3. ✅ `dispatch!()` - Route to handlers
4. ✅ `get_var("opt_*")` - Access flags in handlers
5. ✅ `args.remaining()` - Access non-flag arguments

**Remember**: Arguments first, flags last!