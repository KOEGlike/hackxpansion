# Writing your first driver

This guide will teach you how to create a simple driver for a module.

## Experience needed

To understand this guide, you should have lite programming experience and have read and completed the [First Card](./first-card) guide. If you don't understand something google is your best friend. If you have any further questions just ask in `#hackxpansion`

## Table of contents

:3

## Download software

- **Code Editor** This is needed to write your driver, I recommend VS Code or Zed.
- **Rust** This is the language that the firmware for hackxpansion is written in. Can be downloaded from [here](https://rust-lang.org/learn/get-started/)
- **Rust target for RP2354** The basic install of Rust doesn't contain the compilation target for the RP2354. Install by running `rustup target add thumbv8m.main-none-eabihf`.
- **Picotool** This is needed for flashing firmware onto the hardware. Download from your package manager or from [here](https://github.com/raspberrypi/pico-sdk-tools/releases)
- **Git:** This is needed to publish your project and download resources. Can be downloaded from your package manager or [here](https://git-scm.com/install/)

## Setup your project

In the root of your repo run `cargo new firmware --lib`, this will create a new rust library crate(project)

Your project hierarchy look something like this:

```
my-module/
├─ firmware/
│  ├─ Cargo.toml
│  ├─ src/
│  │  ├─ lib.rs
├─ pcb/
```

- `Cargo.toml` This is the file where you define your project dependencies and other metadata for your crate like it's name and version and description. It is just like `package.json` for JavaScript
- `lib.rs` This is file the main entry point for your driver library

## Include dependencies

You have to add `xpanse-api` as dependency, simply run `cargo add xpanse-api`. This adds all the stuff that is needed for creating a driver to the project.

This command added a new entry in `Cargo.toml`

## Creating basic driver
