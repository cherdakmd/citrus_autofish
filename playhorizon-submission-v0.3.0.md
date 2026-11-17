# PlayHorizon submission draft — Citrus AutoFish 0.3.0

Prepared for manual submission at https://playhorizon.dev/mods/submit. The form itself has not been submitted.

## Suggested listing fields

- **Name:** Citrus AutoFish
- **Version:** 0.3.0 (experimental prerelease)
- **Mod ID:** `citrus_autofish`
- **Game:** Minecraft: Java Edition 26.3
- **Platform / loader:** Horizon native mod (not Fabric, Forge, or Bedrock)
- **Horizon compatibility:** revision `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0`; native hooks API 415; mixins revision `0d6b103c70633d8fd9443d8d0acd3aadaacedb06`
- **Category suggestion:** Utility / Farming (choose the closest category available in the submission form)
- **Source:** https://github.com/cherdakmd/citrus_autofish
- **Release page:** https://github.com/cherdakmd/citrus_autofish/releases/tag/v0.3.0
- **Download:** [citrus_autofish-0.3.0.hnmod](https://github.com/cherdakmd/citrus_autofish/blob/93f8268/release-assets/citrus_autofish-0.3.0.hnmod) (use GitHub's **Download raw file** action; native source mods use `.hnmod`)
- **SHA-256:** `7762bb4c393841f033221fef2a38dde97f64e4a4e27cd743c2fef303347b9c50`

## Short description

Experimental Horizon native mod for Minecraft Java that automates fishing and helps protect fishing rods with configurable hotbar switching and safety settings.

## Full description

Citrus AutoFish automates fishing in Minecraft Java Edition using Horizon. Toggle it with `/autofish` or `.autofish`, and open its settings with `/autofish menu` or Right Shift. Configure automatic rod switching, a durability threshold, safe return to the previous rod, the recast delay, and whether AutoFish stops when no safe spare rod is available. Settings are also available from Horizon's native Mods page and persist between launches.

**Experimental:** the direct `UseItem` packet path uses a sequence counter separate from Horizon's internal counter and may conflict with acknowledgements or other item interactions. Rod switching depends on inventory synchronization and server behavior. Build with Horizon Launcher and test in a disposable single-player or test world first. Do not use where automation is prohibited.

## Submission checklist

- For the native-mod submission type, upload `release-assets/citrus_autofish-0.3.0.hnmod` from this workspace; do not submit the combined `.hmod` unless the form explicitly asks for that package kind.
- Keep the experimental status and packet-sequence warning visible in the listing.
- The bundle contains native Rust source and manifests; it is not a precompiled game binary. The user must allow native code and build it with Horizon Launcher.
- Confirm the listing form's category, upload rules, and supported-version fields before submitting. This draft was prepared without accessing or submitting the PlayHorizon form.
