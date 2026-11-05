#!/usr/bin/env python3
"""Real JSONL migration regressions; grants come only from manifest paths on disk."""
import argparse
import json
import os
from pathlib import Path
import queue
import subprocess
import tempfile
import threading
import unittest
import uuid

ROOTS = [
    'extensions/unicode-symbols',
    'extensions/raycast.unicode-symbols',
    'extensions/packages/raycast.unicode-symbols',
    'extension-registry/packages/raycast.unicode-symbols',
    'extensions/packages/unicode-symbols',
    'extension-registry/packages/unicode-symbols',
]
FILENAMES = ['.tezbar-local-storage.json', 'localStorage.json']
MARKER = '_processMigrationV2'


def grants(manifest, profile):
    # Mirrors the host: declared profile-relative paths, and existing files only.
    return {name: str(profile / relative)
            for name, relative in manifest['migrationPaths'].items()
            if (profile / relative).is_file()}


def legacy(favorites=None, recent=None):
    return {key: json.dumps(value) for key, value in
            [('favorites-v1', favorites), ('recently-used-v2', recent)]
            if value is not None}


class Process:
    def __init__(self, binary, manifest, profile):
        self.manifest, self.profile = manifest, profile
        env = {k: v for k, v in os.environ.items() if k in ('SystemRoot', 'WINDIR', 'TEMP', 'TMP')}
        env['PATH'] = os.path.join(env.get('SystemRoot', 'C:/Windows'), 'System32') if os.name == 'nt' else '/usr/bin:/bin'
        self.process = subprocess.Popen([str(binary)], cwd=binary.parent, env=env,
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.frames = queue.Queue()
        self.ident = 0
        self.attempted = False
        self.ready = False
        self.binary = binary
        def read():
            try:
                for line in self.process.stdout:
                    self.frames.put(json.loads(line))
                self.frames.put(EOFError('process exited'))
            except Exception as error:
                self.frames.put(error)
        threading.Thread(target=read, daemon=True).start()
        assert self.receive()['type'] == 'hello'

    def receive(self):
        frame = self.frames.get(timeout=15)
        if isinstance(frame, Exception):
            raise frame
        return frame

    def request(self, method, params, error=False):
        self.ident += 1
        ident = 'h:' + str(uuid.uuid4())
        self.process.stdin.write((json.dumps({'type': 'request', 'id': ident, 'method': method, 'params': params}) + '\n').encode())
        self.process.stdin.flush()
        while True:
            frame = self.receive()
            # Migration/open must never request a real or fake OS effect.
            assert frame['type'] != 'request', frame
            if frame['type'] == 'response' and frame['id'] == ident:
                assert ('error' in frame) == error, frame
                return frame.get('error') if error else frame['result']

    def initialize(self, error=False, selected=None):
        # The immutable SDK permits initialize once per process, even on failure.
        # Recovery/idempotence are tested across actual process restarts.
        if self.attempted:
            self.close()
            self.__init__(self.binary, self.manifest, self.profile)
        self.attempted = True
        result = self.request('initialize', {
            'protocol': 1, 'extensionId': self.manifest['id'],
            'platform': 'Windows' if os.name == 'nt' else 'macOS',
            'packageDir': str(self.profile / 'package'),
            'dataDir': str(self.profile / 'private'), 'tempDir': str(self.profile / 'temp'),
            'grantedCapabilities': self.manifest['capabilities'], 'reason': 'command',
            'migrationPaths': grants(self.manifest, self.profile) if selected is None else selected,
        }, error=error)
        self.ready = not error
        return result

    def open_dispose(self):
        view = self.request('session.open', {'sessionId': 'migration', 'commandName': 'index',
                                            'preferences': {'viewType': 'list'}, 'argumentValues': {}})
        assert view['root']['type'] == 'List'
        self.request('session.dispose', {'sessionId': 'migration'})

    def close(self):
        try:
            if self.ready:
                self.request('shutdown', {})
            else:
                self.process.stdin.close()
            assert self.process.wait(timeout=5) == 0
        finally:
            if self.process.poll() is None:
                self.process.kill()
                self.process.wait(timeout=5)
            self.process.stdin.close()
            self.process.stdout.close()


class MigrationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.profile = Path(self.temp.name)
        self.store = self.profile / 'private/unicode-symbols/storage.json'
        self.children = []

    def tearDown(self):
        try:
            for child in self.children:
                child.close()
        finally:
            self.temp.cleanup()

    def write(self, relative, value):
        path = self.profile / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value if isinstance(value, bytes) else json.dumps(value).encode())
        return path

    def seed(self, value):
        return self.write('private/unicode-symbols/storage.json', value)

    def read(self):
        return json.loads(self.store.read_bytes()) if self.store.exists() else {}

    def start(self):
        child = Process(BINARY, MANIFEST, self.profile)
        self.children.append(child)
        return child

    def test_manifest_covers_every_managed_file(self):
        expected = {f'{root}/{name}' for root in ROOTS for name in FILENAMES}
        self.assertEqual(set(MANIFEST['migrationPaths'].values()), expected | {'native-extensions-v2/unicode-symbols/storage.json'})
        self.assertLessEqual(len(MANIFEST['migrationPaths']), 32)

    def test_all_twelve_paths_and_v1_recovery(self):
        for root in ROOTS:
            for filename in FILENAMES:
                for v1 in [False, True]:
                    with self.subTest(root=root, filename=filename, v1=v1), tempfile.TemporaryDirectory(dir=self.profile) as temp:
                        profile = Path(temp)
                        source = profile / root / filename
                        source.parent.mkdir(parents=True)
                        # Numeric identity survives an unpaired UTF16 glyph in raw JS records.
                        value = legacy([{'c':38,'score':0.25,'isExtra':True}, {'c':65}, {'c':55296,'v':'\ud800'}], [{'c':65,'score':0.5}])
                        source.write_bytes(json.dumps(value).encode())
                        before = source.read_bytes()
                        store = profile / 'private/unicode-symbols/storage.json'
                        if v1:
                            store.parent.mkdir(parents=True)
                            store.write_text('{"_processMigrationV1":true,"unknown":{"keep":1}}')
                        child = Process(BINARY, MANIFEST, profile)
                        try:
                            self.assertEqual(len(grants(MANIFEST, profile)), 1)
                            child.initialize(); child.open_dispose()
                            result = json.loads(store.read_bytes())
                            self.assertEqual(result['favorites-v1'], [38,65,55296])
                            self.assertEqual(result['recently-used-v2'], [65])
                            self.assertEqual(result['favorites-meta-v1']['38'], {'score':0.25,'extra':True})
                            self.assertEqual(result['recently-used-meta-v2']['65']['score'], 0.5)
                            self.assertIs(result[MARKER], True)
                            if v1:
                                self.assertEqual(result['unknown'], {'keep':1})
                                self.assertIs(result['_processMigrationV1'], True)
                            private_bytes = store.read_bytes()
                            child.initialize(); child.open_dispose()
                            self.assertEqual(store.read_bytes(), private_bytes)
                            self.assertEqual(source.read_bytes(), before)
                        finally:
                            child.close()

    def test_absent_empty_and_disappeared_sources_remain_retryable(self):
        child = self.start()
        child.initialize(); child.open_dispose()
        self.assertFalse(self.store.exists())
        source = self.write(f'{ROOTS[0]}/{FILENAMES[0]}', {})
        child.initialize()
        self.assertFalse(self.store.exists())
        selected = grants(MANIFEST, self.profile)
        source.unlink()
        child.initialize(selected=selected)
        self.assertFalse(self.store.exists())
        self.write(f'{ROOTS[0]}/{FILENAMES[0]}', legacy([{'c':38}]))
        child.initialize()
        self.assertNotIn(MARKER, self.read())
        self.write(f'{ROOTS[1]}/{FILENAMES[1]}', legacy(None, [{'c':65}]))
        child.initialize(); child.open_dispose()
        self.assertEqual(self.read()['favorites-v1'], [38])
        self.assertEqual(self.read()['recently-used-v2'], [65])
        self.assertIs(self.read()[MARKER], True)

    def test_every_source_precedence_rank(self):
        paths = ['native-extensions-v2/unicode-symbols/storage.json'] + [f'{root}/{filename}' for filename in FILENAMES for root in ROOTS]
        sources = []
        for rank, path in enumerate(paths):
            code = 65 + rank
            value = {'favorites-v1':[code], 'recently-used-v2':[code]} if rank == 0 else legacy([{'c':code,'score':rank/16,'isExtra':True}], [{'c':code}])
            source = self.write(path, value)
            sources.append((source, source.read_bytes()))
        child = self.start()
        for rank, (source, before) in enumerate(sources):
            if self.store.exists():
                self.store.unlink()  # isolated fresh profile model per rank
            child.initialize(); child.open_dispose()
            self.assertEqual(self.read()['favorites-v1'], [65+rank])
            self.assertEqual(self.read()['recently-used-v2'], [65+rank])
            for other, original in sources[rank:]:
                self.assertEqual(other.read_bytes(), original)
            source.unlink()

    def test_private_lists_metadata_and_unknown_keys_are_owned(self):
        seed = {'_processMigrationV1':True, 'favorites-v1':[], 'favorites-meta-v1':{},
                'recently-used-meta-v2':{}, 'unknown':{'nested':[1,2]}}
        self.seed(seed)
        self.write(f'{ROOTS[0]}/{FILENAMES[0]}', legacy([{'c':38,'score':0.5}], [{'c':65,'score':0.75,'isExtra':True}]))
        child = self.start(); child.initialize(); child.open_dispose()
        result = self.read()
        for key, value in seed.items():
            self.assertEqual(result[key], value)
        self.assertEqual(result['recently-used-v2'], [65])
        self.assertEqual(result['recently-used-meta-v2'], {})
        before = self.store.read_bytes()
        self.write(f'{ROOTS[0]}/{FILENAMES[0]}', b'bad json')
        child.initialize(); child.open_dispose()
        self.assertEqual(self.store.read_bytes(), before)

    def test_complete_private_model_ignores_malformed_legacy_without_marker(self):
        seed = {'favorites-v1':[], 'recently-used-v2':[65],
                'recently-used-meta-v2':{'65':{'score':0.3,'extra':True}}, 'unknown':True}
        self.seed(seed); before = self.store.read_bytes()
        self.write('native-extensions-v2/unicode-symbols/storage.json', b'bad')
        child = self.start(); child.initialize(); child.open_dispose()
        self.assertEqual(self.store.read_bytes(), before)

    def test_unused_domains_and_lower_priority_corruption_are_irrelevant(self):
        self.seed({'favorites-v1':[], 'favorites-meta-v1':{}})
        self.write('native-extensions-v2/unicode-symbols/storage.json', {'favorites-v1':'invalid', 'favorites-meta-v1':False})
        self.write(f'{ROOTS[0]}/{FILENAMES[0]}', {'favorites-v1':'bad', 'recently-used-v2':'[{"c":65}]'})
        self.write(f'{ROOTS[1]}/{FILENAMES[0]}', b'bad')
        child = self.start(); child.initialize(); child.open_dispose()
        self.assertEqual(self.read()['favorites-v1'], [])
        self.assertEqual(self.read()['recently-used-v2'], [65])

    def test_invalid_data_never_partially_imports_or_marks_and_can_retry(self):
        source = self.write(f'{ROOTS[0]}/{FILENAMES[0]}', {})
        child = self.start()
        invalid = [b'bad', [], {'favorites-v1':[]}, legacy([{'c':99999999}]),
                   legacy([{'c':38},{'c':38}]), legacy([{'c':65+i} for i in range(51)]),
                   legacy(None, [{'c':65+i} for i in range(11)]), legacy([{'c':38,'score':'bad'}]),
                   legacy([{'c':38,'isExtra':1}]), legacy([{'c':38}], [{'c':99999999}])]
        for value in invalid:
            with self.subTest(value=value):
                self.seed({'_processMigrationV1':True, 'unknown':42})
                before = self.store.read_bytes()
                source.write_bytes(value if isinstance(value, bytes) else json.dumps(value).encode())
                source_before = source.read_bytes()
                child.initialize(error=True)
                self.assertEqual(self.store.read_bytes(), before)
                self.assertEqual(source.read_bytes(), source_before)
        source.write_text(json.dumps(legacy([{'c':38}], [])))
        child.initialize(); child.open_dispose()
        self.assertEqual(self.read()['favorites-v1'], [38])

    def test_later_needed_source_error_rolls_back_earlier_valid_source(self):
        self.seed({'_processMigrationV1':True, 'unknown':'keep'})
        before = self.store.read_bytes()
        first = self.write(f'{ROOTS[0]}/{FILENAMES[0]}', legacy([{'c':38}]))
        first_before = first.read_bytes()
        later = self.write(f'{ROOTS[1]}/{FILENAMES[0]}', b'broken')
        child = self.start(); child.initialize(error=True)
        self.assertEqual(self.store.read_bytes(), before)
        self.assertEqual(first.read_bytes(), first_before)
        later.write_text(json.dumps(legacy(None, [])))
        child.initialize(); child.open_dispose()
        self.assertEqual(self.read()['favorites-v1'], [38])
        self.assertEqual(self.read()['recently-used-v2'], [])

    def test_exact_domain_limits_and_all_six_unpaired_records(self):
        codes = [0xd800,0xdb7f,0xdb80,0xdbff,0xdc00,0xdfff]
        favorites = [{'c':c,'v':chr(c),'score':i/64,'isExtra':bool(i%2)}
                     for i,c in enumerate(codes + list(range(65,109)))]
        recent = list(reversed(favorites[:10]))
        source = self.write(f'{ROOTS[-1]}/{FILENAMES[-1]}', legacy(favorites, recent))
        original = source.read_bytes()
        # Even a misleading V2 marker is not permission to skip missing lists.
        self.seed({'_processMigrationV1':True, MARKER:True, 'unknown':False})
        child = self.start(); child.initialize(); child.open_dispose()
        result = self.read()
        self.assertEqual(result['favorites-v1'], [row['c'] for row in favorites])
        self.assertEqual(result['recently-used-v2'], [row['c'] for row in recent])
        for row in favorites:
            self.assertEqual(result['favorites-meta-v1'][str(row['c'])],
                             {'score':row['score'],'extra':row['isExtra']})
        self.assertIs(result['unknown'], False)
        self.assertEqual(source.read_bytes(), original)
        before = self.store.read_bytes()
        child.initialize(); child.open_dispose()
        self.assertEqual(self.store.read_bytes(), before)

    def test_failed_read_does_not_publish_partial_import(self):
        self.seed({'_processMigrationV1':True, 'unknown':42})
        before = self.store.read_bytes()
        source = self.write(f'{ROOTS[0]}/{FILENAMES[0]}', legacy([{'c':38}], []))
        selected = grants(MANIFEST, self.profile)
        source.unlink(); source.mkdir()  # read failure after the host selected a regular file
        self.start().initialize(error=True, selected=selected)
        self.assertEqual(self.store.read_bytes(), before)

    @unittest.skipIf(os.name == 'nt', 'POSIX permission fault injection')
    def test_failed_atomic_write_keeps_original_and_retries(self):
        self.seed({'_processMigrationV1':True, 'unknown':42})
        before = self.store.read_bytes()
        self.write(f'{ROOTS[0]}/{FILENAMES[0]}', legacy([{'c':38}], []))
        child = self.start()
        self.store.parent.chmod(0o500)
        try:
            child.initialize(error=True)
            self.assertEqual(self.store.read_bytes(), before)
        finally:
            self.store.parent.chmod(0o700)
        child.initialize(); child.open_dispose()
        self.assertEqual(self.read()['favorites-v1'], [38])
        self.assertIs(self.read()[MARKER], True)


def run_suite(binary, manifest):
    global BINARY, MANIFEST
    BINARY, MANIFEST = Path(binary).resolve(), manifest
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(MigrationTests))
    if not result.wasSuccessful():
        raise SystemExit(1)
    print('Migration process regressions passed: 12 manifest-derived files, V1 recovery, ownership, precedence, faults and repeated initialize; no OS effects')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('--manifest', type=Path, default=Path(__file__).resolve().parents[1] / 'extension.json')
    args = parser.parse_args()
    run_suite(args.binary, json.loads(args.manifest.read_text()))
