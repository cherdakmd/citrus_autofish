## Citrus AutoFish v0.1.6 — experimental source prerelease

Fixes the two Rust API errors reported while compiling v0.1.5 with Horizon Launcher 0.2.69:
- `Hooks::tick` now accepts Horizon's required `TickEvent` argument.
- `CommandEvent` exposes only the command text; returning `Flow::Cancel` marks the command as handled, so the nonexistent `event.ran` write was removed.

The prior dependency workaround remains: `UseItem` is constructed through the pinned `ServerboundPlayPacket::decode` API, without a forbidden direct path/registry dependency on `horizon-protocol`.

### Compatibility and limitations
- Horizon revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`; Launcher 0.2.69.
- **Native-build was confirmed by the user** on Windows PowerShell 5.1 / Horizon Launcher 0.2.69: `native-9f42064-5ca42cdb9fae` built and was selected for the instance in 2 min 27 s (Performance). The mod has not yet been tested in-game; functionality remains unconfirmed.
- Direct `UseItem` packets are not identical to vanilla input. The mod's independent sequence counter is not synchronized with the client's own actions; overlapping item/block interactions may conflict with sequence acknowledgements. Test only in a disposable world and disable the mod if anything looks wrong.
- `/autofish` toggles silently. The server-side fishing hook may not be available on remote vanilla servers.

### Build command (Windows PowerShell 5.1)
Run from the extracted source root:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

Preserve the full output and `C:\Users\ADMIN\AppData\Roaming\horizon-launcher\native\build.log` if compilation fails.