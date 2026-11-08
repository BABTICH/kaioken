# Release Secrets Configuration

To enable automated code signing and notarization in the GitHub Actions release pipeline, the following secrets must be configured in the repository settings (`Settings > Secrets and variables > Actions`).

## macOS (Apple Developer ID)

1. **APPLE_ID**: The email address associated with your Apple Developer account.
2. **APPLE_APP_SPECIFIC_PASSWORD**: An app-specific password generated at appleid.apple.com (required for notarization).
3. **APPLE_TEAM_ID**: Your 10-character Apple Developer Team ID.
4. **APPLE_CERTIFICATE**: The base64 encoded string of your `.p12` Developer ID certificate.
5. **APPLE_CERTIFICATE_PASSWORD**: The password used to encrypt the `.p12` file.

## Windows (Authenticode)

1. **WIN_CSC_LINK**: The base64 encoded string of your Windows `.pfx` code signing certificate.
2. **WIN_CSC_KEY_PASSWORD**: The password for the `.pfx` certificate.

## GitHub

- **GH_TOKEN**: Automatically provided by GitHub Actions (`secrets.GITHUB_TOKEN`). This allows `electron-builder` to create the GitHub Release and upload the artifacts and `latest.yml` files.

---

## Triggering a Release

To trigger the automated build, sign, and publish pipeline:

```bash
npm version patch  # or minor / major
git push --follow-tags
```

The pipeline will:
1. Build the app for Windows, macOS, and Linux.
2. Sign the binaries using the provided certificates.
3. Notarize the macOS build with Apple.
4. Create a new GitHub Release.
5. Upload the installers (`.exe`, `.dmg`, `.AppImage`, `.deb`).
6. Upload the auto-updater feeds (`latest.yml`, `latest-mac.yml`, `latest-linux.yml`).
