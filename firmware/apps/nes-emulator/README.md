# NES Emulator

This app uses [RuNES](https://github.com/Determinant/runes) as its no-std NES
emulation core. No game is included in the firmware repository.

Set `NES_ROM` to an iNES-format ROM while building the firmware:

```sh
NES_ROM=/path/to/game.nes cargo build --release --target thumbv8m.main-none-eabihf -p xpanse
```

Supported mapper IDs are 0 (NROM), 1 (MMC1), 2 (UxROM), and 4 (MMC3), matching
RuNES. The app appears in the picker only when a ROM was embedded and eight
logical controls (`D-pad`, `A`, `B`, `X`, and `Y`) and the platform framebuffer
resource are available. `X` maps to Select, `Y` maps to Start, and holding
Up+Down exits to the app picker.
