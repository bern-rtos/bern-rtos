# Bern RTOS

Monorepo containing all Bern RTOS code.

For the core component open the `kernel/` directory.


## Usage

### Build/run for an embedded device

```bash
cargo build --example=nucleo_f446_dev
cargo run --example=nucleo_f446_dev
```

### Run PC unit and integration tests

```bash
cargo test --package=bern-kernel --target=x86_64-unknown-linux-gnu
```

### Run hardware integration tests

```bash
cargo test --test=arm-cm4-thread
```
