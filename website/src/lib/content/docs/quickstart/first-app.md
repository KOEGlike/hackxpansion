# Writing your first app

This guide will teach you how to create a simple app that uses some buttons.

## Experience needed

To understand this guide, you should have lite programming experience and have read/completed the [First Driver](./first-driver) guide. If you don't understand something google is your best friend. If you have any further questions just ask in `#hackxpansion`

## Download software

- **Slint Extension** This adds syntax highlight and a live preview to the UI framework.

- All other software needed for [First Driver](./first-driver)

## Setup your app repo

Generally you should create a new repo for your apps, because they generally should be driver independent (for example your app that uses an A button shouldn't care about which button module u plugged in as long as it has an A button).

Once you created your repo on GitHub and cloned it to your local machine, you should run `cargo init --lib .` in your repo directory. This is the same as `cargo new --lib` from the driver guide, but it doesn't create a new dir, instead it uses the one that you are currently in.

## Include dependencies

You have to add `xpanse-api` and `slint` as dependency, simply run `cargo add xpanse-api slint` in your firmware directory. You also need a build time dependency for slint, run `cargo add slint-build --build`. These add all the stuff that is needed for creating a basic app to the project. These commands added a new entries in `Cargo.toml`.

If your app uses drivers that add non standard resources to the registry, you also have to add those crates as a dependency who expose these types.

## UI

For the UI we will use Slint.

For more detailed docs on Slint, check out the [official Slint docs](https://docs.slint.dev/latest/docs/slint/).

### What is Slint?

Your app has two options to put stuff on the screen:

- The first is direct video, where your app directly writes the colors of pixels to the screen, with no middle man. This is very useful for emulators, or if you want to write your own render. But this option is very cumbersome to use for simple UI apps.
- The secund option is [Slint](https://slint.dev/), a very powerful UI framework. Slint is very similar to JS UI frameworks, like React or Svelte: you can define components, style elements very easily, built in reactivity.

One of the very nifty things about Slint is that it has a live preview, so you can see how your app will look, even before you get your console. In the embedded word this is not a given, so this is a luxury feature. I'll cover later on how to use this.

### Creating a `.slint` file

Slint uses `.slint` files to define UI. So, create a dir called `ui` in the root of your repo, and inside that a file called `ui.slint` or smth similar.

### Setting up build script

But we have to tell the rust compiler to use these `.slint` files, so we gotta add a build script.

Simply create a `build.rs` file in the root of your repo, and paste this into it:

```rust
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/ui.slint", config)
        .expect("failed to compile button logger UI");

    println!("cargo:rerun-if-changed=ui/button_logger.slint");
}
```

If you didn't use `ui.slint` for your slint file, simple replace `ui.slint` with the name you choose.

Currently this is how your repo should look like:

```
my-app-repo/
├─ Cargo.toml
├─ build.rs
├─ src/
│  ├─ lib.rs
├─ ui/
│  ├─ ui.slint
```

### Creating basic UI

In this section I will explain a basic `.slint` file that displays how many times a button has been pressed.

```slint
// For your app to use your UI, it has to export a component that inherits Window
export component ButtonLoggerUI inherits Window {
    // These are the dimension of the physical screen
    width: 320px;
    height: 240px;

    // This is a reactive proprety that we will set from our app's logic code
    in-out property <int> count: 0;

    // White BG
    Rectangle {
        width: parent.width;
        height: parent.height;
        background: #ffffff;
    }

    // Just like flex in CSS
    VerticalLayout {
        width: parent.width;
        height: parent.height;
        alignment: center;

        Text {
            // Displaying the reactive count proprety
            text: "Clicks: " + root.count;
            color: #008000;
            horizontal-alignment: center;
        }

        // Each app should have a way to exit
        Text {
            text: "Hold A to exit";
            color: #000000;
            horizontal-alignment: center;
        }
    }
}
```

### Using live preview

_This feature is only available in VSC_

Open your `.slint` file and click `Show Preview` above your exported component. And voila:

![](https://cdn.hackclub.com/019feae4-1809-7ca9-a21d-6cd654abb8dc/image.png)

## Business Logic

Now that you are done with the UI, it's time to actually use it in a app.

This is how a basic `lib.rs` of an app should look like, I added comments to explain it:

```rust
#![no_std]

// We use Box and other types that require the heap, so we have to include this
extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use xpanse_api::{
    app::App,
    interfaces::buttons::{A, Button},
    reexports::{
        defmt,
        embassy_time::Timer,
        slint::{self, ComponentHandle},
    },
    registry::{Registry, ResourceLease},
};

// This will include all the components that we define in `.slint` files
slint::include_modules!();

// Every app is a struct, that has all the resources that it needs inside it
pub struct ButtonLoggerApp {
    button: ResourceLease<Box<dyn Button<A>>>,
}

// Every app needs to impl the App trait
impl App for ButtonLoggerApp {
    // This will be the display name in the app list
    const NAME: &'static str = "Button Logger";

    // This function checks if the registry has all
    // the resources that this app *needs* to run. Like controls/buttons
    //
    // And if it can run, it will be added to the app list
    fn can_run(registry: &Registry) -> bool {
        // We check if the registry has an A button
        registry.has::<Box<dyn Button<A>>>()
    }

    // This function takes all the resources that the app needs,
    // and creates a new app instance if everything is successful
    //
    // This function can try to take resources that the app
    // doesn't *need* to run. Like a game can run without sound,
    // but not without buttons
    fn new(registry: &mut Registry) -> Option<Self> {
        // We try to take an A button
        let button = registry.take_resource::<Box<dyn Button<A>>>()?;
        Some(Self { button })
    }

    // This is where the actual app runs
    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        // We need a Box cuz of rust async trait stuff
        Box::pin(async move {
            // We try to create an instance of our exported component
            let ui = match ButtonLoggerUI::new() {
                Ok(ui) => ui,
                Err(_) => {
                    defmt::error!("ButtonLoggerApp: failed to create UI");
                    return;
                }
            };

            // We try to show this component on the screen
            if ui.show().is_err() {
                defmt::error!("ButtonLoggerApp: failed to show UI");
                return;
            }

            let mut count = 0u32;
            loop {
                // We wait until the button is pressed
                self.button.resource_mut().wait_for_pressed().await;

                // We count for how many ms was the button held
                let mut held_ms = 0;
                while self.button.resource().is_pressed() && held_ms < 1_000 {
                    Timer::after_millis(20).await;
                    held_ms += 20;
                }

                // If the button was held for more than 1000 ms, we exit from the app
                if held_ms >= 1_000 {
                    while self.button.resource().is_pressed() {
                        Timer::after_millis(20).await;
                    }
                    break;
                }

                // If the press was shorter than 1000ms, we add 1 to `count`
                count += 1;
                // Slint automatically generates `set_` functions for reactive variables
                ui.set_count(count as i32);
                // Log the count to the console
                defmt::info!("button A pressed (count: {})", count);
            }

            // Hide the ui when the app exits
            if ui.hide().is_err() {
                defmt::error!("ButtonLoggerApp: failed to hide UI");
            }
        })
    }

    // Since we took a button from the registry
    // we should also give it back
    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.button);
    }
}
```

## Adding your app to the firmware

If you followed the driver guide you should already have a fork and local clone of the main hackxpansion repo. Adding an app is almost the exact process as adding a driver. Here are the steps:

1. Fork and clone the [hackxpansion repo](https://github.com/KOEGlike/hackxpansion) if you haven't already.
2. Go into the `firmware` folder.
3. Add your app crate as a local workspace dependency under `# Apps` in [`firmware/Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/Cargo.toml). The path should point to your local app repo while you are testing it:

```toml
my-app = { path = "../../my-app" } # This also could be an absolute path
```

4. Add the workspace dependency under `# Apps` in [`firmware/xpanse/Cargo.toml`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/xpanse/Cargo.toml):

```toml
my-app = { workspace = true }
```

5. Add your app to `APP_CATALOG` in [`app_loader.rs`](https://github.com/KOEGlike/hackxpansion/blob/main/firmware/xpanse/src/app_loader.rs), following the existing apps:

```rust
AppDescriptor {
    name: my_app::MyApp::NAME,
    can_run: my_app::MyApp::can_run,
    run: run_app_impl::<my_app::MyApp>,
},
```

Rust crate names use underscores in code, so a crate named `my-app` in `Cargo.toml` is imported as `my_app`.

6. Build the firmware by running `cargo build` from the `firmware` folder.
7. Fix any compilation errors, then publish your app crate on [crates.io](https://crates.io).
8. Replace the local path dependency in `firmware/Cargo.toml` with the version published on crates.io:

```toml
my-app = "0.1.0"
```

9. Run `cargo build` again to make sure the firmware builds with the published crate.
10. Make a PR to the hackxpansion repo with your app dependency and `APP_CATALOG` entry.
11. When your modules and console arrive, test the app on the real hardware and fix any bugs.
12. Publish a new version of your app on crates.io if fixes are needed.
13. Make another PR to the hackxpansion repo with the bumped app version.
