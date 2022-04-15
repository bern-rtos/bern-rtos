# Hardware Integration Tests

Hardware integrations tests for different microcontrollers.

## Prerequisites

- rust nightly toolchain
- `probe-run`
    - enable probe-run in `.cargo/conf`
- a serial terminal

## Usage

Compile, program and run a test, e.g.:
```sh
cargo test --test=arm_cm4-task
```
