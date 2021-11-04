# Preview

![Preview](https://github.com/4JX/L5P-Keyboard-RGB/blob/dev/Preview.png)

# Index

- [Download](#download)
- [Available effects](#available-effects)
- [Compatibility](#compatibility)
- [Building from source](#building-from-source)
  - [Using `cargo-make`](#using-cargo-make)
  - [Building manually](#building-manually)

# Download

**⚠️ Use at your own risk, the developer is not responsible for any damages that may arise as a result of using this program.**

Builds will be periodically uploaded to the [releases tab](https://github.com/4JX/L5P-Keyboard-RGB/releases).

You may also download precompiled versions from [here](https://github.com/4JX/L5P-Keyboard-RGB/actions/workflows/release-rust.yml) (requires github account) by clicking the latest entry with a ✅ and going under the "artifacts" section.

# Available effects

**All stock effects:** Static, Breath, Smooth, LeftWave, RightWave.

**Custon effects:**

- **Lightning:** Adds a little _spark_.
- **AmbientLight:** Reacts to content on your screen.
- **Smooth(Left/Right)Wave:** An implementation of the classic wave effect.
- **(Left/Right)Swipe:** Transitions the selected colors from side to side, useful for custom waves.

# Compatibility

This program has been tested to work on the 4 zone keyboard of the **2021** Legion 5 and Legion 5 Pro models on both Windows and Linux.

### "How about X model"

- **Legion 7(i):** Won't work, the backlight on these is per-key and uses a different way of communicating.
- **Any variant with a white backlight:** Havent figured out how to talk to this one yet, but given the limited number of states (off, low, high) there's not many effects I'd be able to add anyways.
- **2020 Models:** These are not currently supported by the program, but support should be easy enough to add.

# Building from source

## Prequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/downloads)
- On Linux, you'll need additional dependencies:

**Ubuntu**

```sh
$ sudo apt-get update && sudo apt-get install -y libpango1.0-dev libx11-dev libxext-dev libxft-dev libxinerama-dev libxcursor-dev libxrender-dev libxfixes-dev libudev-dev nasm libxcb-randr0-dev libusb-1.0-0-dev libdbus-1-dev
```

**EndeavourOS (Arch Linux)**

```sh
$ sudo pacman -S nasm cmake
```

## Using `cargo-make`

Works on both Windows and Linux.

- Install `cargo-make`

```
$ cargo install cargo-make
```

- Clone the repository

```sh
$ git clone https://github.com/4JX/L5P-Keyboard-RGB.git
```

- Build the project

```sh
$ cd L5P-Keyboard-RGB/
$ cargo make build
# Or
$ cargo make build-release
```

## Building manually

- Download and bootstrap [VCPKG](https://github.com/Microsoft/vcpkg#getting-started)
  - You'll need to set an enviorement variable called `VCPKG_INSTALLATION_ROOT` pointing to the directory where you downloaded and bootstrapped VCPKG.

### Windows

- Download the necessary dependencies

```cmd
> vcpkg update && vcpkg install libvpx:x64-windows-static libyuv:x64-windows-static
```

- Clone the repository

```cmd
> git clone https://github.com/4JX/L5P-Keyboard-RGB.git
```

- Build the project

```cmd
> cd L5P-Keyboard-RGB/
> cargo build --release
```

### Linux

- Download the necessary dependencies

```sh
$ vcpkg update && vcpkg install libvpx libyuv
```

- Clone the repository

```sh
$ git clone https://github.com/4JX/L5P-Keyboard-RGB.git
```

- Build the project

```sh
$ cd L5P-Keyboard-RGB/
$ cargo build --release
```

# Crashes, freezes, etc

I cannot guarantee this solution will work for anyone but myself. That being said feel free to open an issue if you encounter any of these problems on the [issues tab](https://github.com/4JX/L5P-Keyboard-RGB/issues).

---

Thanks to legendk95#0574 (272711294338072577) over at discord for initially reverse engineering the way to talk to the keyboard.
