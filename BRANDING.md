# Co-branding Luna Telepresence

The product identity has one source file: `frontend/brand.config.json`.

1. Change the product name, identifier, data-directory names, links, and updater settings.
2. From `frontend`, run `pnpm brand:apply`.
3. Replace the image files in `frontend/public` and `frontend/src-tauri/icons` when a branded icon set is available.
4. Build and sign the application for the target operating system.

`modelLibraryDirectoryName` is intentionally separate from the Tauri application
identifier. Keep it stable across compatible branded builds when they should share
downloaded transcription and summary models. On Windows, Luna stores the shared
library under `%LOCALAPPDATA%\Luna Telepresence Model Library\models`.

Updates are disabled by default so a co-branded build can never install packages signed or published by the upstream Meetily project. To enable updates, provide a release endpoint and the matching Tauri public key in the brand file.

The upstream MIT copyright and license remain in `LICENSE.md`, as required by the license.
