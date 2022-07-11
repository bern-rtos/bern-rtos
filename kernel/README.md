# `bern-kernel`

[![crates.io](https://img.shields.io/crates/v/bern-kernel)](https://crates.io/crates/bern-kernel)
[![docs.rs](https://docs.rs/bern-kernel/badge.svg)](https://docs.rs/bern-kernel)
[![book](https://img.shields.io/badge/web-kernel.bern--rtos.org-red.svg?style=flat&label=book&colorB=d33847)](https://kernel.bern-rtos.org/)

<!-- cargo-rdme start -->

Bern RTOS preemptive real-time kernel for microcontrollers written in Rust.

## Features
-

## Documentation
- [Bern RTOS Kernel Book](https://kernel.bern-rtos.org/)
- [API Documentation](https://docs.rs/bern-kernel/)

## Semantic Versioning
This project follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

Currently the version is below 1.0.0 meaning that everything is very much unstable
and the API could change completely.

## Cargo features
- `time-slicing`: A task runs at max for one system tick period if there are
  other tasks of equal priority

## License
- [MIT License](LICENSE.md)

## Quickstart Guide

<!-- cargo-rdme end -->
