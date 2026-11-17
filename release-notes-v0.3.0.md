# Citrus AutoFish v0.3.0 (experimental)

- Added `.autofish` as a plain chat alias, alongside `/autofish`; typing the exact dot alias is routed locally and not sent as public chat.
- Added command modes: `on`, `off`, `toggle`, `menu`, `settings` and `help`.
- Added a configurable recast delay (2, 4, 6 or 8 ticks; default 4).
- Added a safety option that disables AutoFish when the active rod is in the danger range and no safe spare rod is available (enabled by default).
- Expanded the in-game and standard Mods-page settings.
- Compatible with Horizon revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, hooks API 415, and Horizon Mixins revision `0d6b103c70633d8fd9443d8d0acd3aadaacedb06`.

## Install in Horizon Launcher 0.2.69

Use the native source bundle [`citrus_autofish-0.3.0.hnmod`](https://github.com/cherdakmd/citrus_autofish/blob/93f8268/release-assets/citrus_autofish-0.3.0.hnmod), not the combined `.hmod`, in `horizon-native-mods`. The `.hmod` produced for the requested registry format is not recognized by this Launcher version in that directory and can produce `horizon-native.json: unsupported package path`. Enable Citrus AutoFish and click **Build**; native code runs with your account permissions.

## Experimental status

The direct `UseItem` path uses a sequence counter separate from Horizon's internal counter and may conflict with acknowledgements or other item interactions. Rod switching depends on inventory synchronization and server behavior. The archives and manifest identities were checked, but Rust compilation and v0.3.0 in-game behavior have not been independently verified. Build and test in a disposable single-player world first; do not use where automation is prohibited.
