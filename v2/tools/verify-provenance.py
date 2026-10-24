#!/usr/bin/env python3
from pathlib import Path
import argparse,hashlib,json,subprocess
root=Path(__file__).resolve().parent.parent
args=argparse.ArgumentParser();args.add_argument('--git',action='store_true',help='check required paths visible to git');args.add_argument('--index',action='store_true',help='also verify eventual staged hashes (does not stage)');options=args.parse_args()
p=json.loads((root/'PROVENANCE.json').read_text())
for entry in p['files']:
 f=root/entry['path'];data=f.read_bytes();assert len(data)==entry['bytes'],entry['path'];assert hashlib.sha256(data).hexdigest()==entry['sha256'],entry['path']
sha='7fe4a9da682e598b991633ad671a410b455401071551a3afa90006a7740744c4'
for f in ['icons/unicode-symbols.png','fixtures/upstream/assets/command-icon.png','fixtures/own/assets/command-icon.png']:
 data=(root/f).read_bytes();assert len(data)==226616 and hashlib.sha256(data).hexdigest()==sha,f
lock=json.loads((root/'fixtures/own/package-lock.json').read_text())
for name,version in p['source']['resolved'].items():
 assert lock['packages']['node_modules/'+name]['version']==version,name
 vendor=root/'fixtures/vendor'/name/'package.json'
 if vendor.exists():assert json.loads(vendor.read_text())['version']==version,name
check=[('fuse.js/LICENSE','Apache License'),('unidata16/LICENSE','MIT License'),('typescript/LICENSE.txt','Apache License'),('typescript/ThirdPartyNoticeText.txt','Copyright (c) 1991-2017 Unicode')]
for f,phrase in check:assert phrase in (root/'fixtures/vendor'/f).read_text(),f
for file in (root/'fixtures/vendor/npm').glob('*/package.json'):
 package=json.loads(file.read_text());assert package['license'] in ['MIT','BSD-3-Clause'];licenses=[f for f in file.parent.iterdir()if f.name.lower()in ['license','license.txt','license.md']];assert licenses,file
 text=licenses[0].read_text();assert ('Permission is hereby granted' in text)if package['license']=='MIT' else ('Redistribution and use' in text),file
assert 'UNICODE LICENSE V3' in (root/'licenses/Unicode-3.0.txt').read_text()
assert 'BSD 3-Clause License' in (root/'licenses/WHATWG-LICENSE.txt').read_text()
manifest=json.loads((root/'extension-v2.json').read_text());assert 'author' not in manifest
own=json.loads((root/'fixtures/own/package.json').read_text());assert manifest['preferences']==own['preferences'];assert [{k:c[k]for k in own['commands'][0]}for c in manifest['commands']]==own['commands']
if options.git or options.index:
 gitroot=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=root,text=True).strip());prefix=root.relative_to(gitroot).as_posix()
 visible=set(subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=gitroot).decode().split('\0'))
 required=[root/f['path']for f in p['files']]+[root/f for f in ['Cargo.toml','Cargo.lock','adapter/mod.rs','extension-v2.json','README.md','INTEGRATION.md','VALIDATION.md','NOTICES.md','PROVENANCE.json','integration.patch']]
 for f in required:
  rel=f.relative_to(gitroot).as_posix();assert rel in visible,'Ignored required file: '+rel
 if options.index:
  for f in p['files']:
   rel=prefix+'/'+f['path'];data=subprocess.check_output(['git','show',':'+rel],cwd=gitroot);assert hashlib.sha256(data).hexdigest()==f['sha256'],'Staged mismatch: '+rel
print('Provenance, exact icons, lock versions and actual license texts passed:',len(p['files']),'files',('and Git visibility'if options.git else ''))
