# Legion Keyboard RGB Control Program

![Preview](https://github.com/4JX/L5P-Keyboard-RGB/blob/dev/Preview.png)

A fun little experiment. Probably contains bugs.

**⚠️ Use at your own risk, the developer is not responsible for any damages that may arise as a result of using this program.**

## Download

Builds will be periodically uploaded to the [releases tab](https://github.com/4JX/L5P-Keyboard-RGB/releases).

You may also download precompiled versions from [here](https://github.com/4JX/L5P-Keyboard-RGB/actions/workflows/release-rust.yml) (**requires github account**) by clicking the latest entry with a ✅ and going under the "artifacts" section.

## Available effects

**All stock effects:** Static, Breath, Smooth, LeftWave, RightWave.

- **Lightning:** Adds a little _spark_.
- **AmbientLight:** Reacts to content on your screen.
- **Smooth(Left/Right)Wave:** An implementation of the classic wave effect.
- **(Left/Right)Swipe:** Transitions the selected colors from side to side, useful for custom waves.

## Compatibility

This program has been tested to work on the 4 zone keyboard of the **2021** Legion 5 and Legion 5 Pro models on both Windows and Linux.

### "How about X model"

- **Legion 7(i):** Won't work, the backlight on these is per-key and uses a different way of communicating.
- **Any variant with a white backlight:** Havent figured out how to talk to this one yet, but given the limited number of states (off, low, high) there's not many effects I'd be able to add anyways.

## Crashes, freezes, errrors, etc

I cannot guarantee this solution will work for anyone but myself. That being said feel free to open an issue if you encounter any of these problems on the [issues tab](https://github.com/4JX/L5P-Keyboard-RGB/issues).

---

Thanks to legendk95#0574 (272711294338072577) over at discord for initially reverse engineering the way to talk to the keyboard.
