# Cage Development Roadmap

**Last Updated:** 2025-09-27
**Current Status:** META_PROCESS v2 rollout + P0 bug slate
**Version:** 0.3.1

## Project Overview

Cage is an Age encryption automation CLI with PTY support that eliminates manual TTY interaction while maintaining cryptographic security standards. The project has a solid architectural foundation with comprehensive error handling, security validation, and working PTY automation.

---

## Current Feature Status

### ✅ Well Implemented Features (90%+ Complete)

**Core Architecture:**
- PTY automation with `portable-pty` (proven working in `src/driver.rs`)
- Comprehensive CLI with 8 commands using clap
- Rich error handling system (20+ error types with actionable guidance)
- Security validation and audit logging
- RSB framework integration for enhanced UX
- Unified version management across modules

**Working Operations:**
- `lock` - File/directory encryption with patterns, recursion, format options
- `unlock` - File/directory decryption with preservation options
- `status` - Encryption status analysis and reporting
- `demo` - Usage examples and capability demonstration
- File format support (binary/ASCII armor)
- Verbose logging and comprehensive audit trails

**Infrastructure:**
- Configuration management with environment-specific presets
- Security validator with injection prevention
- Audit logger with operation tracking
- Adapter pattern for multiple Age backends

---

## Critical Implementation Gaps

### ❌ P0 - Blocking Production (Must Fix)

| Task | Summary | Impact |
|------|---------|--------|
| **BUG-01** | Preserve original extensions during lock/unlock so round-trips follow documented `file.ext.cage` pattern. | Current behaviour breaks advertised UX and makes decrypt CLI examples unusable. |
| **BUG-02** | Rework traversal to honour `--recursive`, status, and verify across nested trees. | Directories are reported as processed but child files remain untouched. |
| **BUG-03** | Replace substring pattern filters with glob support (`*.log`, `**/*secret*`). | Flagged feature silently fails; security automation scripts can miss targets. |
| **BUG-04** | Wire unlock options to deletion & selective flows. | `--preserve`/`--selective` are ignored, causing data-retention surprises. |
| **BUG-05** | Remove `expect` dependency from proxy and reuse PTY stack with cross-platform guards. | Proxy command fails on systems without `expect` and contradicts PTY migration. |
| **CAGE-01** | Finish key rotation lifecycle (atomic, rollback, permission retention). | Rotation command remains a stub; compliance stories blocked. |
| **CAGE-02** | Deliver file integrity verification pipeline with actionable reporting. | No integrity signal; tampering/corruption undetectable. |
| **CAGE-03** | Implement backup/recovery system backing `--backup`. | Flag is cosmetic; there is no safety net for destructive ops. |
| **TEST-01** | Align CLI test suites with the `cage` binary and modern scenarios. | Current “integration tests” never run, masking regressions. |

### ❌ P1 - High Priority (Critical for Usability)

| Task | Summary |
|------|---------|
| **CAGE-04** | Complete in-place operation safety layers (RecoveryManager, danger mode prompts). |
| **CAGE-05** | Standardise progress/telemetry so long operations emit feedback. |
| **CAGE-06** | Add layered configuration file support (`~/.cagerc`, project overrides). |
| **CAGE-07** | Implement RageAdapter for alternative backend support. |
| **TEST-02** | Add regression coverage for BUG-01..BUG-04 scenarios. |
| **TEST-03** | Add proxy PTY integration coverage (skip gracefully when unsupported). |

---

## Development Phases

### Phase 1: MVP (4-6 weeks) - Target: 85% Production Ready

**Critical Completions:**
- [ ] **BUG-01 → BUG-05**: Ship the blocking fixes for extension handling, recursion, glob patterns, unlock options, and proxy PTY support.
- [ ] **CAGE-01**: Key rotation lifecycle with rollback and permission preservation.
- [ ] **CAGE-02**: File integrity verification with actionable reporting.
- [ ] **CAGE-03**: Backup & recovery pipeline powering `--backup` and rotation safeguards.
- [ ] **TEST-01**: Restore end-to-end CLI coverage against the `cage` binary.

**Progress Enablement:**
- [ ] **CAGE-05** groundwork for long-running job feedback once core bugs are fixed.
- [ ] **TEST-02** regression coverage to lock the fixes in place.

**Success Criteria:**
- All CLI commands fully functional (no stubs)
- Comprehensive test suite with >80% coverage
- Basic performance optimization complete
- Core security features validated

### Phase 2: Production Ready (8-10 weeks) - Target: 95% Production Ready

**User Experience Enhancements:**
- [ ] **CAGE-04**: Harden in-place workflows with multi-layer safety prompts and recovery artefacts.
- [ ] **CAGE-05**: Expand progress/telemetry once core bug fixes land (spinner, bar, byte styles).
- [ ] **CAGE-06**: Deliver layered configuration support (`~/.cagerc`, project overrides, env merging).
- [ ] **CAGE-07**: Ship RageAdapter as a first-class backend option.
- [ ] **TEST-03**: Establish proxy/PTy regression coverage (skip gracefully on unsupported platforms).
- [ ] Documentation refresh accompanying each shipped task (README + `docs/tech/*`).

**Success Criteria:**
- Production-grade error handling and recovery
- Cross-platform compatibility verified
- Security audit passed
- Complete documentation suite

### Phase 3: Enhanced Features (12+ weeks) - Target: Feature Complete

**Advanced Functionality:**
- [ ] **RageAdapter Implementation**
  - Integration with rage crate as alternative backend
  - Performance comparison with Age binary
  - Feature parity validation
  - Fallback mechanism implementation

- [ ] **Authority Chain Integration**
  - Multi-key encryption support
  - Role-based access control
  - Key delegation and revocation
  - Authority hierarchy management

- [ ] **Performance and Scalability**
  - Streaming encryption for large files (>4GB)
  - Memory-mapped file processing
  - Parallel processing for batch operations
  - Performance profiling and optimization

- [ ] **Enterprise Features**
  - Compliance logging (SOX, HIPAA, PCI-DSS)
  - Hardware security module support
  - Key escrow and recovery mechanisms
  - Monitoring and metrics export

- [ ] **Extensibility**
  - Plugin system for custom adapters
  - API for external integrations
  - Docker container optimization
  - CI/CD integration helpers

**Success Criteria:**
- Enterprise-grade feature set
- Extensible architecture
- Performance benchmarks met
- Compliance requirements satisfied

---

## Quality of Life Improvements

### High Impact UX Enhancements

**Interactive Features:**
- Secure passphrase prompting without command-line exposure
- Real-time progress bars with ETA and transfer rates
- Smart error messages with specific recovery steps
- Operation confirmation for destructive actions

**CLI Ergonomics:**
- Shell completion scripts for major shells
- Configuration profiles for different environments
- Command aliases and shortcuts
- Environment variable support for common options

**Operational Features:**
- Detailed operation logging with timestamps
- Performance metrics and timing information
- Memory usage reporting and optimization
- Integration with system logging (syslog, journald)

### Developer Experience Improvements

**Testing and Validation:**
- Mock Age binary for unit testing
- Performance benchmarking suite
- Memory leak detection
- Cross-platform automated testing

**Documentation:**
- API documentation with examples
- Architecture decision records (ADRs)
- Integration examples for common use cases
- Video tutorials and walkthroughs

---

## Technical Debt and Maintenance

### Code Quality Improvements

**Current Issues:**
- Unused imports causing compilation warnings
- Some placeholder implementations need cleanup
- Test references to removed padlock functionality
- Inconsistent error message formatting

**Maintenance Tasks:**
- [ ] Clean up unused imports across codebase
- [ ] Standardize error message formatting
- [ ] Update all test references from padlock to cage
- [ ] Implement proper logging levels
- [ ] Add comprehensive inline documentation

### Security Hardening

**Security Enhancements:**
- [ ] Secure memory handling for passphrases
- [ ] Additional injection attack prevention
- [ ] File system permission validation
- [ ] Network dependency security assessment
- [ ] Third-party dependency security audit

---

## Success Metrics

### Phase 1 Targets
- **Functionality:** 85% of CLI commands fully implemented
- **Testing:** >80% code coverage with integration tests
- **Performance:** Basic operations complete in <2s for small files
- **Security:** All major attack vectors mitigated

### Phase 2 Targets
- **Usability:** Interactive mode fully functional
- **Compatibility:** Windows, macOS, Linux support verified
- **Documentation:** Complete user guide and API docs
- **Performance:** Large file operations optimized

### Phase 3 Targets
- **Features:** All advanced functionality implemented
- **Enterprise:** Compliance and security requirements met
- **Performance:** Benchmark targets achieved
- **Extensibility:** Plugin system operational

---

## Risk Assessment

### High Risk Items
- **Age Binary Dependencies:** Changes to Age CLI could break automation
- **Platform Compatibility:** Windows TTY automation complexity
- **Performance:** Large file operations may require streaming architecture
- **Security:** Passphrase handling requires careful memory management

### Mitigation Strategies
- Comprehensive integration testing with multiple Age versions
- Platform-specific CI/CD pipelines for validation
- Early performance testing with large files
- Security audit before production deployment

---

## Contributing Guidelines

### Development Priorities
1. **P0 Issues:** Critical for basic functionality
2. **P1 Issues:** Important for user experience
3. **P2 Issues:** Nice-to-have enhancements
4. **P3 Issues:** Future considerations

### Code Standards
- All new code must include comprehensive tests
- Security features require security review
- Performance-critical code needs benchmarking
- Breaking changes require deprecation notices

---

**Next Review Date:** 2025-10-13
**Roadmap Owner:** Cage Development Team
**Status Updates:** Monthly milestone reviews
