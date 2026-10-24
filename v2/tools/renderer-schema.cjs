const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),Module=require('node:module'),cp=require('node:child_process');
const root=path.resolve(__dirname,'..'),beta=path.join(root,'fixtures/beta/src/renderer/src/raycast-api');
const ts=require('../fixtures/vendor/typescript/lib/typescript.js');const orig=Module._load;
let tests=0;
const npmRoot=path.join(root,'fixtures/vendor/npm');const packages=fs.readdirSync(npmRoot);
function expect(value){return {toBe:v=>assert.equal(value,v),toEqual:v=>assert.deepEqual(value,v),toContain:v=>assert.ok(value.includes(v)),toBeNull:()=>assert.equal(value,null),toMatch:v=>assert.match(value,v),toMatchObject:v=>{for(const k of Object.keys(v))assert.deepEqual(value[k],v[k]);}};}
Module._load=function(request,parent,isMain){
 const name=packages.find(p=>request===p||request.startsWith(p+'/'));if(name)return orig(path.join(npmRoot,request),parent,isMain);
 if(request==='vitest')return {describe:(_,fn)=>fn(),it:(name,fn)=>{fn();tests++;},expect};
 if(request.endsWith('/ui/primitives')){const React=orig(path.join(npmRoot,'react'),parent,isMain);const C=({children})=>React.createElement('span',null,children);return {Hint:C,HintBar:C,Kbd:C,ViewHeader:C,cx:(...v)=>v.filter(Boolean).join(' ')};}
 if(request.endsWith('/ui/Markdown')){const React=orig(path.join(npmRoot,'react'),parent,isMain);return {Markdown:({text})=>React.createElement('pre',null,text)};}
 if(request.endsWith('/colorConverter'))return {buildColorFormatRows:()=>[],buildColorConversionResults:()=>[]};
 if(request==='./hooks'&&parent?.filename===path.join(beta,'index.tsx'))return {};
 if(request==='./action-runtime'&&parent?.filename===path.join(beta,'index.tsx'))return {};
 if(request==='./action-runtime-registry'&&parent?.filename===path.join(beta,'index.tsx'))return {};
 return orig(request,parent,isMain);
};
for(const ext of ['.tsx','.ts'])Module._extensions[ext]=(module,file)=>{
 let code=ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true},fileName:file}).outputText;
 const names={'list-runtime.tsx':['parseListRows','parseListAccessory'],'grid-runtime.tsx':['collectGridItems','parseGridAccessory'],'form-runtime.tsx':['collectFormItems'],'detail-runtime.tsx':['markdownFromNode','parseImageOnlyMarkdown']};
 for(const name of names[path.basename(file)]||[])code+=`\nmodule.exports.${name}=${name};`;
 module._compile(code,file);
};
const React=require(path.join(npmRoot,'react')),server=require(path.join(npmRoot,'react-dom/server'));
const list=require(path.join(beta,'list-runtime.tsx')),grid=require(path.join(beta,'grid-runtime.tsx')),form=require(path.join(beta,'form-runtime.tsx')),detail=require(path.join(beta,'detail-runtime.tsx'));
for(const f of ['grid-runtime.test.ts','form-runtime.test.ts','detail-runtime.test.ts','native-v2-visuals.test.ts'])require(path.join(beta,f));
const requests=[{preferences:{viewType:'list',gridItemSize:'small',primaryAction:'copy'},query:'arow',emitView:true},{preferences:{viewType:'grid',dataSet:'full',gridItemSize:'small'},query:'U+1F600',emitView:true}];
const run=cp.spawnSync(path.join(root,'target/debug/examples/oracle_bridge'),[],{input:requests.map(JSON.stringify).join('\n')+'\n',encoding:'utf8',maxBuffer:30e6});assert.equal(run.status,0,run.stderr);
const outputs=run.stdout.trim().split('\n').map(JSON.parse);let wire=0;
for(let i=0;i<outputs.length;i++){
 const v=outputs[i].view;const root=v.root;assert.equal(root.props.__hasServerSearch,true);assert.equal(root.props.filtering,false);const rows=i===0?list.parseListRows(root):grid.collectGridItems(root);assert.ok(rows.length);assert.ok(rows[0].actionIds.length>=10);assert.ok(rows.every(r=>r.actionIds.every(id=>v.actions.some(a=>a.id===id))));
 const accessory=i===0?list.parseListAccessory(root.props.searchBarAccessory):grid.parseGridAccessory(root.props.searchBarAccessory);assert.equal(accessory.options[0].value,'null');assert.equal(accessory.options.length,i===0?42:347);assert.deepEqual(accessory.options.slice(1).map(x=>x.title),JSON.parse(fs.readFileSync(path.join(path.resolve(__dirname,'..'),'fixtures/own/assets',i===0?'dataset.json':'full-dataset.json'))).blocks.map(b=>b.blockName).sort((a,b)=>a.localeCompare(b)));
 const options={root,title:v.title,actions:v.actions,onBack:()=>{},onRunPrimaryAction:()=>{},onOpenActions:()=>{},onSearchTextChanged:()=>{},onLoadMore:()=>{}};
 const markup=server.renderToStaticMarkup(React.createElement(i===0?list.ListRuntime:grid.GridRuntime,options));assert.ok(markup.includes(i===0?'Arrow':'Grinning face'));if(i===1){assert.ok(markup.includes('data:image/svg+xml;base64,'));assert.ok(markup.includes('repeat(8,'));}
 // The typo/hex query does not occur in row titles; these rows survive server filtering.
 assert.ok(markup.includes(rows[0].title.replaceAll('&','&amp;')));wire++;
}
const sample={type:'Form',props:{backActionId:'back'},children:[{type:'Form.TextField',props:{id:'label',title:'Title',value:'Arrow',actionId:'label'}},{type:'Form.TextField',props:{id:'trigger',title:'Trigger',value:';arrow',actionId:'trigger'}},{type:'Form.Description',props:{title:'Character',text:'→'}}]};
assert.deepEqual(form.collectFormItems(sample).filter(x=>x.type==='field').map(x=>[x.field.id,x.field.value,x.field.actionId]),[['label','Arrow','label'],['trigger',';arrow','trigger']]);assert.ok(server.renderToStaticMarkup(React.createElement(form.FormRuntime,{root:sample,title:'Save as Snippet',onBack:()=>{},onSubmitForm:()=>{},onChangeField:()=>{},onInvokeAction:()=>{},onOpenActions:()=>{}})).includes(';arrow'));wire++;
const detailRoot={type:'Detail',props:{markdown:'# Unicode'},metadata:{type:'Detail.Metadata',children:[{type:'Detail.Metadata.Label',props:{title:'Code',text:'U+0041'}}]}};
assert.equal(detail.markdownFromNode(detailRoot),'# Unicode');assert.ok(server.renderToStaticMarkup(React.createElement(detail.DetailRuntime,{root:detailRoot,title:'Unicode',onBack:()=>{},onRunPrimaryAction:()=>{},onOpenActions:()=>{}})).includes('U+0041'));wire++;
// Run the complete, unchanged Draft 2020-12 schema with Ajv, not a handcrafted validator.
const Ajv=require(path.join(npmRoot,'ajv/dist/2020')).default;const schema=JSON.parse(fs.readFileSync(path.join(root,'fixtures/beta/schemas/extension-v2.schema.json')));const validate=new Ajv({strict:false,allErrors:true}).compile(schema);const manifest=JSON.parse(fs.readFileSync(path.join(root,'extension-v2.json')));assert.equal(validate(manifest),true,JSON.stringify(validate.errors));let negatives=0;
const bad=m=>{assert.equal(validate(m),false,JSON.stringify(m));negatives++;};const change=(key,value)=>{const m=structuredClone(manifest);m[key]=value;bad(m);};
for(const key of schema.required){const m=structuredClone(manifest);delete m[key];bad(m);}
for(const [k,v] of [['schemaVersion',1],['id','raycast.unicode-symbols'],['slug','Bad_Slug'],['name',''],['version',''],['runtime','javascript'],['replaces',[]],['replaces',['x']],['replaces',['raycast.x','raycast.x']],['icon','/tmp/icon.png'],['icon','/extensions/icons/U.png'],['platforms',['macOS','macOS']],['platforms',['Android']],['preferences','bad'],['commands',[]],['verification',[]],['author','forbidden'],['artworkBackground','#123']])change(k,v);
for(const field of ['name','platforms','runtime']){const m=structuredClone(manifest);delete m.commands[0][field];bad(m);}
for(const [key,value] of [['name',''],['platforms',[]],['platforms',['Android']],['platforms',['macOS','macOS']],['runtime','node']]){const m=structuredClone(manifest);m.commands[0][key]=value;bad(m);}
for(const [key,value] of [['platform','Android'],['state','unknown']]){const m=structuredClone(manifest);m.verification[0][key]=value;bad(m);}
for(const key of ['platform','state']){const m=structuredClone(manifest);delete m.verification[0][key];bad(m);}
for(const invalid of [null,[],"manifest",42])bad(invalid);
for(const [k,property] of Object.entries(schema.properties)){if(property.type)change(k,property.type==='string'?42:property.type==='array'?{}:property.type==='object'?[]:'wrong');}
for(const [key,value] of [['title',42],['gap',false]]){const m=structuredClone(manifest);m.commands[0][key]=value;bad(m);}
for(const [key,value] of [['note',false],['verifiedAt',42],['verifiedCommit',[]]]){const m=structuredClone(manifest);m.verification[0][key]=value;bad(m);}
for(const [key,value]of [['preferences',["wrong"]],['artwork','/tmp/image.png'],['verification',[manifest.verification[0],manifest.verification[0]]]])change(key,value);
// Stronger package invariants beyond the schema's intentionally permissive commands.
assert.equal('author' in manifest,false);assert.deepEqual(manifest.commands.map(c=>[c.name,c.title,c.description]),[['index','Search Unicode Symbol','Browse and copy-paste common Unicode symbols.']]);assert.deepEqual(manifest.verification.map(v=>[v.platform,v.state]),[['macOS','not-run'],['Windows','not-run']]);
console.log(`Unchanged beta renderer tests: ${tests}; List/Grid/Detail/Form wire+SSR contracts: ${wire}; schema valid + ${negatives} negative cases passed. UI primitives/Markdown/color helpers are headless stubs; no desktop QA.`);
