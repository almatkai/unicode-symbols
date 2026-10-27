#!/usr/bin/env python3
"""Inventory retained evidence only; this does not fetch or stage anything."""
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent.parent
own='8dd3d677cc89f7aae5e96d2d860aa38095d32b03';beta='fd300e276356587726566026f072563eb9c34cb1';up='6734d85992290066fd31a7f5bee3cca3d3545a1a'
files=[]
for tree in ['fixtures','icons','licenses','tests']:
 for p in sorted((root/tree).rglob('*')):
  if not p.is_file() or 'node_modules' in p.parts:continue
  rel=p.relative_to(root).as_posix()
  if tree=='tests' and p.suffix!='.json':continue
  if rel.startswith('fixtures/own/'):
   origin='https://github.com/almatkai/unicode-symbols/blob/'+own+'/'+rel[len('fixtures/own/'):]
  elif rel.startswith('fixtures/beta/'):
   origin='Tezbar git '+beta+':'+rel[len('fixtures/beta/'):].removesuffix('.fixture')
  elif rel.startswith('fixtures/upstream/'):
   origin='https://github.com/raycast/extensions/blob/'+up+'/extensions/unicode-symbols/'+rel[len('fixtures/upstream/'):]
  elif rel.startswith('fixtures/vendor/'):
   origin='Exact installed npm package contents; own package-lock.json or tools/verification.package-lock.json. See NOTICES.md.'
  elif rel.startswith('fixtures/derived/'):
   origin='Derived from original dataset block names by Node en-US localeCompare; checked against the executed source.'
  elif rel.startswith('icons/'):
   origin='Byte-exact original command-icon.png from Raycast '+up
  elif rel.startswith('tests/'):
   origin='Executed original own TS modules and locked Fuse 6.6.2 via tools/oracle.cjs; generated verification snapshot.'
  else:
   origin={'Unicode-3.0.txt':'https://www.unicode.org/license.txt (retrieved 2026-10-24)','WHATWG-IPR.html':'https://whatwg.org/ipr-policy (retrieved 2026-10-24)','WHATWG-LICENSE.txt':'https://raw.githubusercontent.com/whatwg/html/main/LICENSE (retrieved 2026-10-24)','raycast-MIT.txt':'https://raw.githubusercontent.com/raycast/extensions/'+up+'/LICENSE','own-MIT.txt':'Own LICENSE, readable newline-normalized derivative'}.get(p.name,'See NOTICES.md')
  data=p.read_bytes();files.append({'path':rel,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'origin':origin})
manifest={'source':{'repository':'https://github.com/almatkai/unicode-symbols','sha':own,'sourceLock':'fixtures/own/package-lock.json','resolved':{'fuse.js':'6.6.2','@raycast/api':'1.103.10','@raycast/utils':'2.2.2','react':'19.0.0','typescript':'5.9.3','unidata16':'1.1.0','tsx':'4.23.13'}},'upstream':{'repository':'https://github.com/raycast/extensions','sha':up,'path':'extensions/unicode-symbols','originalAuthor':'mmazzarolo','contributors':['FezVrasta','pernielsentikaer','j3lte','xilopaint','xmok','muhammadrizo']},'beta':{'sha':beta,'reference':'read-only host git show; no host edits'},'runtime':'Compiled-in Rust. No JS/Bun/network dependency at runtime. Node fixtures are verification only.','files':files}
(root/'PROVENANCE.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print('Recorded',len(files),'evidence files')
