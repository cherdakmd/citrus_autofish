## Citrus AutoFish v0.1.4 — experimental source prerelease

Supersedes v0.1.3, which still asked offline Cargo to resolve `horizon-protocol` from crates.io.

### Changes
- Keep the Horizon pin at `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, matching Launcher 0.2.69 and containing `crates/horizon-mods` in the staged source.
- Point `horizon-protocol` directly at `../../crates/horizon-protocol`. The native-build stage places this mod at `native/stage.new/native-api/citrus_autofish`, so this path resolves to the crate inside the staged Horizon source and needs no registry or extra Git fetch.
- Keep the experimental direct `UseItem` packet workaround. The 0.2.69 API does not provide the vanilla `click_mapping` helper.

### Important limitations
- **Not compiled in this environment; not yet tested in the launcher or in-game. Do not assume it builds or works until verified.**
- Direct `UseItem` packets are not identical to vanilla input. The independent sequence counter is not synchronized with the client's own actions; overlapping item/block interactions may conflict with sequence acknowledgements. Test only in a disposable world and disable the mod if anything looks wrong.
- `/autofish` toggles silently; this legacy API path does not provide local chat confirmation.
- AutoFish relies on the server-side fishing hook and may not work on remote vanilla servers.

### Build command (Windows PowerShell 5.1)
Run from the extracted source root:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

The release is source-only; Launcher compiles locally. If the build fails, preserve the full output and `C:\Users\ADMIN\AppData\Roaming\horizon-launcher\native\build.log`.