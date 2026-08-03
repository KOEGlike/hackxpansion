# Writing your first driver

This guide will teach you how to create a simple driver for a module.

## Experience needed

To understand this guide, you should have lite programming experience and have read and completed the [First Card](./first-card) guide. If you don't understand something google is your best friend. If you have any further questions just ask in `#hackxpansion`

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

In this section I will explain a basic driver that adds two buttons to the global registry, so apps can use them later.

```rust
#![no_std]

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, B, pin_button},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
};

// Each driver is a struct, that impls the Driver and DriverMeta trait
pub struct TwoButtonDriver;

// This trait allows the main firmware to know when should it load this driver,
// on what resistor combination
impl DriverMeta for TwoButtonDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K6,
        md1: ModuleDetectResistor::R1K5,
    };
}

// This is where the actual business logic happens
impl<G: BankPins> Driver<G> for TwoButtonDriver {
    async fn create(
        // Each driver gets a bank of GPIO pins it can work with, GPIO0 through GPIO10
        gpio_bank: GpioBank<G>,
        // They also get some data about which slot they are
        slot: ModuleSlot,
        // They also get access to the registry, where they can add peripherals which
        // apps will be able to use, I'll explain this later more in depth
        registry: &mut Registry,
        // They also get access to a bus allocator, since the RP2354 only
        // has 2 I2C, 2 SPI and 2 UART peripherals not every module can
        // get its own hardware UART for example, I'll also explain this later more in depth
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        // Here we add an A button to the registry
        registry.register(
            slot,
            TwoButtonDriver::ID,
            pin_button::<A>(gpio_bank.gpio0.into()),
        );

        // Here we add an B button to the registry
        registry.register(
            slot,
            TwoButtonDriver::ID,
            pin_button::<B>(gpio_bank.gpio1.into()),
        );

        // we don't use any busses
        let _ = bus_allocator;

        Ok(())
    }
}
```

### The Registry

This is a structure where drivers can add `Resources` to, which are things like buttons, screens, knobs etc. And apps can take these to use them.

One driver can add as many resource as it wants to the registry.

In our case both resources are buttons, created by `pin_button`, which returns a `Box<dyn Button<R>>` where `R` is the role of the button, like `A`, `B`, `X`, `Y`, and more.

But the registry accepts any type, so you can create your own resource types and APIs, which apps can consume.
If you want to do this with a custom resource types, instead of a preexisting interface type, look at the implementations of them.

#### Groups

Hardware can be use for multiple things, like a button could be used as `A`, `B`, `X`, `Y`, etc. but currently, if you added a piece of hardware to the registry, you can't add it again, even if you used an Arc Mutex the app could take both resource using the same hardware and that might cause some issues.

Here are where groups come in, you can add multiple resources to a group, and if one gets taken by an app, all other members of the group become unavailable for apps to use.

In this example we register 4 groups, each having two buttons, if one is taken of two, the other is not available anymore to the apps.

If you want to do this with a custom resource types, instead of a preexisting interface type, look at the implementations of them.

```rust
 let (button_a, button_down) = aliased_pin_buttons::<A, Down>(button_a_pin.into());
        let (button_b, button_right) = aliased_pin_buttons::<B, Right>(button_b_pin.into());
        let (button_x, button_left) = aliased_pin_buttons::<X, Left>(button_x_pin.into());
        let (button_y, button_up) = aliased_pin_buttons::<Y, Up>(button_y_pin.into());

        registry
            .register_groups(
                slot,
                FourButtonDriver::ID,
                (
                    (button_a, button_down),
                    (button_b, button_right),
                    (button_x, button_left),
                    (button_y, button_up),
                ),
            )
            .map_err(|_| DriverError::InitFailed)?;
```

## Adding your driver to the firmware

After you finished your driver, you have to clone the [hackxpansion repo](https://github.com/KOEGlike/hackxpansion), go in the firmware folder, and add your driver crate as a dependency in the workspace [`Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/Cargo.toml), for now it can be a local path

After that you need to add your driver to [`load_driver.rs`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/xpanse/src/load_driver.rs), look at how other drives are added.

Build the project by running `cargo build`, if it compiles, publish you driver crate on [crates.io](https://crates.io), and swap your local path with your driver crate on crates io in the workspace [`Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/Cargo.toml) of the hackxpansion firmware, then make a PR to the repo with your newly added driver

# Work In Progress
