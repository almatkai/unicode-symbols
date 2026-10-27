#!/usr/bin/env python3
"""Materialize packaged template manifests. No access to a host checkout/profile."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parent.parent
host=root/'.work/beta-host';src=host/'src';ext=src/'native_core/extensions'
ext.mkdir(parents=True,exist_ok=True)
(host/'tests').mkdir(exist_ok=True)
shutil.copyfile(root/'tools/beta-host.Cargo.toml.in',host/'Cargo.toml')
for file,dest in [('src-tauri/src/native_core/common.rs',src/'native_core/common.rs'),('src-tauri/src/native_core/extensions/v2_sdk.rs',ext/'v2_sdk.rs'),('src-tauri/src/native_core/extensions/native.rs',ext/'native.rs')]:
 shutil.copyfile(root/'fixtures/beta'/file,dest)
runtime=(root/'fixtures/beta/src-tauri/src/native_core/extensions/v2_runtime.rs').read_text()
runtime=runtime.replace('"perplexity" => Some(value)','"perplexity" | "unicode-symbols" => Some(value)')
runtime=runtime.replace('        _ => &[],','        "unicode-symbols" => super::v2_unicode_symbols::commands(),\n        _ => &[],')
runtime=runtime.replace('        _ => Err(format!("No native adapter for {slug}")),','        "unicode-symbols" => super::v2_unicode_symbols::handle(ctx,command,operation,payload,state).await,\n        _ => Err(format!("No native adapter for {slug}")),')
assert runtime.count('"unicode-symbols"')==3
(ext/'v2_runtime.rs').write_text(runtime)
# The parent ExtensionsCore/Perplexity suite needs the entire host. Preserve it in
# fixtures, and use only its import marker here; external contract tests drive the
# unchanged NativeSessions implementation with the production Unicode adapter.
(ext/'native_tests.rs').write_text('const _:()=super::perplexity::HEADLESS;\n')
shutil.copyfile(root/'tools/registry-stub.rs',ext/'registry.rs')
mods='''mod native;
#[cfg(test)] mod perplexity {pub const HEADLESS:()=();}
mod v2_sdk;
mod registry;
mod v2_runtime;
#[path="../../../native-extensions/unicode-symbols/adapter/mod.rs"]
mod v2_unicode_symbols;
'''
for name in ['amphetamine','audio_device','cleanshotx','color_picker','gif_search','google_calendar','google_search','port_manager','speedtest','timers','perplexity']:
 mods+='mod v2_'+name+' { pub fn commands()-> &\'static [&\'static str]{ &[] } pub async fn handle(_: &crate::native_core::common::CoreContext,_:&str,_:&str,_:&str,_:&serde_json::Value,_:&mut serde_json::Value)->crate::native_core::common::CoreResult{Err("Unrelated adapter is outside this fixture".into())} }\n'
# Original handlers accept five args; remove the extra argument from the unrelated stub signature.
mods=mods.replace('_:&str,_:&str,_:&str,_:&serde_json::Value','_:&str,_:&str,_:&serde_json::Value')
mods+='''
pub use v2_unicode_symbols::handle as adapter;
pub use v2_sdk::{read_store,update_store};
#[derive(Clone,Default)] pub struct Sessions { inner:native::NativeSessions,files:std::sync::Arc<std::sync::Mutex<()>> }
impl Sessions { pub async fn call(&self,ctx:&crate::native_core::common::CoreContext,channel:&str,body:serde_json::Value)->crate::native_core::common::CoreResult {self.inner.handle(ctx,&self.files,channel,&[body]).await.ok_or("Unhandled fixture operation")?} pub async fn clear(&self){self.inner.clear().await;} }
pub fn registered(id:&str,command:&str)->bool{v2_runtime::supports(id,command)}
'''
(ext/'mod.rs').write_text(mods)
(src/'lib.rs').write_text('#![allow(dead_code)]\npub mod native_core { pub mod common; pub mod extensions; }\n')
link=host/'native-extensions/unicode-symbols';link.parent.mkdir(exist_ok=True)
if not link.exists():link.symlink_to(root,target_is_directory=True)
stub=host/'tauri-stub';(stub/'src').mkdir(parents=True,exist_ok=True)
(stub/'Cargo.toml').write_text('[package]\nname="tauri"\nversion="2.0.0"\nedition="2021"\n[dependencies]\nserde_json="=1.0.150"\n')
shutil.copyfile(root/'tools/tauri-stub.rs',stub/'src/lib.rs')
shutil.copyfile(root/'tools/beta-tests.rs',host/'tests/contract.rs')
if (root/'tools/beta-host.lock').exists():shutil.copyfile(root/'tools/beta-host.lock',host/'Cargo.lock')
print(host)
