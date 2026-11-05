# Standalone Rust package

The root package is schema 3, process protocol 1, version 0.1.0, canonical ID
`com.tezbar.native.unicode-symbols`. The original `package.json` name is
`unicode-symbols`, so its replacement is `raycast.unicode-symbols`.
`gh release list --repo almatkai/unicode-symbols` returned no releases when selecting
this first native package version. Nothing in this implementation publishes it.

The production executable is `bin/tezbar-unicode-symbols` on macOS arm64 and
`bin/tezbar-unicode-symbols.exe` on Windows x64. The root Cargo workspace imports
its own existing `v2/` engine by path and the independently published
`tezbar-extension-sdk` revision `aaec571673ab64a85a136cf6bd054ef660c4d93c` by Git.
Cargo.lock is committed. Rust 1.85 is the minimum, with MSRV-aware dependency
resolution in `.cargo/config.toml`. The downloaded SDK API, not a local path crate,
is used by `native/main.rs` and its `serve()` entrypoint.

## Command and behavior

The one original command, `index` / **Search Unicode Symbol**, is a native view
command on macOS and Windows. `v2/src/lib.rs`, `search.rs`, and `session.rs` retain
the full offline engine: 5,811 minimal / 40,510 full records, 2,032 HTML entities,
Fuse 6.6.2 UTF-16 Bitap/extended search, original section/score ordering, exact
character promotion, numeric query parsing, source preference limits, favorites,
history and item metadata. Full dataset remains grid-only, as in the original.

All five source preferences are copied unchanged to extension.json:
`searchResultLimitStr`, `viewType`, `gridItemSize`, `dataSet`, `primaryAction`.
None is a checkbox. Original TypeScript, its manifest, command names and icons
are preserved; their code is never a runtime subprocess.

Copy/paste character, HEX, Unicode escape, HTML entity, HTML decimal, Compart and
Unicode Explorer links, block filtering, favorite add/remove/clear, and recent
remove/clear are available through original bound action IDs. Original insertion,
limits and per-action history callbacks are retained. The process uses only typed
`host.clipboard.write`, `host.clipboard.pasteText`, and `host.shell.open` requests;
those exact capabilities are granted. Reads, window control and unrelated services
are not needed. Acknowledgements use actual host object shapes (`{ok:true}` and
`{ok:true,message:"Pasted text"}`), with malformed replies tested separately.

Search/refresh/disposal use the SDK's transactional session state and cancellation.
Successful action responses return complete views, never unchanged action envelopes.
Failures return protocol errors, which the current renderer displays. Copy messages
are sent only after an acknowledged effect and completed history persistence.
An effect ledger outside transactional state preserves acknowledged or uncertain
OS operations if cancellation or storage failure discards staged state. An uncertain
paste/open is not automatically repeated; return to search and deliberately choose
a fresh binding. Idempotent copies can retry. Refresh preserves action bindings and
returns unchanged. Dispose clears session-owned effect records; shutdown is SDK-owned.

Large views are bounded to 150 rows per page using `Session::render_window`.
The original unwindowed `render` remains for the compiled-in adapter and original
oracle tests. Next/Previous Page actions are included in every visible row's
`actionIds`; they do not change search results or ordering. Search and filter changes
reset the page. This also covers original uncapped empty-query block filtering.

## Explicit gaps and QA limits

- **Save as Snippet:** the current process host allowlist has no snippet creation
  service. The original action remains visible and returns a clear unsupported
  error without dispatching a fabricated host call or showing false success.
- Six original unpaired UTF-16 boundary records retain their canonical identities
  and escape data. Their glyph copy/paste rejects explicitly because JSONL/OS
  text is UTF-8. Format copies and reference links still work. Legacy favorites
  containing these records migrate by code without decoding their invalid glyph.
- The current grid renderer flattens section headings, selects the dark SVG image
  variant, and does not render original grid accessory tooltips. The engine keeps
  those original fields. Original custom Raycast action-panel sections/shortcuts
  are not represented by the native view schema; all actions are in row menus.
  Paste targets the active application rather than naming it in the title.
- Real desktop QA remains **unverified on both platforms**. Headless protocol,
  schema and renderer tests do not prove actual clipboard, paste, browser or UI
  behavior. Windows binary build/smoke is handed off to central coordination;
  including Windows dependency notices does not establish Windows coverage.

## Saved data and preferences

Only declared, host-granted migration files are read, never scanned profile roots:

| Grant | Profile-relative file |
|---|---|
| nativeStore | native-extensions-v2/unicode-symbols/storage.json |
| legacyLocalStorage | extensions/unicode-symbols/.tezbar-local-storage.json |
| legacyPackageLocalStorage | extensions/packages/raycast.unicode-symbols/.tezbar-local-storage.json |
| legacyShimStorage | extensions/unicode-symbols/localStorage.json |
| legacyPackageShimStorage | extensions/packages/raycast.unicode-symbols/localStorage.json |

The current JS view runner writes `.tezbar-local-storage.json`; the older no-view
shim writes `localStorage.json`, both under the extension package root. Its values
are JSON strings. The source uses `favorites-v1` and `recently-used-v2`, containing
character objects with optional `score` / `isExtra`. Migration validates canonical
codes, duplicates, limits and metadata, retaining source order and converting to
v2 numeric identities/metadata. Source files remain untouched. Existing private
lists win, then native v2, then declared JS files in table order. Each list and its
metadata move together so a lower-priority file cannot introduce orphan metadata.
Malformed sources abort without a completion marker or partial private import.
An atomic `_processMigrationV1` marker makes import idempotent. Private storage is
`<host-granted dataDir>/unicode-symbols/storage.json` via SDK atomic updates.

Nonstandard JS development package locations are outside these narrow grants and
cannot be auto-imported. No direct Raycast-on-disk storage access is claimed.
Preferences are supplied by the host on session open; the package keeps their
source names/defaults so the host's ordinary replacement/preference fallback applies.

## Build and verification

Run from this repository; no global Cargo configuration or shared target is needed:

```sh
export CARGO_TARGET_DIR="$PWD/native-target"
export CARGO_BUILD_JOBS=2
cargo +1.85.0 test --workspace --locked
cargo +1.85.0 clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo +1.85.0 build -p tezbar-unicode-symbols --example oracle_bridge --locked
node v2/tools/oracle.cjs --compare
node v2/tools/renderer-schema.cjs
python3 v2/tools/verify-provenance.py
node scripts/verify-manifest.cjs
python3 scripts/vendor-licenses.py --check
cargo +1.85.0 build --release --locked --target aarch64-apple-darwin
python3 scripts/package.py --target aarch64-apple-darwin \
  --binary native-target/aarch64-apple-darwin/release/tezbar-unicode-symbols
python3 scripts/smoke-package.py dist/unicode-symbols-aarch64-apple-darwin.zip
```

The original oracle regenerates and compares its tracked snapshots. Node is used
only for offline schema/oracle/renderer tooling. The retained beta SSR tests use
headless stubs and never require materializing/building the old beta app. The
schema checker uses the full copied schema 3 from the read-only host reference
(commit d77f03b12a93d72cd2a56a73faa1a04fae94a2cd) and offline vendored Ajv.

Package ZIPs contain exactly extension.json, bin/, resources/, licenses/, LICENSE
and NOTICE files. The packaged manifest inventories every other member by SHA256;
it cannot hash itself. Archive SHA256 covers the manifest too. Packaging verifies
the immutable SDK pin/lock, manifest/source parity, executable identity and command
coverage, original icon, and all supplied dependency license/notice documents for
the locked normal/build closure of both target platforms. Run
`python3 scripts/vendor-licenses.py` after an intentional lockfile change and review
its source documents. All supplied alternative license texts and SDK NOTICE ship.

`dist/` receives `<slug>-<target>.zip`, `.zip.sha256`, `.artifact.json`,
`.executable.sha256`, and `.files.json`. No cross-target execution is simulated:
Windows packaging must run with a real Windows executable on its build host.
The smoke script extracts to a new directory with a minimal environment and drives
hello/initialize/open/call/dispose/shutdown, migration, Grid/List search, every
supported action family, negative host replies, forged/stale/cross-session IDs,
real protocol cancellation, refresh, and multi-page block results. All reverse RPC
calls are recording fakes; it neither installs nor touches the user's desktop.

## Install handoff

The ZIP manifest intentionally has empty `artifacts`: an archive cannot include its
own final SHA. After independent review, the release coordinator supplies the actual
per-target archive URL/SHA256 and executable SHA256 in a trusted release manifest.
The host's `--extensions-v3-import /absolute/trusted-release-manifest.json
/absolute/package.zip` validates that reference plus the closed ZIP inventory.
A combined trusted reference may use empty `files` to derive each verified target's
inventory. Catalog approval/release publication is separate; this implementation
performs no push, tag, release, host edit or install. Close/reopen Tezbar after an
explicit import. The package is not remotely installable until approved release
artifacts and the trusted host reference/catalog entry are supplied.
