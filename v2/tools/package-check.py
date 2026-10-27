#!/usr/bin/env python3
"""Create and validate the distributable using only its own extracted files."""
from pathlib import Path
import json,os,subprocess,tarfile
root=Path(__file__).resolve().parent.parent
run=lambda argv,cwd=root:subprocess.run(argv,cwd=cwd,check=True)
run(['cargo','package','--allow-dirty','--locked','--offline'])
archive=root/'target/package/tezbar-unicode-symbols-0.1.0.crate';out=root/'.work/extracted';out.mkdir(parents=True,exist_ok=True)
with tarfile.open(archive) as tar:
 for m in tar.getmembers():
  path=(out/m.name).resolve();assert path.is_relative_to(out.resolve()),m.name
  assert not m.issym() and not m.islnk(),m.name
  assert 'node_modules' not in path.parts and '.work' not in Path(m.name).parts,m.name
 tar.extractall(out)
pkg=out/'tezbar-unicode-symbols-0.1.0'
provenance=json.loads((pkg/'PROVENANCE.json').read_text())
for f in provenance['files']:assert (pkg/f['path']).is_file(),f['path']
run(['python3','tools/verify-provenance.py'],pkg)
run(['cargo','test','--locked','--offline'],pkg)
run(['cargo','+1.85.0','test','--locked','--offline'],pkg)
run(['cargo','build','--example','oracle_bridge','--locked','--offline'],pkg)
run(['node','tools/oracle.cjs','--compare'],pkg)
run(['node','tools/renderer-schema.cjs'],pkg)
run(['python3','tools/materialize-beta.py'],pkg)
run(['cargo','test','--manifest-path','.work/beta-host/Cargo.toml','--locked','--offline'],pkg)
run(['cargo','clippy','--manifest-path','.work/beta-host/Cargo.toml','--all-targets','--locked','--offline','--','-D','warnings'],pkg)
print('Extracted package passed offline without parent checkout, host profile, node_modules, or external fixtures:',archive)
