# NW Bootloader

A bootloader for the Numworks n0110 calculator written in Rust.

## Setup

To setup the developement environment follow the instructions in
[`rustworks/SETUP.md`](https://github.com/nw-rs/)


## STLink

If you have an STLink debugger (I am using the STLink V3SET) you can flash
using one of the following commands:

### Flash

Specify the chip manually:

```zsh
cargo flash --chip=stm32f730V8Tx
```

Let `cargo-make` specify the chip for you:

```zsh
cargo make flash
```

### Debug

Using `cargo-embed` (recommended):

```zsh
cargo embed
```

Using `probe-rs`:

```zsh
cargo run
```

## DFU flash

Complete setup, plug in your calculator and put it into DFU mode (press 6 and
the reset button on the back at the same time), then run the following command:

```zsh
cargo make dfu
```

