# Citrus AutoFish v0.3.0

- Added `.autofish` as a plain chat alias, alongside `/autofish`; typing the exact dot alias is routed locally and not sent as public chat.
- Added explicit command modes: `on`, `off`, `toggle`, `menu`, `settings` and `help` for either alias.
- Added a configurable recast delay (2, 4, 6 or 8 ticks; default 4).
- Added a safety option that disables AutoFish when the active rod is in the danger range and no safe spare rod is available (enabled by default).
- Expanded the in-game and standard Mods-page settings with the new controls.
- Remains compatible with Horizon revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, hooks API 415.

The direct `UseItem` path is still experimental. Compile this source bundle in Horizon Launcher and verify on a test world before using it on a server.
