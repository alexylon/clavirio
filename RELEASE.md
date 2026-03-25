# Release Process Guide

This project uses **cargo-release** to automate the release workflow.

## Setup

Cargo-release is already installed. The configuration is in `release.toml`.

## Release Commands

### 1. Dry Run (Recommended First Step)
Preview what will happen without making changes:

```bash
cargo release patch
```

This shows:
- Version bump (0.1.17 → 0.1.18)
- Files to be modified
- Git operations
- Publishing steps

### 2. Perform Release
Execute the full release workflow:

```bash
cargo release patch --execute
```

This automatically:
1. ✅ Bumps version in `Cargo.toml`
2. ✅ Runs tests: `cargo test`
3. ✅ Runs formatting check: `cargo fmt --check`
4. ✅ Updates `CHANGELOG.md`
5. ✅ Publishes to crates.io
6. ✅ Creates release commit: `"Release v0.1.18"`
7. ✅ Creates Git tag: `v0.1.18`
8. ✅ Pushes commits and tags to remote

### 3. Release with Specific Version
Bump to a specific version (major, minor, patch):

```bash
# Patch release (0.1.17 → 0.1.18)
cargo release patch --execute

# Minor release (0.1.17 → 0.2.0)
cargo release minor --execute

# Major release (0.1.17 → 1.0.0)
cargo release major --execute
```

**Note:** Always specify a version level (patch/minor/major). Running `cargo release` without a level will try to re-release the current version.

### 4. Release Without Publishing to crates.io
If you want to skip publishing:

```bash
cargo release patch --execute --no-publish
```

### 5. Release Without Pushing to Git
For testing/staging:

```bash
cargo release patch --execute --no-push
```

## What Gets Updated

### Files Modified Automatically
- **Cargo.toml**: Version number
- **CHANGELOG.md**: New release section (if properly configured)
- **Git**: Creates commit and annotated tag

### Generated Release Commit
- Commit message: `"Release vX.Y.Z"`
- Includes: version bumps and CHANGELOG updates

### Created Git Tag
- Tag name: `vX.Y.Z` (e.g., `v0.1.18`)
- Tag message: `"Release vX.Y.Z"`
- Annotated tag for better tracking

## Before Release

Ensure:
1. ✅ All changes are committed: `git status`
2. ✅ Tests pass: `cargo test`
3. ✅ Code is formatted: `cargo fmt`
4. ✅ You have push access to the remote repository
5. ✅ crates.io credentials are configured (for publishing)

## crates.io Publishing Setup

If this is your first time publishing to crates.io:

```bash
# Login to crates.io (interactive)
cargo login

# This creates/updates ~/.cargo/credentials.toml
```

## Configuration

The release process is configured in `release.toml`:

- **changelog**: Automatically update CHANGELOG.md
- **git.push**: Push commits and tags to remote
- **git.tag-name**: Tag format (v{{version}})
- **publish**: Publish to crates.io

## Example Release Workflow

```bash
# Step 1: Verify nothing to commit
$ git status
On branch main
nothing to commit

# Step 2: Preview the release
$ cargo release --dry-run
Prepared release of clavirio[RELEASE.md](../clavirio/RELEASE.md) v0.1.18

# Step 3: Execute release
$ cargo release patch --execute
Releasing clavirio v0.1.18
- Bump version in Cargo.toml
- Run pre-release checks
- Update CHANGELOG.md
- Publish to crates.io
- Create release commit
- Create git tag v0.1.18
- Push to remote

# Step 4: Verify on GitHub
$ git tag
v0.1.17
v0.1.18  ← New tag

# Step 5: Verify on crates.io
# https://crates.io/crates/clavirio
```

## Troubleshooting

### "No commits since last release"
You need new commits since the last tag:
```bash
git log v0.1.17..HEAD
```

### "Failed to publish"
Check crates.io credentials:
```bash
cargo login
```

### "Uncommitted changes"
All changes must be committed before release:
```bash
git status
git add .
git commit -m "message"
```

### "Permission denied for push"
Ensure you have push access:
```bash
git push origin main  # Test push
```

## Advanced: Customize CHANGELOG Updates

To have cargo-release automatically update CHANGELOG.md with conventional commit messages, ensure your commit messages follow the format:

```
feat: add new feature
fix: fix bug
docs: update documentation
refactor: refactor code
```

Then release will automatically categorize them in CHANGELOG.md.

## Rollback Release

If something goes wrong:

```bash
# Undo the last commit
git reset --soft HEAD~1

# Delete the tag locally
git tag -d v0.1.18

# Delete the tag remotely
git push origin :refs/tags/v0.1.18
```

## References

- [cargo-release documentation](https://rust-lang.github.io/cargo-release/)
- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
