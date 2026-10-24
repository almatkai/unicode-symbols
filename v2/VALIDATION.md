# Validation record

Source SHA: **8dd3d677cc89f7aae5e96d2d860aa38095d32b03**. Beta contract: **fd300e276356587726566026f072563eb9c34cb1**. All work is in the authorized temporary clone on `feat/native-v2-rust`, based on fetched `origin/main`. No commit, staging, push, PR, host/profile/ledger edits, or real OS effects were performed. V1 tracked bytes are retained; root README only adds the discovery link.

Executed on 2026-10-24, macOS ARM64, Rust stable **1.98.0**, Cargo **1.98.0**, MSRV **1.85.0**, Node **26.7.0**. “macOS” here identifies the test machine, not integrated desktop QA.

| Check | Result |
| --- | --- |
| `cargo fmt --check`, explicit adapter/harness rustfmt | Pass |
| Strict all-target Clippy, stable, core and beta adapter harness | Pass |
| Core tests, stable and 1.85.0 | 8 pass on each |
| Production adapter / actual NativeSessions contracts, stable and 1.85.0 | 9 pass on each |
| Unchanged beta `common.rs` / `v2_sdk.rs` unit tests | 5 pass on each |
| Windows GNU cross check and strict all-target Clippy, core and adapter harness | Pass; compilation only |
| Executed original TS + actually locked Fuse 6.6.2 oracle | 499 cases / 81 distinct searches pass |
| Entire original datasets compared record for record via executable oracle | 46,321 records pass, including all six raw surrogate records |
| Original action panel / persistence hook oracle | 7 panels plus insertion/dedup/removal/clear/limit checks pass |
| Original symbol SVGs and numeric formats | Matched; both `RAYCASTX=0` and `1` source formatting retained |
| Unchanged beta renderer test files | 14 pass |
| Real List/Grid/Detail/Form parser + React SSR wire contracts | 4 pass |
| Full unchanged Draft 2020-12 schema via Ajv | Valid manifest plus 65 negative cases pass |
| Icon, source fixture, lock-version and actual license-text verification | Pass; 711 evidence files hash-pinned |
| Disposable V1 original-lock smoke test / TypeScript typecheck | 1 test pass / pass |
| Cargo package and extracted-package offline checks | Pass; all evidence files present; core stable/MSRV tests, executable oracle, renderer/schema, beta tests and strict harness Clippy pass offline |
| Real host vendoring, registration, rebuild and native CLI coverage | **Not-run** |
| Real macOS and Windows desktop UI / OS QA | **Not-run** on both |

Run `sh tools/validate.sh` from this package. The Windows checks require the `x86_64-pc-windows-gnu` Rust standard-library target installed; they do not link/run a Windows desktop application. Run `python3 tools/package-check.py` for the extracted-package check. The package includes template manifests and retained locks, not nested Cargo test crates that Cargo would silently omit. The byte-exact beta manifest is retained as `fixtures/beta/src-tauri/Cargo.toml.fixture` so it does not turn that fixture directory into an excluded nested Cargo package. The script materializes those under its own `.work/` directory, then builds/tests offline using Cargo's already-fetched cache. No runtime downloads are needed.

## Oracle evidence

`tools/oracle.cjs` compiles and executes the **byte-exact original** own `dataset-manager.ts`, `list.ts`, preferences, formatting, action JSX, and persistence hooks using the retained TypeScript 5.9.3 compiler. It loads the retained original Fuse 6.6.2 distribution, not an expected-value reimplementation. Raycast/react hook/JSX/environment boundaries are recording stubs. The standalone Rust example is driven with the same requests. The oracle compares exact original fields and sequence, scores within `1e-11`, section order/membership, preference results, SVGs, action values and callbacks. Snapshot files record its output, and Rust tests also consume the ranking snapshots; the executable oracle comparison is run independently.

The matrix covers list/grid × minimal/full × all three cell sizes × both primary actions; typo/fuzzy, extended syntax and long patterns; alias, old-name and numeric matching; whitespace, repeated spaces, quotes, OR and non-ASCII/astral/combining inputs; decimal/hex/escape precedence; absent/missing matches; block filters/extras; favorites/recent sections; zero, invalid, fractional, negative, hexadecimal and infinite limit strings. Defaults and data-generation records come from the actual own source/lock, rather than upstream dependency ranges. Full-data mode stays disabled in list view as in the source. Selector `localeCompare` order is checked against the source block names.

## Beta fixtures and safety coverage

The production adapter compiles with byte-exact beta `common.rs`, `v2_sdk.rs`, `native.rs` and the registered dispatch shape. The harness uses actual beta-locked serde 1.0.228, serde_json 1.0.150, tokio 1.52.3, tempfile 3.27.0, log 0.4.32 and uuid 1.23.3. The core has its own exact Cargo dependency versions/lock. Fixture files retain their original hashes.

The Tauri app/Emitter, installed-registry boundary and unrelated adapters are labeled headless stubs. Runtime registration is generated from the pinned original `v2_runtime.rs`, adding only the Unicode slug/commands/handler. The full original `native_tests.rs` is retained as evidence but its unrelated ExtensionsCore/Perplexity suite is outside this harness; a test-import marker replaces that include **in the materialized harness only**. Custom external tests drive the unchanged NativeSessions implementation with the production Unicode adapter. All five original common/SDK unit tests execute unchanged.

The nine contract tests cover canonical/legacy aliases and real launch/dispatch/refresh/disposal/uninstall; actual payload preferences and ignored argumentValues; stale/forged/cross-session/discarded-search IDs; malformed events preserving legitimate IDs; refresh clone safety; strict null/boolean/contradictory/transport acknowledgements; idempotent copy retry; uncertain paste/open preventing duplicate effects; successful effect followed by corrupt-storage failure and persistence-only retry; native cancellation of a pending effect; snippet validation, canonical body, draft updates and Back; definite snippet rejection versus uncertain acknowledgement; and SDK concurrent atomic updates preserving unrelated keys. Source recent/favorite metadata and no-promotion limits are separately tested in the core. Routers only record requests; none operates an OS clipboard, browser, window, focus or paste.

Renderer tests execute the unchanged List/Grid/Detail/Form modules with actual beta-locked React/react-dom 18.3.1. Test-only export instrumentation exposes private parser functions without changing fixture bytes. UI primitives, Markdown and unrelated color conversion helpers are stubs; action/runtime-hook boundaries are stubbed for the baseline tests. These are parser/SSR checks, not browser interaction, pixel, font, keyboard, desktop or OS verification. The Detail contract is a baseline synthetic parser fixture; the Unicode command itself uses List/Grid and the snippet Form. `__hasServerSearch` and `filtering:false` ensure a fuzzy/hex result is retained even when its title does not contain the query.

## Remaining limitations and review boundaries

See README for the pinned beta's flattened grid sections, preferred dark image variant, ignored grid accessories/tooltips, incomplete shortcut/action grouping, list dropdown programmatic-label gap, generic Active App label, and absence of a genuine HUD/toast API. Save as Snippet adapts to the beta's title/trigger form with a canonical fixed character body; the actual beta store requires nonempty/non-whitespace body and trigger, so whitespace-only characters can be rejected by that route. No favorites/history migration from unknown V1 storage is asserted. Six unpaired surrogate character values cannot cross its UTF-8 OS router; raw data and formatting actions remain intact, with escaped surrogate values used in List favorites/history. Unicode font/glyph availability and performance on both real desktops remain unverified.

XERJ was attempted first with canonical-root prefix **`ax-4b3371635441`** and isolated workspace state. Initial indexing was bounded at 35 seconds and stopped only this task's process after HTTP 429 / RSS watermark rejection; search returned no useful matches. A final bounded retry found the global server unreachable. It was never restarted or modified. Index coverage is **incomplete**; exact source/data checks used direct reads and exhaustive `rg`/hash comparisons.

The parent must inspect the full diff, stage and run `python3 v2/tools/verify-provenance.py --git --index` to verify the eventual staged evidence hashes. Root `dist/` ignore rules required explicit unignore rules for vendored oracle/validator distributions. `--git` checks visibility without staging. Integration/rebuild and both platform verification entries remain not-run until the parent produces real QA evidence.
