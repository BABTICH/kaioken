# Kaioken Desktop Studio — Release Runbook

This runbook is the official release procedure for Kaioken Desktop Studio.

A release must not ship if any `GATE` item in the regression matrix fails.

---

## 1. Release Gates

Before packaging:

```bash
npm run typecheck
npm run build
npm run test:e2e
```

Inside the app:

1. Open Settings.
2. Run **Release Certification**.
3. Required result:
   - `pass`, or
   - `warn` only with documented justification.
4. `fail` blocks release.

---

## 2. Versioning

Use semantic versioning:

```text
MAJOR.MINOR.PATCH
```

Examples:

```text
0.6.0
0.6.1
1.0.0-beta.1
1.0.0-dev.1
```

Version must match:

- `package.json`
- updater mock feed or real update feed
- release notes
- Git tag

---

## 3. Channel Promotion

Channels:

```text
stable
beta
dev
```

Promotion path:

```text
dev -> beta -> stable
```

Rules:

- `dev` builds may be unstable.
- `beta` requires passing E2E smoke tests.
- `stable` requires full manual regression pass and release certification.

---

## 4. Build Artifacts

Build all target platforms:

```bash
npm run dist:win
npm run dist:mac
npm run dist:linux
```

Expected artifacts:

- Windows:
  - NSIS installer
- macOS:
  - DMG or ZIP
- Linux:
  - AppImage
  - DEB

---

## 5. Signing and Notarization

### Windows

Requirements:

- Authenticode certificate.
- Signed installer.
- Signed main executable.

Manual signing example:

```bash
signtool sign /tr http://timestamp.sectigo.com /td sha256 /fd sha256 /a dist/Kaioken-Setup-x.y.z.exe
```

### macOS

Requirements:

- Apple Developer ID certificate.
- Hardened runtime.
- Notarization.

Manual notarization example:

```bash
xcrun notarytool submit dist/Kaioken-x.y.z.dmg \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_PASSWORD" \
  --wait
```

Then staple:

```bash
xcrun stapler staple dist/Kaioken-x.y.z.dmg
```

### Linux

No mandatory signing.

Optional:

- SHA256 checksums.
- GPG-signed release metadata.

---

## 6. Update Feed

For the mock updater, channel state is local.

For real release automation later:

1. Publish artifacts to update provider:
   - GitHub Releases, or
   - S3-compatible static feed, or
   - custom update server.
2. Publish channel metadata:
   - `latest.yml`
   - `latest-mac.yml`
   - `latest-linux.yml`
3. Ensure channel name matches:
   - stable
   - beta
   - dev

---

## 7. Installed Smoke Test

After installing the release artifact:

1. Boot app.
2. Open Settings.
3. Verify Security Posture shows nominal.
4. Run Release Certification.
5. Open Chat.
6. Create mock session.
7. Open Terminal drawer.
8. Open Wiki.
9. Open Codemap.
10. Open Cards.
11. Open Cost.
12. Open Research.
13. Check Release Notes modal.

---

## 8. Rollback Plan

If a release fails after publication:

1. Mark release as broken in release tracker.
2. Revert update feed to previous stable version.
3. Publish rollback notes.
4. Disable automatic update promotion for affected channel.
5. Create incident report.

---

## 9. Post-Release Monitoring

Check:

- Crash reports.
- Startup budget warnings.
- Memory guardrail warnings.
- Update install failures.
- Engine restart loops.

Release is considered healthy after 24 hours with no critical crash reports.
