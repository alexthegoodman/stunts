# Stunts - Professional Video Editor

Stunts intends to make motion graphics and film much more efficient.

- `cargo run`

## Release

Remember to increment version number in wix/main.wxs

- `wix extension add -g WixToolset.UI.wixext`
- `wix build wix\main.wxs -ext WixToolset.UI.wixext -o stunts-installer.msi`
