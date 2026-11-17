## Citrus AutoFish v0.1.3 — experimental source prerelease

Supersedes v0.1.2, whose launcher build stopped in Cargo offline mode while trying to fetch `horizon-protocol` as a second Git dependency.

### Changes
- Keep Horizon pinned to `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, matching the revision Launcher 0.2.69 first checks and containing the staged workspace's `crates/horizon-mods` crate.
- Declare `horizon-protocol` as version `0.1.0`, matching Horizon's Cargo.lock and the native-mod template's supported dependency form, instead of requesting a separate checkout of the Horizon Git repository.
- Keep the experimental `UseItem` packet approach needed because this revision has no supported vanilla `click_mapping` host method.

### Important limitations
- **Not compiled in this environment; not yet tested in the launcher or in-game. Do not assume this release builds or works until verified.**
- Direct `UseItem` packets are not identical to vanilla input. The independent sequence counter is not synchronized with the client's own actions; overlapping item/block interactions may conflict with sequence acknowledgements. Test only in a disposable world and disable the mod if anything looks wrong.
- `/autofish` toggles silently; the legacy API path does not provide local chat confirmation.
- AutoFish relies on the server-side fishing hook and may not work on remote vanilla servers.

### Build command (Windows PowerShell 5.1)
Run from the extracted source root:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

The release is source-only; the launcher compiles locally. If the build fails, preserve the complete output and the resulting `native\build.log`.