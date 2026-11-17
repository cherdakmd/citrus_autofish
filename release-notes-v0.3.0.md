# Citrus AutoFish v0.3.0 (experimental)

- Added `.autofish` as a plain chat alias, alongside `/autofish`; typing the exact dot alias is routed locally and not sent as public chat.
- Added command modes: `on`, `off`, `toggle`, `menu`, `settings` and `help`.
- Added a configurable recast delay (2, 4, 6 or 8 ticks; default 4).
- Added a safety option that disables AutoFish when the active rod is in the danger range and no safe spare rod is available (enabled by default).
- Expanded the in-game and standard Mods-page settings.
- Compatible with Horizon revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, hooks API 415, and Horizon Mixins revision `0d6b103c70633d8fd9443d8d0acd3aadaacedb06`.

## HMOD package

`citrus_autofish-0.3.0.hmod` is a Horizon native-mod package: it contains the native bundle plus a matching `horizon-mod.toml` manifest, as required for the combined `.hmod` format. Download it from [`release-assets/citrus_autofish-0.3.0.hmod`](https://github.com/cherdakmd/citrus_autofish/blob/855c61e/release-assets/citrus_autofish-0.3.0.hmod) using GitHub's **Download raw file** action. Install into the instance's `horizon-native-mods` folder, enable it, and build with Horizon Launcher.

SHA-256: `00e90ab5f2601f8e00e0980e195ea605c814150ce0713026c9b8b80e3829333b`

## Experimental status

The direct `UseItem` path uses a sequence counter separate from Horizon's internal counter and may conflict with acknowledgements or other item interactions. Rod switching depends on inventory synchronization and server behavior. The `.hmod` archive and its manifest/layout were checked, but Rust compilation and v0.3.0 in-game behavior have not been independently verified. Build and test in a disposable single-player world first; do not use where automation is prohibited.
