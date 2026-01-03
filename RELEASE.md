# Release Process

This document describes the process for creating a signed release of oracle.omen.

## Pre-Release Checklist

- [ ] All tests pass: `cargo test --all-features`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --no-deps --all-features`
- [ ] Determinism tests pass: `cargo test --test determinism`
- [ ] Replay identity tests pass: `cargo test --test replay_identity`
- [ ] Fuzzing has run (sanity check): `cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=60`
- [ ] CHANGELOG.md updated with version
- [ ] Version numbers updated in all Cargo.toml files
- [ ] git tag created: `git tag -s v0.1.0`

## Release Artifacts

### 1. Source Archive

```bash
git archive v0.1.0 --prefix=oracle-omen-0.1.0/ -o oracle-omen-0.1.0.tar.gz
```

### 2. Release Bundle

The release bundle contains:

```
oracle-omen-0.1.0-bundle.tar.gz
├── oracle-omen-0.1.0.tar.gz          # Source archive
├── oracle-omen-0.1.0.tar.gz.blake3   # BLAKE3 hash
├── oracle-omen-0.1.0.tar.gz.sig      # Signature (Ed25519)
├── SHA256SUMS                         # SHA256 checksums
├── SHA256SUMS.sig                     # Signature of checksums
├── RELEASE_NOTES.md                   # Release notes
└── verify.sh                          # Verification script
```

### 3. Build Verification Script

```bash
#!/bin/bash
set -euo pipefail

echo "Oracle Omen Release Verification"
echo "================================="
echo ""

# Check for required tools
for tool in tar b3sum ssh-keygen; do
    if ! command -v $tool &> /dev/null; then
        echo "Error: $tool not found"
        exit 1
    fi
done

# Verify BLAKE3 hash
echo "Checking BLAKE3 hash..."
b3sum --check oracle-omen-0.1.0.tar.gz.blake3

# Verify signature (requires signer's public key)
echo "Verifying signature..."
# Implementation depends on signature format

# Verify SHA256SUMS
echo "Verifying SHA256 checksums..."
sha256sum --check SHA256SUMS

echo ""
echo "All verifications passed!"
echo "Archive: oracle-omen-0.1.0.tar.gz"
echo "Version: 0.1.0"
```

## Creating a Release

### Step 1: Update Version Numbers

Update version in all `Cargo.toml` files:

```toml
[package]
name = "oracle_omen_core"
version = "0.1.0"  # Update this
```

### Step 2: Update CHANGELOG

Move unreleased entries to new version section.

### Step 3: Tag and Sign

```bash
# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Sign the tag (optional but recommended)
git tag -s v0.1.0 -m "Release v0.1.0"

# Push tag
git push origin v0.1.0
```

### Step 4: Generate Artifacts

```bash
# Create source archive
git archive v0.1.0 --prefix=oracle-omen-0.1.0/ -o oracle-omen-0.1.0.tar.gz

# Generate BLAKE3 hash
b3sum oracle-omen-0.1.0.tar.gz > oracle-omen-0.1.0.tar.gz.blake3

# Sign the hash (Ed25519)
# Using oracle_omen_patches signing key
oracle-omen sign oracle-omen-0.1.0.tar.gz.blake3

# Generate SHA256SUMS
sha256sum oracle-omen-0.1.0.tar.gz > SHA256SUMS

# Sign SHA256SUMS
oracle-omen sign SHA256SUMS
```

### Step 5: Create Release Bundle

```bash
tar czf oracle-omen-0.1.0-bundle.tar.gz \
    oracle-omen-0.1.0.tar.gz \
    oracle-omen-0.1.0.tar.gz.blake3 \
    oracle-omen-0.1.0.tar.gz.sig \
    SHA256SUMS \
    SHA256SUMS.sig \
    RELEASE_NOTES.md \
    verify.sh
```

### Step 6: Upload to GitHub

```bash
gh release create v0.1.0 \
    ./oracle-omen-0.1.0-bundle.tar.gz \
    ./oracle-omen-0.1.0.tar.gz \
    --notes-file RELEASE_NOTES.md \
    --title "Oracle Omen v0.1.0"
```

## Release Notes Template

```markdown
# Oracle Omen v0.1.0

## Downloads

| File | Hash | Signature |
|------|------|-----------|
| [oracle-omen-0.1.0.tar.gz](oracle-omen-0.1.0.tar.gz) | `blake3:...` | [sig](oracle-omen-0.1.0.tar.gz.sig) |
| [oracle-omen-0.1.0-bundle.tar.gz](oracle-omen-0.1.0-bundle.tar.gz) | `blake3:...` | - |

## Verification

```bash
# Download and extract
wget https://github.com/user/oracle.omen/releases/download/v0.1.0/oracle-omen-0.1.0-bundle.tar.gz
tar xzf oracle-omen-0.1.0-bundle.tar.gz

# Verify
bash verify.sh
```

## What's New

### Added
- Initial release of oracle.omen
- Deterministic event logging with BLAKE3
- Replay engine with divergence detection
- Capability-based access control
- Planning DSL and DAG execution
- Memory CRDT with provenance
- Policy language and engine
- Self-evolution patch system
- WASM sandbox with fuel limits
- CLI with all commands

### Security
- All tools require explicit capabilities
- WASM sandbox isolates untrusted code
- Patch signatures prevent unauthorized modification

### Determinism
- All hashing uses BLAKE3
- All collections use BTreeMap
- LogicalTime replaces system time
- No unseeded randomness

## Known Issues

None.

## Upgrade Guide

This is the initial release. No upgrade path needed.

## Documentation

- [README](../README.md)
- [ARCHITECTURE](../docs/ARCHITECTURE.md)
- [AUDIT_GUIDE](../docs/AUDIT_GUIDE.md)
- [Full Documentation](../docs/)

## License

GPL v3
```

## Post-Release

- [ ] Announce on relevant channels
- [ ] Update website (if applicable)
- [ ] Create GitHub milestone for next version
- [ ] Close all issues included in release
- [ ] Archive release artifacts for long-term storage

## Signing Key

Releases are signed using Ed25519.

Public key:
```
# Oracle Omen Release Signing Key
# This key signs all oracle.omen releases
```

To verify signatures:
```bash
# Add public key to your keyring
# Then verify
oracle-omen verify <file> <signature>
```

## Reproducible Builds

To verify reproducible builds:

```bash
# Build from source
cargo build --release

# Compare binaries
diff -q target/release/oracle-omen <downloaded-binary>
```

## Support

For issues:
- GitHub Issues: https://github.com/staticpayload/oracle.omen/issues
- Security: security@oracleomen.example.com
```
