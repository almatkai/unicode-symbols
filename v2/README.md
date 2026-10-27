# Unicode Symbols Search — Native V2 Rust

A standalone Rust Cargo library and concrete **compiled-in** Tezbar beta adapter for `index` / **Search Unicode Symbol**. Native V2 is schema version 2. Building this package alone does not add executable dispatch to Tezbar; vendor it, register the adapter/catalog, and rebuild the host as described in [INTEGRATION.md](INTEGRATION.md).

Source: `almatkai/unicode-symbols` at **8dd3d677cc89f7aae5e96d2d860aa38095d32b03**. Original Raycast extension: `6734d85992290066fd31a7f5bee3cca3d3545a1a/extensions/unicode-symbols`, originally by **mmazzarolo**, with FezVrasta, pernielsentikaer, j3lte, xilopaint, xmok and muhammadrizo. Own-source attribution remains almatkai. See [NOTICES.md](NOTICES.md) and [PROVENANCE.json](PROVENANCE.json).

The runtime has no JavaScript, Bun, dynamic loading or network search dependency. The source's minimal dataset (5,811 records / 41 blocks), full dataset (40,510 / 346), and HTML entity table (2,032 records, last duplicate wins) are embedded unchanged. Fuse **6.6.2**, actually locked by the source, is ported in Rust: extended queries, UTF-16 Bitap chunks, field norms, original weights, score products, stable ties, exact-character promotion, and section sorting. This includes the source's unusual decimal/hex `parseInt` precedence, negative/fractional limit behavior, uncapped empty-query block filters, and full dataset only in grid mode.

Preferences retain the original names/defaults: limit `100` (normal positive values capped at 1000), grid view, medium cells, minimal dataset, and primary paste. Small/medium/large request 8/5/3 columns. List mode keeps the source's Ancient Symbols substitution. Selector order is the source's executed English ICU `localeCompare` order, retained as a checked derived fixture for deterministic Windows/macOS output.

Original actions remain: copy/paste character, save as snippet, HEX / Unicode escape / HTML entity / decimal copies, block filter, favorites, recent history, and Compart / Unicode Explorer links. Source history insertion rules, no duplicate promotion, limits (10 recent / 50 favorites), saved `score` / `isExtra`, and section membership are retained. Storage is a separate Native V2 namespace; no legacy favorites/history migration is claimed. The actual beta `snippets:add` route backs a native title/trigger form, with character body bound to the selected canonical record. Snippet drafts survive Back.

The original command PNG is unchanged: **226,616 bytes**, SHA256 **7fe4a9da682e598b991633ad671a410b455401071551a3afa90006a7740744c4**. It was compared with bytes downloaded from the pinned original Raycast path. Public URL: `/extensions/icons/unicode-symbols.png`; host destination: `src/renderer/public/extensions/icons/unicode-symbols.png`.

## Validate

From this `v2/` directory:

```sh
cargo test --locked
cargo +1.85.0 test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo build --example oracle_bridge --locked
node tools/oracle.cjs --compare
node tools/renderer-schema.cjs
python3 tools/materialize-beta.py
cargo test --manifest-path .work/beta-host/Cargo.toml --locked
cargo clippy --manifest-path .work/beta-host/Cargo.toml --all-targets --locked -- -D warnings
python3 tools/verify-provenance.py
```

Node is used only for offline verification, against retained original TS, Fuse, React, and Ajv files; it is never invoked by the Rust runtime. Cargo dependencies may need a first fetch. After that, add `--offline`; the oracle and renderer/schema tests themselves need no downloads. See [VALIDATION.md](VALIDATION.md) for Windows checks, extracted-package verification, exact test coverage and limitations.

## Beta limitations

Both macOS and Windows integrated desktop verification are **not-run**, including host registration/rebuild. Headless parser, SSR, target checks and stubbed routes do not establish desktop UI or OS integration correctness.

The pinned beta grid flattens section headings, prefers the supplied dark image variant, and ignores item accessory/tooltips. Original tooltip data and the original supplemental SVG are retained, but those grid affordances cannot be displayed by this renderer. List aliases are carried as accessories. Its dropdown does not consume a programmatically supplied selected value, so action-driven filter changes can leave its selector label stale while results use the correct filter. Arbitrary source keyboard shortcuts and Raycast action-panel sections have no equivalent dispatch in the beta; actions remain in its action menu. Grid's existing favorite shortcut and standard navigation come from the host. The frontmost-app API is not used; paste is labeled “Active App”. There is no genuine native HUD/toast API; successful actions redraw the existing view after strict route acknowledgements.

Six original full-dataset boundary records contain unpaired UTF-16 surrogates. Their raw records/escapes are retained and verified, and SVG conversion matches the source's replacement glyph. The UTF-8 beta router cannot represent those character values for OS copy/paste/snippet bodies, so these actions reject explicitly; formatted escapes/HEX/decimal and browser references remain usable. If such a full-set item is saved and later shown through List favorites/history, its UTF-16 escape is displayed because the wire cannot contain the invalid UTF-8 value. Malformed UTF-16 query strings cannot traverse the beta's Rust JSON transport. These six records are never silently replaced in stored source data.

Copy retries are idempotent. If copy/paste succeeds but history persistence fails, retry resumes persistence without repeating the acknowledged effect. A transport/malformed acknowledgement or cancellation leaves paste/open/snippet outcome uncertain: it is not repeated automatically. Return to search and deliberately select a new action to retry. A definitive `snippets:add` validation rejection can be corrected and retried. Unknown/stale/cross-session IDs and malformed events preserve legitimate bindings. Refresh returns `unchanged` and never generates new IDs.
