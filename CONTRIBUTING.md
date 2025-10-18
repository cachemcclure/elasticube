# Contributing to ElastiCube

Thank you for your interest in contributing to ElastiCube! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Commit Messages](#commit-messages)

---

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful, inclusive, and constructive in all interactions.

---

## Getting Started

### Prerequisites

- **Rust**: Version 1.90.0 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Python** (optional, for Python bindings): Version 3.8 or later
  ```bash
  # Install maturin for building Python bindings
  pip install maturin
  ```

- **Git**: For version control

---

## Development Setup

1. **Fork and Clone**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/elasticube.git
   cd elasticube
   ```

2. **Build the Project**:
   ```bash
   cd elasticube-core
   cargo build --features all-sources
   ```

3. **Run Tests**:
   ```bash
   # Run all tests
   cargo test --features all-sources

   # Run specific test
   cargo test test_name --features all-sources

   # Run with output
   cargo test --features all-sources -- --nocapture
   ```

4. **Run Examples**:
   ```bash
   # Run a specific example
   cargo run --example basic_cube_building

   # Run object storage example (requires object-storage feature)
   cargo run --example object_storage_demo --features object-storage
   ```

5. **Build Documentation**:
   ```bash
   cargo doc --no-deps --features all-sources --open
   ```

---

## How to Contribute

### Reporting Bugs

1. **Search existing issues** to avoid duplicates
2. **Create a new issue** with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs. actual behavior
   - Rust version, OS, and relevant environment details
   - Minimal code example demonstrating the issue

### Suggesting Features

1. **Check existing feature requests** and discussions
2. **Open an issue** describing:
   - The problem your feature would solve
   - Proposed API or behavior
   - Alternative solutions considered
   - Potential impact on existing code

### Contributing Code

1. **Find an issue** to work on or create one
2. **Comment on the issue** to claim it and discuss approach
3. **Fork and create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** following our coding standards
5. **Add tests** for your changes
6. **Update documentation** as needed
7. **Submit a pull request**

---

## Pull Request Process

### Before Submitting

- [ ] Code compiles without warnings: `cargo build --features all-sources`
- [ ] All tests pass: `cargo test --features all-sources`
- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy --features all-sources -- -D warnings`
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] New tests cover your changes

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Motivation
Why is this change needed? What problem does it solve?

## Changes
- List of specific changes made

## Testing
How were these changes tested?

## Breaking Changes
List any breaking changes (if applicable)

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] No new warnings
```

### Review Process

1. At least one maintainer review is required
2. All CI checks must pass
3. Requested changes must be addressed
4. Maintainer will merge once approved

---

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for automatic formatting
- Run `cargo clippy` to catch common mistakes
- Prefer explicit types in public APIs
- Use `&str` for borrowed string parameters, `impl Into<String>` for owned

### Naming Conventions

- **Types**: `PascalCase` (e.g., `ElastiCube`, `QueryBuilder`)
- **Functions/Methods**: `snake_case` (e.g., `add_dimension`, `load_csv`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_BATCH_SIZE`)
- **Modules**: `snake_case` (e.g., `cube`, `query`, `sources`)

### Error Handling

- Use `Result<T, Error>` for fallible operations
- Provide context in error messages
- Use `thiserror` for error types
- Avoid `unwrap()` and `expect()` in library code (tests are ok)

### Documentation

Every public item must have documentation:

```rust
/// Brief one-line description
///
/// Longer description if needed, explaining behavior and use cases.
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function can return an error
///
/// # Examples
/// ```rust,ignore
/// let result = function(param)?;
/// ```
pub fn function(param: Type) -> Result<ReturnType> {
    // ...
}
```

### Code Organization

- Keep functions focused and small
- Group related functionality in modules
- Use `pub(crate)` for internal APIs
- Mark items `#[non_exhaustive]` where appropriate
- Implement common traits (`Debug`, `Clone`, etc.) where sensible

---

## Testing Guidelines

### Unit Tests

Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = create_input();

        // Act
        let result = function(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Place integration tests in `elasticube-core/tests/`:

```rust
// tests/my_integration_test.rs
use elasticube_core::*;

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test complete user workflows
}
```

### Test Organization

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test public API and workflows
- **Property tests**: Use quickcheck for property-based testing
- **Benchmarks**: Place in `benches/` directory

### Test Coverage

- Aim for high coverage of public APIs
- Test edge cases and error conditions
- Include both success and failure scenarios
- Test with different feature flag combinations

---

## Documentation

### Types of Documentation

1. **API Documentation** (rustdoc):
   - Describe what each item does
   - Include examples for public APIs
   - Document panics, errors, and edge cases

2. **User Guide** (`docs/USER_GUIDE.md`):
   - High-level tutorials and concepts
   - Common use cases and patterns
   - Performance tuning tips

3. **Examples** (`examples/`):
   - Runnable code demonstrating features
   - Cover common use cases
   - Include comments explaining the code

### Documentation Standards

- Use proper markdown formatting
- Include code examples that compile
- Keep examples minimal and focused
- Update docs when changing APIs
- Link to related items with `[`item`]`

---

## Commit Messages

### Format

```
type(scope): brief description

Longer explanation if needed. Wrap at 72 characters.
Explain what and why, not how.

Fixes #123
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code restructuring without behavior change
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, etc.

### Examples

```
feat(query): add support for window functions

Implements window functions (ROW_NUMBER, RANK, etc.) using
DataFusion's window function support.

Fixes #45
```

```
fix(builder): validate schema before building cube

Previously, invalid schemas would only fail during query execution.
Now we validate during build() to provide better error messages.

Fixes #78
```

---

## Feature Flags

When adding new optional dependencies:

1. **Add to `Cargo.toml`**:
   ```toml
   [dependencies]
   new-dep = { version = "1.0", optional = true }

   [features]
   my-feature = ["new-dep"]
   all-sources = [..., "my-feature"]
   ```

2. **Use conditional compilation**:
   ```rust
   #[cfg(feature = "my-feature")]
   pub use sources::my_source::MySource;
   ```

3. **Document in lib.rs**:
   ```rust
   /// My feature types
   ///
   /// Only available with the `my-feature` feature:
   /// ```toml
   /// elasticube-core = { version = "0.2", features = ["my-feature"] }
   /// ```
   #[cfg(feature = "my-feature")]
   pub use ...;
   ```

4. **Test with the feature**:
   ```bash
   cargo test --features my-feature
   ```

---

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Update `PROJECT_CHECKLIST.md` to mark phase complete
4. Run full test suite: `cargo test --features all-sources`
5. Build documentation: `cargo doc --no-deps --features all-sources`
6. Create git tag: `git tag -a v0.2.0 -m "Release v0.2.0"`
7. Push tag: `git push origin v0.2.0`
8. Publish to crates.io: `cargo publish` (when ready)

---

## Getting Help

- **Documentation**: Check `docs/` directory and rustdoc
- **Examples**: Look at `examples/` for usage patterns
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions

---

## License

By contributing to ElastiCube, you agree that your contributions will be licensed under either:

- MIT License
- Apache License, Version 2.0

at the option of the user.

---

Thank you for contributing to ElastiCube! 🚀
