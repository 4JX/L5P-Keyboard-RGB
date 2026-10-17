# legion-input

An observe-only keyboard event source for Linux and Windows.

- normalized physical key identities;
- native event metadata (evdev code, or Windows scan and virtual-key codes);
- key-down, key-up, and device-reset events;
- shared held-key state and event subscriptions;
- one event-driven platform worker, shared by every cloned input handle;
- deterministic shutdown when the final input handle is dropped;
- Linux evdev discovery, removal, and hotplug recovery;
- a Windows low-level keyboard hook using physical scan codes.

## Linux permissions

The process must be allowed to read keyboard devices under `/dev/input`.
On most distributions this means adding the user to the `input` group and
starting a new login session. A udev rule granting equivalent access can be
used instead on systems where that is preferred or where the group does not
exist.

Access to the `input` group generally allows reading input from every attached
device, not only the keyboard used by this application.

The overall backend shape and portions of the key mapping were informed by
[keytap 0.4.0](https://github.com/jamiepine/keytap).

Heavily assisted by GPT-5.6 Sol, given it's outside my normal knowledge.
