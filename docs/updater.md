# Updater verification

Update checks use the running executable's installation channel. Standalone
executables query GitHub only; executables in the recognized WinGet package
directories query only the `winget` source with `winget show --versions`.
WinGet checks offer the newest published stable version newer than the running
version, even when GitHub has a later release. An equal or older source version
means the installation is up to date. Source failures remain errors and never
trigger a fallback to the other channel. WinGet upgrades use that same source.

The portable updater selects one exact asset from the configured repository's
latest stable GitHub release. Native x64 systems use
`claude-code-usage-monitor.exe`; native ARM64 systems use
`claude-code-usage-monitor-arm64.exe`. On an ARM64 system, an existing x64
install running under emulation selects the ARM64 asset after it has received
the migration-aware x64 version. The first release containing the new updater
acts as the x64 bridge; the next newer stable release performs the automatic
architecture migration. To migrate immediately, download the ARM64 asset and
replace the portable executable while keeping its local filename and path.

Architecture detection uses `IsWow64Process2` when available. If the API is
unavailable or fails, the updater records the failure in diagnostics and uses
the running binary's architecture. The selected download's PE machine type is
checked before the updater helper is launched and again by the helper before
replacing the installed executable. Missing or mismatched architecture assets
fail safely without falling back to another build.

The updater rejects duplicate selected assets, unexpected download URLs,
invalid semantic versions, drafts, and prereleases (including prerelease tags
incorrectly marked as stable). Version precedence follows SemVer:
`2.14.0-beta1` is below `2.14.0`, and build metadata does not affect ordering.

Every update requires a valid `sha256:` digest and a nonzero size of at most
100 MiB in the GitHub release asset metadata. The updater streams the download,
checks its actual byte count and SHA-256, and only promotes the temporary file
after both match. It stops reading after the expected size plus one byte,
regardless of Content-Length, and deletes its partial file on failure. Requests
and redirects require HTTPS.

The helper receives the expected size and digest, rechecks the staged file before
replacing the installed executable, and holds a Windows file handle that denies
writes and deletion through replacement. Missing or malformed verification
arguments are errors; the old helper invocation without a digest is rejected.

## Trust boundary

The digest comes from the authenticated GitHub API over TLS. This detects corrupt
or substituted downloads relative to that metadata. It is **not a publisher
signature** and does not protect against a compromised GitHub repository or
release account. There is no Authenticode or detached-signature verification in
this updater; adding it requires release signing and a trusted publisher key.
Code already running as the same Windows user is outside this trust boundary.

GitHub computes asset digests for uploaded release assets; see the
[GitHub release asset API](https://docs.github.com/en/rest/releases/assets).
Releases without a digest cannot be installed through the portable updater.
The release workflow does not need a separate checksum upload. WinGet continues
to perform installation through its own upgrade command.

## Tests

Run `cargo test --locked updater` on Windows. Tests exercise metadata selection,
architecture-specific asset selection, PE machine validation, SemVer
precedence, streamed size limits, known SHA-256 vectors, corrupt and truncated
bodies, interrupted and failed I/O, partial-file cleanup, helper arguments,
and Windows file locking. Tests do not download or execute updates. CI builds
and tests the x64 and ARM64 targets; the release workflow verifies each PE
machine type and publishes both assets only after both builds succeed.
Channel tests also cover source isolation, WinGet version parsing, and source
errors. To verify the live WinGet listing without installing an update, run
`cargo test --locked winget_lists_published_versions_from_the_live_source -- --ignored`.
