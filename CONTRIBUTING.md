# Contributing to bevy-test-suite

Thank you for your interest in contributing to bevy-test-suite! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and constructive. We're all here to make Bevy testing better.

## Getting Started

1. **Fork and Clone**
   ```bash
   git clone https://github.com/noahsabaj/bevy-test-suite.git
   cd bevy-test-suite
   ```

2. **Build and Test**
   ```bash
   cargo build --all-features
   cargo test --all-features
   cargo run --example dual_approach
   ```

3. **Check Your Work**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## Development Setup

### System Dependencies (Linux/Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y \
  libasound2-dev \
  libudev-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev
```

### Minimum Rust Version

This crate requires Rust 1.75 or newer (following Bevy 0.16's MSRV).

## Contributing Guidelines

### Dual Approach Philosophy

This crate deliberately offers TWO testing approaches:
1. `#[bevy_test]` attribute macro (imperative)
2. `test_scenario!` and related macros (declarative)

**Both approaches are equally important.** New features should consider how they work with both paradigms.

### Code Style

- Use `rustfmt` for all code formatting
- All Clippy warnings are errors in CI
- Keep examples clear and educational
- Document public APIs thoroughly

### Adding New Features

1. **New Macros**: Place in appropriate module (`attribute.rs`, `scenario.rs`, etc.)
2. **Parse Structures**: Define clear `Parse` implementations
3. **Examples**: Add examples showing the feature in action
4. **Tests**: Include tests demonstrating correct behavior
5. **Documentation**: Update relevant docs and README

### Common Parsing Patterns

For custom keywords in proc macros:
```rust
// CORRECT - parse as Ident
let ident: Ident = content.parse()?;
if ident != "given" {
    return Err(syn::Error::new(ident.span(), "Expected 'given'"));
}

// WRONG - don't use Token!
content.parse::<Token![given]>()?;  // This won't work!
```

### Entity Reference Convention

In declarative macros, entities are referenced by creation order:
- First entity: `entity_0`
- Second entity: `entity_1`
- And so on...

## Submitting Changes

### Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clear, concise commit messages
   - Keep commits focused and atomic
   - Update CHANGELOG.md in the Unreleased section

3. **Run CI checks locally**
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo doc --no-deps
   ```

4. **Push and create PR**
   - Use a clear PR title describing the change
   - Reference any related issues
   - Include examples of the feature in use

### Commit Messages

Follow conventional commit format:
```
feat: add new assertion macro for component state
fix: correct entity reference parsing in test_scenario
docs: update examples for time manipulation
test: add coverage for edge cases in system testing
chore: update dependencies
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_name

# Run examples
cargo run --example dual_approach
```

### Writing Tests

- Test both success and failure cases
- Include edge cases
- Use descriptive test names
- Keep tests focused and isolated

## Documentation

### Code Documentation

- Document all public APIs
- Include examples in doc comments
- Explain complex algorithms
- Note any limitations or assumptions

### Example Format

```rust
/// Creates a test scenario with given/when/then structure.
///
/// # Example
///
/// ```
/// test_scenario!(player_damage {
///     given: {
///         entities: [
///             Player { health: 100, armor: 10 }
///         ]
///     },
///     when: {
///         event: DamageEvent { target: entity_0, amount: 30 }
///     },
///     then: {
///         entity_0.get::<Player>().unwrap().health == 70
///     }
/// });
/// ```
```

## Release Process

Releases are automated via GitHub Actions when a version tag is pushed:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`:
   - Move Unreleased items to new version section
   - Add date to version header
3. Commit changes: `git commit -m "Release v0.2.0"`
4. Tag release: `git tag -a v0.2.0 -m "Release v0.2.0"`
5. Push: `git push origin main v0.2.0`

## Architecture Decisions

### Why Proc Macros?

- Better error messages with span-accurate reporting
- Complex parsing of given/when/then structures
- Superior IDE support
- Compile-time type checking

### TestApp Trait Pattern

The `bevy_test_utils!()` macro generates the TestApp trait to provide a clean API while avoiding circular dependencies.

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Join the discussion in relevant Bevy community channels

## Recognition

Contributors will be acknowledged in release notes and the project README.

Thank you for helping make Bevy testing better!
