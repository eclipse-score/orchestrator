# score_orchestrator

Repository for **Orchestration framework**

[![Nightly CIT](../../actions/workflows/tests.yml/badge.svg)](../../actions/workflows/tests.yml)
[![Nightly CIT (Bazel)](../../actions/workflows/tests_bazel.yml/badge.svg)](../../actions/workflows/tests_bazel.yml)


## Feature status and roadmap

* [Orchestration](src/orchestration/doc/features.md)

## Continuous Integration Nightly Tests

This repository includes two GitHub Actions workflows for component integration testing:

### Component Integration Tests (Cargo-based)
- **Schedule**: Runs nightly at 1:45 UTC
- **Build System**: Uses Cargo for Rust components
- **Testing**: Executes Python test suite with pytest
- **Nightly Mode**: Runs tests 20 times with `--count 20 --repeat-scope session` for enhanced
  reliability testing
- **Triggers**: Push/PR to main/development branches, and scheduled nightly runs

### Component Integration Tests (Bazel-based)
- **Schedule**: Runs nightly at 1:15 UTC
- **Build System**: Uses Bazel for all components
- **Testing**: Builds Rust test scenarios and runs Python component integration tests
- **Nightly Mode**: Uses `cit_repeat` target for flake detection
- **Triggers**: Push/PR to main/development branches, and scheduled nightly runs

Monitor via the status badges above and the Actions tab

## Setup

### System dependencies

```bash
sudo apt-get update
sudo apt-get install -y curl build-essential protobuf-compiler libclang-dev git python3-dev python-is-python3 python3-venv
```

### Rust installation

[Install Rust using rustup](https://www.rust-lang.org/tools/install)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

### Bazel installation

[Install Bazel using Bazelisk](https://bazel.build/install/bazelisk)

```bash
curl --proto '=https' -sSfOL https://github.com/bazelbuild/bazelisk/releases/download/v1.26.0/bazelisk-amd64.deb
dpkg -i bazelisk-amd64.deb
rm bazelisk-amd64.deb
```

Correct Bazel version will be installed on first run, based on `bazelversion` file.

## Build

List all targets:

```bash
bazel query //...
```

Build selected target:

```bash
bazel build <TARGET_NAME>
```

Build all targets:

```bash
bazel build //...
```

## Build for QNX8

### Preparations

Please follow
[Where to obtain the QNX 8.0 SDP](https://github.com/eclipse-score/toolchains_qnx?tab=readme-ov-file#where-to-obtain-the-qnx-80-sdp)
to get access to QNX8 and how to setup QNX8 for `S-CORE`. In above link You will also find an
instructions how to replace SDP in case You need to use other one (ie HW specific).


### Building
```bash
./scripts/build_qnx8.sh BAZEL_TARGET (default is //src/...)
```

## Run

List all binary targets, including examples:

```bash
bazel query 'kind(rust_binary, //src/...)'
```

> Bazel is not able to distinguish between examples and regular executables.

Run selected target:

```bash
bazel run <TARGET_NAME>
```

## Test

List all test targets:

```bash
bazel query 'kind(rust_test, //...)'
```

Run all tests:

```bash
bazel test //...
```

Run unit tests (tests from `src/` directory):

```bash
bazel test //src/...
```

Run selected test target:

```bash
bazel test <TARGET_NAME>
```

## Cargo-based operations

Please use **Bazel** whenever possible. However for easy development we provide a build based on
**Cargo**

### Build with Cargo

It's recommended to use `cargo xtask`. It has the advantage of using separate build directories for
each task causing no need to rebuild from scratch for different types of checks

Build using `xtask` - debug and release:

```bash
cargo xtask build
cargo xtask build:release
```

Build using `cargo` directly:

```bash
cargo build
```

### Run with Cargo

List all examples:

```bash
cargo xtask run --example
```

Using `cargo xtask`:

```bash
cargo xtask run --example <EXAMPLE_NAME>
```

### Run unit tests with Cargo

Using `cargo xtask`:

```bash
cargo xtask build:test --lib
```
