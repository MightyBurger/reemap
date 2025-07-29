<div align="center">

# Reemap

Lightweight input remaps

![reemap screenshot](resource/example.png)

**NOTE: Reemap is currently in development.** It still has important issues that must be resolved before
a release. See [Development Status](#development-status). ️⚠️

</div>


Reemap is an input remapping tool for Windows with a focus on video games. It runs in the background
and remaps keyboard and mouse inputs.

Reemap was originally developed for [Ori] speedrunners who use input remaps.

Reemap is named after Reem. In [Ori and the Blind Forest][Ori], Reem is the spirit whose ancestral
tree grants Bash.

This repository contains the source code for Reemap.

### Escape Hatch

If you get stuck, **enable Scroll Lock** to temporarily disable remaps. ⚠️

## Features

- **Profiles** - remap inputs differently for each program
- **Layers** - conditionally override inputs in a profile
- **Keyboard and mouse** - freely map keyboard inputs to mouse inputs, and vice versa
- **Multi maps** - map one input to multiple outputs
- **Export and import profiles** - share or back up profiles
- **Confine cursor to window** - prevent the mouse from leaving the window (useful for fullscreen
games on multi-monitor setups)

The input remaps are implemented thoughtfully and from the perspective of using Reemap for video games.
For example, the software will not cause keys to get stuck as you switch layers. This is an issue
with some existing remap software, including X-Mouse.

Reemap does not support macros or any form of timed inputs.

## Installation

Reemap is still in development. If you want to try it out now, you can build from source; see
[Compilation](#compilation). The completed versions will be available on a Github Releases page.

## Compilation

To compile Reemap, you will need:
- [Rust]
- [Windows SDK]

Clone the repository. Then, in the repository root, run `cargo build --release`.

## Development Status

This section will be removed on the first release of Reemap.

- [x] allow exporting, importing profiles
- [x] allow reordering profiles, layers, remap outputs
- [x] check process name or window name
- [x] enumerate all open windows
- [x] app icon
- [x] add background to UI, touch up UI
- [x] search/filter buttons
- [x] allow copying profiles, layers
- [x] clip mouse
- [ ] only re-render GUI when necessary (note; currently fine when minimized to tray)
- [ ] write tutorial
- [ ] better support different keyboard layouts - possibly switch to scancodes instead of virtual
key codes?
- [ ] some UI code is repetative; other UI code is in weird spots; just do a clean-up pass
- [ ] could possibly do with an abstraction for hooks to clean up the hook thread's code; it's a
little ugly right now
- [ ] API for variants
- [ ] create a distributable installer (signing certificate?)

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

[Ori]: https://www.orithegame.com/
[Rust]: https://www.rust-lang.org/
[Windows SDK]: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/
