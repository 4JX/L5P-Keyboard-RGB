# L5P Keyboard RGB Control Program

A fun little experiment. Probably contains bugs.

**⚠️ Use at your own risk, the developer is not responsible for any damages that may arise as a result of using this program.**

## Compatibility

This program has been tested to work on the 4 zone keyboard of the Legion 5 Pro 2021. It [_should_](https://www.reddit.com/r/LenovoLegion/comments/panu9f/progress_update_on_effects_d/haf346a?utm_source=share&utm_medium=web2x&context=3) alledgedly work on the Legion 5 Gen 6 2021 too, though I'd need to test it with an user who has one.

### "How about X model"

- **Legion 7(i):** Won't work, the backlight on these is per-key and uses a different way of communicating.
- **Any variant with a white backlight:** Havent figured out how to talk to this one yet, but given the limited number of states (off, low, high) there's not many effects I'd be able to add anyways.

## Download

You may download precompiled versions from [here](https://github.com/4JX/L5P-Keyboard-RGB/actions/workflows/release-rust.yml) by clicking the latest entry with a ✅ and going under the "artifacts" section.

## Available effects

**All stock effects:** Static, Breath, Smooth, LeftWave, RightWave.

- **(Left/Right)Pulse:** A light travels from one side to the other of the keyboard.
- **Lightning:** Adds a little _spark_.
- **AmbientLight:** Reacts to content on your screen.
- **Smooth(Left/Right)Wave:** An implementation of the classic wave effect.
- **(Left/Right)Swipe:** Transitions the selecter colors from side to side, useful for custom waves.

## Crashes, freezes, errrors, etc

I cannot guarantee this solution will work for anyone but myself. That being said feel free to open an issue if you encounter any of these problems on the [issues tab](https://github.com/4JX/L5P-Keyboard-RGB/issues).

---

Thanks to legendk95#0574 (272711294338072577) over at discord for initially reverse engineering the way to talk to the keyboard.
