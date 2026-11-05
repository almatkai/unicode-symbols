# Unicode Symbols Search

Browse and copy-paste [Unicode 17.0.0](https://www.unicode.org/versions/Unicode17.0.0/) symbols.

## Installable native Rust extension

The root [extension.json](extension.json) describes the standalone schema 3 process
package, **com.tezbar.native.unicode-symbols**, replacing **raycast.unicode-symbols**.
Its `index` / **Search Unicode Symbol** command uses the mature [v2 Rust engine](v2/README.md)
with the original datasets, Fuse 6.6.2 search, preferences and command icon.
The original Raycast TypeScript source and package manifest remain in this repository.

The native package contains a Rust executable, resources and third-party notices;
it requires no JavaScript/Bun process, Tezbar app crate, or beta fixture materialization.
See [NATIVE.md](NATIVE.md) for builds, package verification, migration and install handoff.
Real macOS/Windows desktop QA remains unverified. **Save as Snippet explicitly reports
unsupported** because the process host has no snippet creation API. The six original
unpaired UTF-16 records cannot be copied/pasted as characters over UTF-8; format copies
and browser references remain usable. Large result sets use 150-row pages with
Next/Previous Page actions, preserving search order and access to every result.

## Configuration

### Data Set

By default the Data Set is set to "Minimal" which includes common symbols and only has ~5400 results. This is only a subset of the full Unicode 17 dataset.
If you cannot find the symbol you are looking for, please set this configuration to "Full" and try again. This set is +40000 symbols big and might impact performance.

> Please note that if you are using the "List view" instead of "Grid", only the "Minimal" dataset is supported due to technical reasons.

Native V2 compiled-in Rust port: [v2/README.md](v2/README.md).
