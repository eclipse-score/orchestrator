# Component Integration Tests

For general information check [main README.md file](../README.md).

## Setup

Create `venv`, activate and install dependencies:

```bash
python -m venv <REPO_ROOT>/.venv
source <REPO_ROOT>/.venv/bin/activate
pip install -r <REPO_ROOT>/tests/test_cases/requirements.txt
```

## Usage

Set current working directory to the following:

```bash
cd <REPO_ROOT>/tests/test_cases
```

### Run tests

Bazel:

```bash
bazel test //tests/test_cases:cit
```

Basic run:

```bash
pytest .
```

Run with additional flags:

```bash
pytest -vsx . -k <PATTERN> --build-scenarios
```

- `-v` - increase verbosity.
- `-s` - show logs (disable capture).
- `-k <PATTERN>` - run tests matching the pattern.
- `--build-scenarios` - build Rust test scenarios before execution.

Run tests repeatedly:

```bash
pytest . --count <VALUE> --repeat-scope session -x
```

- `--count <VALUE>` - number of repeats.
- `--repeat-scope session` - scope of repeat.
- `-x` - exit on first error.

Refer to `pytest` manual for `pytest` specific options.
Refer to `conftest.py` for test suite specific options.

### Create HTML report

To generate HTML report use:

```bash
pytest -v . --build-scenarios --self-contained-html --html report.html --traces <VALUE>
```

- `--self-contained-html` - generate self contained HTML file.
- `--html report.html` - HTML report output path.
- `--traces <VALUE>` - verbosity of traces in output and HTML report - "none", "target" or "all".

> Traces are collected using `stdout`.
> Setting `--capture` flag (including `-s`) might cause traces to be missing from HTML report.

## Standalone execution of test scenarios

Test scenarios can be run independently from `pytest`.

Set current working directory to the following:

```bash
cd <REPO_ROOT>/tests/test_scenarios/rust
```

List all available scenarios:

```bash
cargo run -- --list-scenarios
```

Run specific test scenario:

```bash
cargo run -- --name <TEST_GROUP>.<TEST_SCENARIO> --input <TEST_INPUT>
```

Example:

```bash
cargo run -- --name basic.only_shutdown --input '{"runtime": {"task_queue_size": 256, "workers": 1}}'
```

Run test scenario executable directly:

```bash
<REPO_ROOT>/target/debug/test_scenarios --name basic.only_shutdown --input '{"runtime": {"task_queue_size": 256, "workers": 1}}'
```

### Using Bazel

Bazel handles all setup steps like environment, rebuilding test scenarios by itself.
All Component Integration Tests can be executed with:

```bash
bazel test //tests/test_cases:cit
```

Run all tests 5 times to check for sporadic errors:

```bash
bazel test //tests/test_cases:cit_repeat
```

Run Test Scenarios:

```bash
bazel run //tests/test_scenarios/rust:test_scenarios -- --help
```
