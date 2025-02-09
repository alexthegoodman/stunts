# Stunts - Professional Video Editor

Requires these repos aside this one:

- `common-floem`
- `stunts-engine`
- `common-motion-2d-reg` (attention branch)

Stunts intends to make motion graphics and film much more efficient.

- `cargo run --release`

## Release

Remember to increment version number in wix/main.wxs

- `cargo build --release --features production`
- `wix extension add -g WixToolset.UI.wixext` (once)
- `wix build wix\main.wxs -ext WixToolset.UI.wixext -o stunts-installer-v1-0-0.msi`
