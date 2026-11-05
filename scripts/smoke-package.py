#!/usr/bin/env python3
"""Verify an extracted package through real JSONL; all desktop effects are recorded fakes."""
import argparse, hashlib, json, os, queue, stat, subprocess, tempfile, threading, uuid, zipfile
from pathlib import Path, PurePosixPath

def smoke(path):
    with tempfile.TemporaryDirectory() as temp, zipfile.ZipFile(path) as archive:
        root=Path(temp)/'package'; root.mkdir()
        names=archive.namelist()
        assert len(names)==len(set(names))==len(set(n.casefold() for n in names)), 'Duplicate ZIP paths'
        for item in archive.infolist():
            name=PurePosixPath(item.filename)
            assert not name.is_absolute() and '..' not in name.parts and '\\' not in item.filename and ':' not in item.filename, 'Unsafe ZIP path'
            assert stat.S_ISREG(item.external_attr>>16), 'Nonregular ZIP member'
        manifest=json.loads(archive.read('extension.json')); inventory=manifest['files']
        assert set(names)==set(inventory)|{'extension.json'}, 'ZIP/inventory mismatch'
        for name,digest in inventory.items():
            assert hashlib.sha256(archive.read(name)).hexdigest()==digest, 'SHA inventory mismatch: '+name
        archive.extractall(root)
        slug=manifest['slug']; assert slug=='unicode-symbols'
        binary=root/'bin'/('tezbar-'+slug+('.exe' if os.name=='nt' else '')); binary.chmod(0o755)
        env={k:v for k,v in os.environ.items() if k in ('SystemRoot','WINDIR','TEMP','TMP')}
        env['PATH']=os.path.join(env.get('SystemRoot','C:/Windows'),'System32') if os.name=='nt' else '/usr/bin:/bin'
        info=json.loads(subprocess.check_output([str(binary),'--protocol-info'],cwd=root,env=env,timeout=10))
        assert info['id']==manifest['id'] and info['commands']==['index'] and info['protocol']==1 and info['version']==manifest['version']
        process=subprocess.Popen([str(binary)],cwd=root,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=None)
        messages=queue.Queue(); calls=[]; sizes=[]
        def reader():
            try:
                while True:
                    line=process.stdout.readline(16*1024*1024+1)
                    if not line: messages.put(EOFError('Unexpected EOF')); return
                    assert len(line)<=16*1024*1024, 'Oversized protocol frame'
                    sizes.append(len(line)); messages.put(json.loads(line))
            except Exception as error: messages.put(error)
        threading.Thread(target=reader,daemon=True).start()
        def send(frame):
            process.stdin.write((json.dumps(frame)+'\n').encode()); process.stdin.flush()
        def receive():
            frame=messages.get(timeout=15)
            if isinstance(frame,Exception): raise frame
            return frame
        def start(method,params):
            ident='h:'+str(uuid.uuid4()); send({'type':'request','id':ident,'method':method,'params':params}); return ident
        def finish(ident,expected_error=None,host_error=None):
            while True:
                frame=receive()
                if frame['type']=='response' and frame['id']==ident:
                    if expected_error:
                        assert expected_error.lower() in str(frame.get('error','')).lower(), frame
                        return frame
                    assert 'error' not in frame, frame
                    return frame['result']
                if frame['type']=='request':
                    method=frame['method']; calls.append(frame)
                    assert method in manifest['capabilities'] or method.removeprefix('host.') in manifest['capabilities'], frame
                    assert method in ('host.clipboard.write','host.clipboard.pasteText','host.shell.open'), frame
                    reply={'type':'response','id':frame['id']}
                    if host_error: reply['error']={'code':'stub_denied','message':host_error}
                    elif method=='host.clipboard.pasteText': reply['result']={'ok':True,'message':'Pasted text'}
                    elif method=='host.shell.open': reply['result']={'ok':True,'message':'Opened'}
                    else: reply['result']={'ok':True}
                    send(reply)
                else: assert frame['type'] in ('event','cancel'),frame
        def request(method,params,**kw): return finish(start(method,params),**kw)
        def call(operation,payload=None,session='zip-smoke',**kw):
            return request('session.call',{'sessionId':session,'operation':'extension:'+operation,'payload':payload or {}},**kw)
        def rows(view):
            assert view['mode']=='view' and view['root']['type'] in ('Grid','List'), view
            out=[row for section in view['root']['children'] for row in section['children']]
            ids={a['id'] for a in view['actions']}
            assert all(set(row['props']['actionIds'])<=ids for row in out)
            assert len(out)<=150
            return out
        def action(view,title,code=38):
            actions={a['id']:a for a in view['actions']}
            row=next(r for r in rows(view) if r['props']['id'].endswith(':'+str(code)))
            return next(ident for ident in row['props']['actionIds'] if actions[ident]['title']==title)
        def invoke(view,title,code=38,**kw): return call('invoke-action',{'actionId':action(view,title,code)},**kw)
        try:
            hello=receive(); assert all(hello.get(k)==v for k,v in {'type':'hello','id':manifest['id'],'version':manifest['version'],'protocol':1}.items()),hello
            legacy=Path(temp)/'legacy.json'
            legacy.write_text(json.dumps({'favorites-v1':json.dumps([{'c':38,'score':0.5,'isExtra':True}]),'recently-used-v2':json.dumps([{'c':65}])}))
            private=Path(temp)/'data'
            request('initialize',{'protocol':1,'extensionId':manifest['id'],'platform':'Windows' if os.name=='nt' else 'macOS','packageDir':str(root),'dataDir':str(private),'tempDir':str(Path(temp)/'temp'),'grantedCapabilities':manifest['capabilities'],'reason':'command','migrationPaths':{'legacyLocalStorage':str(legacy)}})
            view=request('session.open',{'sessionId':'zip-smoke','commandName':'index','preferences':{'primaryAction':'copy'},'argumentValues':{}}); rows(view)
            stored=private/'unicode-symbols/storage.json'
            assert json.loads(stored.read_text())['favorites-v1']==[38]
            assert not calls, 'Opening must not perform desktop effects'
            view=call('search-text-changed',{'searchText':'&'}); assert rows(view)
            good_id=action(view,'Copy Character to Clipboard')
            call('invoke-action',{'actionId':'forged'},expected_error='forged')
            call('invoke-action',{'actionId':good_id},host_error='Clipboard denied',expected_error='Clipboard denied')
            assert json.loads(stored.read_text())['recently-used-v2']==[65]
            view=call('invoke-action',{'actionId':good_id}); assert view['message']=='Copied to clipboard'
            assert calls[-1]['method']=='host.clipboard.write' and calls[-1]['params']=={'text':'&'}
            assert json.loads(stored.read_text())['recently-used-v2']==[38,65]
            call('invoke-action',{'actionId':good_id},expected_error='forged')
            for title,text in [('Copy "0026" (HEX) to Clipboard','0026'),('Copy "\\u0026" (Unicode Escape) to Clipboard','\\u0026'),('Copy "&amp;" (HTML Entity) to Clipboard','&amp;'),('Copy "&#38;" (HTML Decimal) to Clipboard','&#38;')]:
                view=invoke(view,title); assert calls[-1]['params']=={'text':text}
            view=invoke(view,'Paste Character to Active App'); assert calls[-1]['method']=='host.clipboard.pasteText' and calls[-1]['params']=={'text':'&'}
            for title,url in [('Open Character on Compart','https://www.compart.com/en/unicode/U+0026'),('Open Character on Unicode Explorer','https://unicode-explorer.com/c/0026')]:
                view=invoke(view,title); assert calls[-1]['method']=='host.shell.open' and calls[-1]['params']=={'target':url}
            count=len(calls); invoke(view,'Save as Snippet',expected_error='unsupported'); assert len(calls)==count
            view=invoke(view,'Remove from Favorites'); assert json.loads(stored.read_text())['favorites-v1']==[]
            view=invoke(view,'Add to Favorites'); assert json.loads(stored.read_text())['favorites-v1']==[38]
            view=invoke(view,'Clear All Favorites'); assert json.loads(stored.read_text())['favorites-v1']==[]
            view=invoke(view,'Remove from Recently Used'); assert json.loads(stored.read_text())['recently-used-v2']==[65]
            view=invoke(view,'Copy Character to Clipboard')
            view=invoke(view,'Clear All Recently Used'); assert json.loads(stored.read_text())['recently-used-v2']==[]
            view=invoke(view,'Set Filter to "Basic Latin"')
            assert view['root']['props']['searchBarAccessory']['props']['value']=='Basic Latin'
            view=invoke(view,'Clear Filter (Show All Characters)')
            assert view['root']['props']['searchBarAccessory']['props']['value']=='null'
            old=view; assert call('refresh-session')['mode']=='unchanged'
            view=invoke(old,'Copy Character to Clipboard') # refresh preserves bindings
            # Cancellation of a pending paste must fence state and prevent automatic replay.
            paste_id=action(view,'Paste Character to Active App')
            ident=start('session.call',{'sessionId':'zip-smoke','operation':'extension:invoke-action','payload':{'actionId':paste_id}})
            reverse=receive(); assert reverse['type']=='request' and reverse['method']=='host.clipboard.pasteText'; calls.append(reverse)
            send({'type':'cancel','requestId':ident,'reason':'smoke cancellation'})
            finish(ident,expected_error='cancel')
            count=len(calls);call('invoke-action',{'actionId':paste_id},expected_error='uncertain');assert len(calls)==count
            # Cross-session IDs and stale views cannot authorize effects.
            other=request('session.open',{'sessionId':'other','commandName':'index','preferences':{'viewType':'list'},'argumentValues':{}});rows(other)
            call('invoke-action',{'actionId':paste_id},session='other',expected_error='forged')
            request('session.dispose',{'sessionId':'other'})
            # Full grid + large block filter is paged, below wire cap; page actions are on rows.
            full=request('session.open',{'sessionId':'full','commandName':'index','preferences':{'dataSet':'full'},'argumentValues':{}}); rows(full)
            filter_id=next(a['id'] for a in full['actions'] if a['title']=='Select a Character Set')
            full=call('invoke-action',{'actionId':filter_id,'formValues':{'value':'Egyptian Hieroglyphs Extended-A'}},session='full'); first=rows(full); assert len(first)==150
            next_id=next(a['id'] for a in full['actions'] if a['title']=='Next Page'); assert next_id in first[0]['props']['actionIds']
            second=call('invoke-action',{'actionId':next_id},session='full'); assert not {r['props']['id'] for r in first}&{r['props']['id'] for r in rows(second)}
            prev_id=next(a['id'] for a in second['actions'] if a['title']=='Previous Page')
            back=call('invoke-action',{'actionId':prev_id},session='full'); assert [r['props']['id'] for r in rows(back)]==[r['props']['id'] for r in first]
            request('session.dispose',{'sessionId':'full'})
            request('session.dispose',{'sessionId':'zip-smoke'})
            call('refresh-session',expected_error='unknown session')
            request('shutdown',{}); assert process.wait(timeout=5)==0
        finally:
            if process.poll() is None: process.kill();process.wait(timeout=5)
            process.stdin.close();process.stdout.close()
        print(f'{slug}: ZIP inventory, handshake, migration, Grid/List, search, all supported actions, failures, stale/cross-session IDs, cancellation, large-block paging, refresh/dispose/shutdown passed; {len(calls)} fake host calls; largest frame {max(sizes)} bytes; no installation or desktop effects')
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('archive',type=Path);a=p.parse_args();smoke(a.archive)
