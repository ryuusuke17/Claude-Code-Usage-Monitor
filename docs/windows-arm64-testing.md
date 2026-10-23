# Windows 11 ARM64 validation

Use this checklist on a Windows 11 ARM64 desktop after the native release build
passes CI. Keep a copy with the release evidence and record any skipped check.

## Record the test environment

- Windows edition and build:
- ARM device / processor:
- App version and ARM64 asset SHA-256:
- Test date:

## Native app checks

- [ ] Launch the ARM64 executable and confirm it runs as ARM64.
- [ ] Confirm the taskbar widget and tray icon appear and respond to their menu,
  click, and refresh actions.
- [ ] Open the dashboard and Theme Studio. Confirm OpenGL rendering works; force
  the existing WARP fallback and confirm it presents a frame.
- [ ] Confirm text clipboard copy and paste work in the dashboard.
- [ ] Confirm enabled providers can read their credentials, SQLite data, and
  usage responses without a panic or unexpected file change.
- [ ] Check display scaling, multiple monitors, taskbar docking, and placement
  after Explorer restarts.
- [ ] Enable startup, sign out or restart Windows, and confirm the app starts.

## Update and compatibility checks

- [ ] Run an x64 portable build under ARM emulation and update it to the
  migration-aware x64 version.
- [ ] On a newer stable release, confirm the portable updater selects ARM64,
  replaces the same executable path, restarts the app as ARM64, and retains
  settings, themes, credentials, and startup behavior.
- [ ] Update ARM64 to a newer ARM64 release and confirm the same settings remain.
- [ ] Confirm a mismatched or corrupt asset is rejected before replacement and
  leaves the installed executable unchanged.
- [ ] Repeat x64 launch and portable update smoke checks on a native x64 system.

## Results

Passed checks:

Failed checks and diagnostics:

Skipped checks and reason:
