# Compile into the pinned beta host

Reference contract: Tezbar commit **fd300e276356587726566026f072563eb9c34cb1**. This repository contains a compiled-in adapter, not a dynamically installable extension. The host checkout was read only during implementation; these steps remain **not-run** and belong to the parent's review/release workflow.

1. Vendor the complete distributable subtree at **`src-tauri/native-extensions/unicode-symbols`**. Keep paths stable: the adapter includes `../src/lib.rs`; that library embeds original assets from `../fixtures/own/assets` and the derived selector order. For a working copy, use:

   ```sh
   rsync -a --exclude target --exclude .work --exclude node_modules /path/to/unicode-symbols/v2/ /path/to/Tezbar/src-tauri/native-extensions/unicode-symbols/
   ```

   Alternatively extract the Cargo package there. Do not copy a `target/` or node_modules directory. Retain notices and provenance when redistributing the vendored source.

2. The host already uses serde/serde_json; enable **`serde_json`'s `raw_value` feature**, and ensure `base64 = "0.22.1"` is available. The retained fixture manifest shows the beta's actual locked dependency versions. This adapter uses existing CoreContext, `v2_sdk::copy/open/read_store/update_store`, and the real router. No Tauri API change is necessary. It is compiled as a sibling of `v2_sdk`, from `src-tauri/src/native_core/extensions/mod.rs`:

   ```rust
   #[path = "../../../native-extensions/unicode-symbols/adapter/mod.rs"]
   mod v2_unicode_symbols;
   ```

3. In `v2_runtime.rs`, add `unicode-symbols` to `slug`; add its `commands()` arm; and delegate its `handle(ctx, command, operation, payload, state)`. Copy `extension-v2.json` as a new object into `v2_catalog.json`. Use only its actual `index` command metadata. Canonical ID is `com.tezbar.native.unicode-symbols`, replacement is `raycast.unicode-symbols`, runtime is `rust`, intended platforms are macOS/Windows, and both verification records stay `not-run`. Root `author` is intentionally absent; credits are in description/notices.

4. Copy **`icons/unicode-symbols.png`**, byte for byte, to **`src/renderer/public/extensions/icons/unicode-symbols.png`**. The catalog URL must be **`/extensions/icons/unicode-symbols.png`**. Verify SHA256 `7fe4a9da682e598b991633ad671a410b455401071551a3afa90006a7740744c4` and size 226616 before/after copying. Preserve `fixtures/own/assets/exclamation-outline.svg`; the pinned grid does not render item accessories, so no extra visible host artwork is claimed.

5. Update coverage assertions: pinned `v2_catalog.rs::bundled_ids_and_coverage_are_consistent` expects **12** packages instead of 11; the catalog has **85 declared commands** instead of 84. Update `docs/NATIVE_V2_FORMAT.md` from eleven to twelve sources, preserving its distinction between declaration, compiled platform coverage and real verification. V1's 98 own entries / 247 commands are unchanged. Recheck any other host assertions against the actual final diff.

   [integration.patch](integration.patch) contains these exact module/runtime/catalog/coverage/documentation edits against the pin. It does not include the vendor subtree, binary icon, or dependency feature change. Regenerate with `python3 tools/make-integration-patch.py`; review/apply it in the parent's chosen host branch after checking for subsequent host changes. Do not apply blindly to a different beta.

6. Rebuild the host Rust native core and frontend with that host's documented toolchain (the pin uses `pnpm build:native && tauri build` via `pnpm build`). If extending host dependencies, update its lockfile through Cargo and review the resulting diff. Then run the compiled native coverage CLI / `scripts/extensions-v2-list.mjs` against the newly built binary, install the canonical catalog entry, and verify dispatch actually exists. Copying JSON or a marker into a profile does not supply executable code.

7. Perform genuine macOS **and** Windows QA before changing verification: each dataset/view/cell size/primary action; fuzzy and exact queries; favorites/recent persistence and limits; filter changes; snippet title/trigger/draft/Back; confirmed copy/paste into a separate app; canonical browser URLs; rejected routes/permissions; refresh/search cancellation; uninstall/reinstall and reversible V1 fallback. Validate glyph/font availability and beta grid/accessory/shortcut limitations on both desktops.

## Wire and effect contract

`handle` accepts `index` only and the real beta operations `extension:run-command`, `extension:search-text-changed`, `extension:invoke-action`, `extension:refresh-session`, `extension:dispose-session`. Preferences are read from `payload.preferences`; the command declares no arguments, so actual `argumentValues` is ignored. Search uses `payload.searchText`. Dropdown and form changes arrive through invoke-action with `formValues.value`; snippet submit carries `formValues.label/trigger`. There is no invented form-field IPC operation.

Results use actual `type/props/children`, per-row `actionIds`, a `List.Dropdown` accessory, `__hasServerSearch: true` and `filtering: false`, so the client does not re-filter/reorder server results. Form `backActionId` preserves drafts; submit is `kind: "submit-form"`. Refresh returns the SDK's unchanged response, without mutations or fresh IDs, even when called on a disposable state clone. Every new render replaces bindings using a process-random nonce plus monotonic counter; discarded search clones cannot mint IDs that revive later.

Effects use `clipboard:write [text]` via SDK, `core:platform:paste-text [text]`, `shell:open [target]` via SDK, and the pinned launcher's `snippets:add [{label, trigger, body}]`. Paste-text owns clipboard write/hide/focus restoration/wait/paste; the adapter never follows it with another paste operation. It neither reads the clipboard nor sends a redundant `core:platform:paste`. Text and URLs are derived from the bound canonical character; arbitrary event `text`, `url`, or snippet `body` is ignored. Strict success means an object with `ok:true`, no nonnull `error`, and `success` absent or true. The external result is never inferred from a null/boolean/contradictory response.

## Storage

The real SDK owns `<data_dir>/native-extensions-v2/unicode-symbols/storage.json`. No host profile or legacy V1 store is read/written by this port. Legacy preferences may be resolved by the host registry before delivery; no source favorites/history migration is asserted. Native keys are `favorites-v1` (canonical codepoint IDs), `recently-used-v2`, and their `favorites-meta-v1` / `recently-used-meta-v2` maps for saved score/extra markers. Updates go through the pinned SDK's process-atomic lane, preserve unrelated object keys, reject corrupt arrays/metadata, and do not overwrite malformed JSON. No other storage is introduced.
