# Writing your first driver

This guide will teach you how to create a simple driver for a module.

## Experience needed

To understand this guide, you should have lite programming experience and have read/completed the [First Card](./first-card) guide. If you don't understand something google is your best friend. If you have any further questions just ask in `#hackxpansion`

## Download software

- **Code Editor** This is needed to write your driver, I recommend VS Code or Zed.
- **Rust** This is the language that the firmware for hackxpansion is written in. Can be downloaded from [here](https://rust-lang.org/learn/get-started/)
- **Rust target for RP2354** The basic install of Rust doesn't contain the compilation target for the RP2354. Install by running `rustup target add thumbv8m.main-none-eabihf`.
- **Picotool** This is needed for flashing firmware onto the hardware. Download from your package manager or from [here](https://github.com/raspberrypi/pico-sdk-tools/releases)

## Crash course on Rust

Rust is a compiled low level language, which means it doesn't have a garbage collector and has direct access to memory, like C. But it also has high level language creature comforts, like a package manager and a robust type-system, like TypeScript.

One of the main features of Rust is the `Barrow Checker`, which prevents you from shooting yourself in the foot when manipulating memory. It enforces a set of rules on your code which eliminates a whole class of bugs, like buffer overflows, race conditions, etc.

This guide will not teach you Rust, there are already existing guides/tutorials which can explain the language far better than I could. Check out the [`Helpful Resources`](../helpful-resouces) guide for links to the rust book and other helpful stuff.

## Setup your rust project

In the root of your module repo run `cargo new firmware --lib`, this will create a new rust library crate(project)

Your project hierarchy should look something like this:

```
my-module-repo/
├─ firmware/
│  ├─ Cargo.toml
│  ├─ src/
│  │  ├─ lib.rs
├─ pcb/
```

- `Cargo.toml` This is the file where you define your project dependencies and other metadata for your crate(package) like it's name and version. It is just like `package.json` for JavaScript
- `lib.rs` This file is the main entry point for your driver library

## Include dependencies

You have to add `xpanse-api` as dependency, simply run `cargo add xpanse-api` in your firmware directory. This adds all the stuff that is needed for creating a basic driver to the project. This command added a new entry in `Cargo.toml`.

If your modules uses an IC, like an environmental sensor, you will need a driver for it, you will need to add it the same way as we did for `xpanse-api`. To find drives just search `YOUR IC driver rust`.

## Creating basic driver

In this section I will explain a basic driver that adds two buttons to the global registry(this is explained later), so apps can use them later.

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
        // They also get some data about which slot they are in
        slot: ModuleSlot,
        // They also get access to the registry, where they can add peripherals which
        // apps will be able to use. I'll explain this later more in depth
        registry: &mut Registry,
        // They also get access to a bus allocator, since the RP2354 only
        // has 2 I2C, 2 SPI and 2 UART peripherals, not every module can
        // get its own hardware UART for example. I'll also explain this later more in depth
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        // Here we add an A button to the registry
        registry.register(
            slot,
            TwoButtonDriver::ID,
            pin_button::<A>(gpio_bank.gpio0.into()),
        );

        // Here we add a B button to the registry
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

This is a structure where drivers can add `Resources` to, which are things like buttons, screens, sensors, knobs etc. And apps can take these and use them.

One driver can add as many resource as it wants to the registry.

In our case both resources are buttons, created with `pin_button`, which returns a `Box<dyn Button<R>>` where `R` is the role of the button, like `A`, `B`, `X`, `Y`, and more.

But the registry accepts any type, so you can create your own resource types and APIs, which apps can consume.
If you want to do this with a custom resource types, instead of a preexisting interface type(like Button), look at the implementations of these preexisting interfaces, and try to do something similar.

#### Groups

Hardware can be use for multiple things, like a button could be used as `A`, `B`, `X`, `Y`, etc. But currently, if you added a piece of hardware to the registry, you can't add it again, even if you used an Arc Mutex an app could take both resource using the same hardware and that might cause some issues.

Here are where groups come in. You can add multiple resources to a group, and if one gets taken by an app, all other members of the group become unavailable for the app to use.

In this example we register 4 groups, each having two buttons, if one is taken of the two in a group, the other is not available anymore to the app.

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

Adding a driver is really easy. Here are the steps:

1. Fork and clone the [hackxpansion repo](https://github.com/KOEGlike/hackxpansion) if you haven't already.
2. Go into the `firmware` folder.
3. Add your driver crate as a local workspace dependency under `# Drivers` in [`firmware/Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/Cargo.toml). The path should point to the `firmware` folder in your local module repo while you are testing it:

```toml
my-driver = { path = "../../my-module-repo/firmware" } # This could also be a local path
```

4. Add the workspace dependency under `# Drivers` in [`firmware/xpanse/Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/xpanse/Cargo.toml):

```toml
my-driver = { workspace = true }
```

5. Add a match arm for your driver in [`load_driver.rs`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/xpanse/src/load_driver.rs), following the existing drivers:

```rust
Some(id) if id == my_driver::MyDriver::ID => {
    match my_driver::MyDriver::create(bank, slot, registry, bus).await {
        Ok(()) => defmt::info!("My driver initialized in {:?}", slot),
        Err(error) => {
            defmt::error!("My driver init failed in {:?}: {:?}", slot, error)
        }
    }
}
```

Rust crate names use underscores in code, so a crate named `my-driver` in `Cargo.toml` is imported as `my_driver`.

6. Build the firmware by running `cargo build` from the `firmware` folder.
7. Fix any compilation errors, then publish your driver crate on [crates.io](https://crates.io).
8. Replace the local path dependency in `firmware/Cargo.toml` with the version published on crates.io:

```toml
my-driver = "0.1.0"
```

9. Run `cargo build` again to make sure the firmware builds with the published crate.
10. Make a PR to the hackxpansion repo with your driver dependency and `load_driver.rs` match arm.
11. When your module and console arrive, test the driver on the real hardware and fix any bugs.
12. Publish a new version of your driver on crates.io if fixes are needed.
13. Make another PR to the hackxpansion repo with the bumped driver version.
