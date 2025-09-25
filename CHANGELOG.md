# Changelog

All notable changes to bevy-test-suite will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-09-25

### Added
- Initial release of bevy-test-suite
- `#[bevy_test]` attribute macro for reducing test boilerplate
- `test_scenario!` declarative macro for complex integration tests
- `test_system!` macro for isolated system testing
- `test_component!` macro for component behavior testing
- `bevy_test_utils!()` macro for generating TestApp trait and utilities
- MockWorld and MockInput builder patterns
- Rich assertion macros for Bevy testing
- Comprehensive examples demonstrating both imperative and declarative approaches
- Full documentation and usage guides

### Features
- Reduces Bevy test setup from 30-50 lines to 5-10 lines
- Dual-approach design: choose between familiar attribute syntax or powerful declarative macros
- Entity reference system for declarative tests
- Time manipulation utilities (advance by frames, seconds, or days)
- Zero runtime overhead - all macros expand at compile time
- Compatible with Bevy 0.16