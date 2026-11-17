## Citrus AutoFish v0.1.5 — experimental source prerelease (superseded)

**Superseded by v0.1.6.** The user's Launcher 0.2.69 build reached Rust compilation and reported two trait-API mismatches: `tick` needs a `TickEvent`, and `CommandEvent` has no `ran` field. Use v0.1.6 instead.

### Changes
- Keep the Horizon pin at `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`, matching Launcher 0.2.69.
- Remove the direct `horizon-protocol` dependency entirely.
- Construct the pinned revision's `UseItem` packet using the public `ServerboundPlayPacket::decode` API, with serverbound packet id 67 and its vanilla wire payload. This keeps the exact Horizon packet type without a forbidden path or registry dependency.

### Important limitations
- **Not compiled in this environment; not yet tested in the launcher or in-game. Do not assume it builds or works until verified.**
- The packet id is specific to Horizon revision `9f42064`. Do not change the source pin independently.
- Direct `UseItem` packets are not identical to vanilla input. The mod's independent sequence counter is not synchronized with the client's own actions; overlapping item/block interactions may conflict with sequence acknowledgements. Test only in a disposable world and disable the mod if anything looks wrong.
- `/autofish` toggles silently; the legacy API path does not provide local chat confirmation.
- AutoFish relies on the server-side fishing hook and may not work on remote vanilla servers.

### Build command (Windows PowerShell 5.1)
Run from the extracted source root:

```powershell
& "C:\Users\ADMIN\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe" native-build .\mixins\citrus_autofish --trust-native-code
```

The release is source-only; Launcher compiles locally. If it fails, preserve the complete output and `C:\Users\ADMIN\AppData\Roaming\horizon-launcher\native\build.log`.