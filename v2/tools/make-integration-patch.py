#!/usr/bin/env python3
from pathlib import Path
import json,difflib
root=Path(__file__).resolve().parent.parent
base=root/'fixtures/beta';changes={}
f='src-tauri/src/native_core/extensions/mod.rs';s=(base/f).read_text();changes[f]=s.replace('mod v2_sdk;','mod v2_sdk;\n#[path = "../../../native-extensions/unicode-symbols/adapter/mod.rs"]\nmod v2_unicode_symbols;')
f='src-tauri/src/native_core/extensions/v2_runtime.rs';s=(base/f).read_text();s=s.replace('"perplexity" => Some(value)','"perplexity" | "unicode-symbols" => Some(value)');s=s.replace('        _ => &[],','        "unicode-symbols" => super::v2_unicode_symbols::commands(),\n        _ => &[],');s=s.replace('        _ => Err(format!("No native adapter for {slug}")),','        "unicode-symbols" => super::v2_unicode_symbols::handle(ctx, command, operation, payload, state).await,\n        _ => Err(format!("No native adapter for {slug}")),');changes[f]=s
f='src-tauri/src/native_core/extensions/v2_catalog.json';items=json.loads((base/f).read_text());items.append(json.loads((root/'extension-v2.json').read_text()));changes[f]=json.dumps(items,ensure_ascii=False,indent=2)+'\n'
f='src-tauri/src/native_core/extensions/v2_catalog.rs';changes[f]=(base/f).read_text().replace('assert_eq!(items.len(), 11);','assert_eq!(items.len(), 12);')
f='docs/NATIVE_V2_FORMAT.md';changes[f]=(base/f).read_text().replace('the eleven installed extension sources','the twelve extension sources (including the compiled-in Unicode Symbols Search port)').replace('It contains no saved preferences','The catalog declares 85 command entries after this addition (previously 84); compiled dispatch still determines actual platform coverage. It contains no saved preferences')
patch=''.join(''.join(difflib.unified_diff((base/f).read_text().splitlines(True),new.splitlines(True),fromfile='a/'+f,tofile='b/'+f))for f,new in changes.items())
(root/'integration.patch').write_text(patch)
print('Generated review patch against fd300e276356587726566026f072563eb9c34cb1 (does not modify a host checkout)')
