## Citrus AutoFish v0.1.2 — experimental source prerelease

This is a diagnostic fix for the Horizon Launcher 0.2.69 source revision mismatch reported while building v0.1.1.

### Changes
- Pin Horizon to the installed revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` instead of `95c79d7`. The latter source tree lacks `crates/horizon-mods/Cargo.toml`, which Launcher 0.2.69's staged workspace requires.
- Adapt the Rust hooks to the older 0.2.69 trait API (`Host` and `tick`).
- Replace unavailable `click_mapping` with a direct `ServerboundPlayPacket::UseItem` request, using an experimental independent sequence counter.

### Important limitations
- **The Rust crate has not been compiled in this environment, and this release has not been tested in the launcher or in-game. Do not treat it as confirmed working.**
- Direct packets are not equivalent to clicks in Horizon's vanilla input queue. The mod cannot read/update the client's internal action sequence through this API; simultaneous item/block interactions may conflict with sequence acknowledgements. Test only in a disposable world and disable the mod if anything looks wrong.
- AutoFish toggles with `/autofish`, but this legacy API path does not provide local chat confirmation.
- The server-side fishing hook may not be available on remote vanilla servers.

### Build command (Windows PowerShell 5.1)
Run this from the extracted source root:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

The release contains source only; Launcher compiles the native mod locally. If the build fails, preserve the full log rather than assuming it is still downloading.