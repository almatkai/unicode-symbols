// The complete frozen schema, validated by the existing offline Ajv distribution.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const Module = require('node:module');
const npmRoot = path.join(root,'v2/fixtures/vendor/npm');
const packages = fs.readdirSync(npmRoot);
const load = Module._load;
Module._load = function(request,parent,isMain) {
  if (packages.some(name=>request===name || request.startsWith(name+'/'))) return load(path.join(npmRoot,request),parent,isMain);
  return load(request,parent,isMain);
};
const Ajv = require('../v2/fixtures/vendor/npm/ajv/dist/2020').default;
const schema = JSON.parse(fs.readFileSync(path.join(root, 'schemas/extension-v3.schema.json')));
// The host validates repository/artifact URLs too; keep this offline checker strict.
const ajv = new Ajv({ strict: false, allErrors: true });
ajv.addFormat('uri', value => { try { return Boolean(new URL(value).protocol); } catch { return false; } });
const validate = ajv.compile(schema);
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'extension.json')));
assert.equal(validate(manifest), true, JSON.stringify(validate.errors));
const original = JSON.parse(fs.readFileSync(path.join(root, 'package.json')));
assert.deepEqual(manifest.replaces, ['raycast.' + original.name]);
assert.deepEqual(manifest.preferences, original.preferences);
assert.deepEqual(manifest.commands.map(({name,title,description,mode,platforms})=>({name,title,description,mode,platforms})), original.commands);
assert.deepEqual(manifest.platforms, original.platforms);
assert.equal(manifest.id, 'com.tezbar.native.' + original.name);
assert.equal(manifest.execution, 'process');
assert.deepEqual(manifest.capabilities, ['clipboard.write', 'clipboard.pasteText', 'shell.open']);
assert.deepEqual(fs.readFileSync(path.join(root,'resources/assets/command-icon.png')), fs.readFileSync(path.join(root,'assets/command-icon.png')));
for (const mutation of [m=>m.schemaVersion=2,m=>m.protocolVersion=2,m=>m.execution='embedded',m=>m.id='raycast.unicode-symbols',m=>m.commands=[],m=>m.capabilities=['invented']]) {
  const bad=structuredClone(manifest); mutation(bad); assert.equal(validate(bad),false,JSON.stringify(bad));
}
console.log('Schema 3, negative schema probes, canonical replacement, original command/preferences/platform/icon parity passed');
