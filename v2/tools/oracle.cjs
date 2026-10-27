// Execute byte-exact original TS modules through the pinned TypeScript compiler.
const fs=require('node:fs'),path=require('node:path'),Module=require('node:module'),assert=require('node:assert/strict'),cp=require('node:child_process');
const root=path.resolve(__dirname,'..'),own=path.join(root,'fixtures/own');
const ts=require('../fixtures/vendor/typescript/lib/typescript.js');
let preferences={},context={},effects=[];
const jsx=(type,props)=>({type,props:props||{}});
const react={useMemo:fn=>fn(),useCallback:fn=>fn};
const api={environment:{assetsPath:path.join(own,'assets')},getPreferenceValues:()=>preferences,Action:Object.assign('Action',{CopyToClipboard:'copy',Paste:'paste',CreateSnippet:'snippet',OpenInBrowser:'open'}),ActionPanel:Object.assign('ActionPanel',{Section:'Section'}),Icon:new Proxy({},{get:(_,p)=>String(p)}),Keyboard:{Shortcut:{Common:{Copy:{key:'c'},Open:{key:'o'}}}},getFrontmostApplication:async()=>({name:'Active App'})};
const originalLoad=Module._load;
Module._load=function(request,parent,isMain){
 if(request==='@raycast/api')return api;
 if(request==='@raycast/utils')return {usePromise:()=>({data:undefined})};
 if(request==='react')return react;
 if(request==='react/jsx-runtime')return {jsx,jsxs:jsx,Fragment:'Fragment'};
 if(request==='fuse.js')return originalLoad(path.join(root,'fixtures/vendor/fuse.js/dist/fuse.common.js'),parent,isMain);
 if(request==='@/context/ListContext')return {useListContext:()=>context};
 if(request.startsWith('@/'))request=path.join(own,'src',request.slice(2));
 return originalLoad(request,parent,isMain);
};
for(const ext of ['.ts','.tsx'])Module._extensions[ext]=(module,file)=>module._compile(ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true,target:ts.ScriptTarget.ES2022},fileName:file}).outputText,file);
const defaults=Object.fromEntries(JSON.parse(fs.readFileSync(path.join(own,'package.json'))).preferences.map(p=>[p.name,p.default]));
function load(p){preferences={...defaults,...p};for(const file of Object.keys(require.cache)){if(file.startsWith(path.join(own,'src')))delete require.cache[file];}return {manager:require(path.join(own,'src/lib/dataset-manager.ts')),list:require(path.join(own,'src/lib/list.ts')),prefs:require(path.join(own,'src/lib/preferences.ts')),format:require(path.join(own,'src/lib/character-formatting.ts'))};}
function query(request){const source=load(request.preferences||{}),p=source.prefs;const data=source.manager.getFilteredDataset(request.query??'',request.filter??null);const original=JSON.parse(fs.readFileSync(path.join(own,'assets/full-dataset.json'))).characters;const rec=codes=>(codes||[]).map(c=>original.find(x=>x.c===c));const sections=source.list.buildList(data,rec(request.recent),!request.query,request.filter??null,rec(request.favorites)).map(s=>({title:s.sectionTitle,lowest:s.lowestScore,items:s.items.map(c=>[c.c,c.score??null,c.isExtra??false])}));return {request,preferences:{limit:p.searchResultLimit,view:p.viewType,columns:p.gridColumnNumber(),full:p.dataSetName==='full-dataset',primary:p.primaryAction},characters:data.characters,sections};}
const queries=['','arrow','arow','latin capital','cmd','U+1F600','😀','0x0041','65abc','\\u0041','ffffffff','  superscript  9  ','a\tb','"arrow"','^latin','!arrow','x|y',"'",'mathematical alphanumeric symbols extraordinarily long pattern'];
const requests=[];
for(const viewType of ['list','grid'])for(const dataSet of ['minimal','full'])for(const gridItemSize of ['small','medium','large'])for(const primaryAction of ['copy','paste']){
 for(const query of queries)requests.push({preferences:{viewType,dataSet,gridItemSize,primaryAction,searchResultLimitStr:'12'},query});
}
for(const limit of ['0','-1','-0.5','1.9','1001','garbage','0x2','Infinity','-Infinity'])for(const query of ['','arrow'])requests.push({preferences:{viewType:'grid',dataSet:'full',searchResultLimitStr:limit},query,filter:'Arrows'});
for(const query of ['U+0041','²','superscript 9','GRINING','\t','Á','İ','日本','😀','0000','123e2','\n','a | b','"a b"','D800','U+D800','surrogate','e\u0301','𝒜','👩‍💻','\uFEFFcmd\uFEFF','\u00a0','\u0085cmd'])requests.push({preferences:{viewType:'grid',dataSet:'full',searchResultLimitStr:'25'},query,favorites:[65,128512],recent:[178,65]});
requests.push({preferences:{viewType:'list',searchResultLimitStr:'12'},query:'178',filter:'Superscripts and Subscripts'});
requests.push({preferences:{viewType:'list',searchResultLimitStr:'12'},query:'65',filter:'Arrows'});
// Deduplicate search calculations across appearance/primary combinations; still assert every preference combination.
const cache=new Map();const cases=requests.map(r=>{const k=JSON.stringify([r.preferences.viewType==='grid'&&r.preferences.dataSet==='full',r.preferences.searchResultLimitStr,r.query,r.filter,r.favorites,r.recent]);let result=cache.get(k);if(!result){result=query(r);cache.set(k,result);}const p=load(r.preferences).prefs;return {...result,request:r,preferences:{limit:p.searchResultLimit,view:p.viewType,columns:p.gridColumnNumber(),full:p.dataSetName==='full-dataset',primary:p.primaryAction}};});
// Execute actual JSX action panel, formatting hooks, and callback effects.
load({primaryAction:'copy'});
const all=JSON.parse(fs.readFileSync(path.join(own,'assets/full-dataset.json'))).characters;
const html=new Map(JSON.parse(fs.readFileSync(path.join(own,'assets/html.json'))).html_entities.map(e=>[e.code,e.value]));
context={filter:null,findHtmlEntity:c=>html.get(c)||null,isRecentlyUsed:()=>true,isFavorite:()=>false};
for(const fn of ['addToRecentlyUsedItems','clearRecentlyUsedItems','removeFromRecentlyUsedItems','addToFavorites','removeFromFavorites','clearFavorites','setDatasetFilterAnd'])context[fn]=arg=>effects.push([fn,arg?.c??arg??null]);
const formatPath=path.join(own,'src/lib/character-formatting.ts');
const savedEnv=process.env.RAYCASTX;const formatters={};
for(const mode of ['0','1']){process.env.RAYCASTX=mode;delete require.cache[require.resolve(formatPath)];formatters[mode]=require(formatPath);}
if(savedEnv===undefined)delete process.env.RAYCASTX;else process.env.RAYCASTX=savedEnv;
delete require.cache[require.resolve(formatPath)];const originalFormatting=require(formatPath);
const panel=require(path.join(own,'src/components/CharacterActionPanel.tsx')).CharacterActionPanel;
function flatten(n,out=[]){if(!n)return out;if(Array.isArray(n)){for(const x of n)flatten(x,out);return out;}if(typeof n==='object'){out.push(n);flatten(n.props?.children,out);}return out;}
const actionFixtures=[];
for(const code of [9,38,65,178,8984,128512,119808]){const item=all.find(c=>c.c===code);const nodes=flatten(panel({item,section:'Arrows'}));const actions=nodes.filter(n=>['copy','paste','open','snippet','Action'].includes(n.type));const copies=actions.filter(n=>n.type==='copy');for(const n of actions){if(n.props.onCopy)n.props.onCopy();if(n.props.onPaste)n.props.onPaste();if(n.props.onAction)n.props.onAction();}assert.equal(actions[0].type,'copy');assert.equal(actions[0].props.content,item.v);assert.equal(actions[1].type,'paste');actionFixtures.push({code,formatting:Object.fromEntries(Object.entries(formatters).map(([mode,f])=>[mode,{light:f.encodeSVG(item.v,false),dark:f.encodeSVG(item.v,true),hex:f.numberToHex(code)}])),actions:actions.map(n=>({type:n.type,...Object.fromEntries(Object.entries(n.props).filter(([k,v])=>typeof v!=='function'&&k!=='children'))}))});}
// Execute original useItemList and persistence hook with a deterministic React hook runner.
let slots=[],cursor=0,pending=[],stored={};
react.useState=initial=>{const i=cursor++;if(!(i in slots))slots[i]=initial;return [slots[i],v=>{slots[i]=typeof v==='function'?v(slots[i]):v;}];};
react.useEffect=fn=>{const i=cursor++;if(!(i in slots)){slots[i]=true;pending.push(fn());}};
api.LocalStorage={getItem:async k=>stored[k],setItem:(k,v)=>{stored[k]=v;}};
const useItemList=require(path.join(own,'src/hooks/use-item-list.ts')).useItemList;
(async()=>{
 const run=()=>{cursor=0;return useItemList({key:'favorites-v1',comparator:'c',limit:2});};
 run();await new Promise(r=>setImmediate(r));let hook=run();assert.equal(hook.areItemsLoaded,true);
 for(const c of [65,66,65,67]){hook.addItem(all.find(x=>x.c===c));hook=run();}
 assert.deepEqual(hook.items.map(c=>c.c),[67,66]);hook.removeItem(all.find(c=>c.c===66));hook=run();assert.deepEqual(hook.items.map(c=>c.c),[67]);hook.clearItems();assert.deepEqual(JSON.parse(stored['favorites-v1']),[]);
 fs.writeFileSync(path.join(root,'tests/oracle-cases.json'),JSON.stringify(cases));fs.writeFileSync(path.join(root,'tests/oracle-ranks.json'),JSON.stringify(cases.map(c=>({...c,characters:c.characters.map(x=>[x.c,x.score??null])}))));fs.writeFileSync(path.join(root,'tests/original-actions.json'),JSON.stringify({actionFixtures,effects},null,2)+'\n');
 console.log(`Original TS/Fuse 6.6.2 oracle executed: ${cases.length} cases, ${cache.size} distinct searches; 7 action panels and source persistence hook passed`);
 if(process.argv.includes('--compare')){
  const bin=path.join(root,'target/debug/examples/oracle_bridge');const child=cp.spawnSync(bin,[],{input:requests.map(r=>JSON.stringify(r)).join('\n')+'\n',encoding:'utf8',maxBuffer:150*1024*1024});assert.equal(child.status,0,child.stderr);const actual=child.stdout.trim().split('\n').map(JSON.parse);assert.equal(actual.length,cases.length);
  for(let i=0;i<cases.length;i++){const a=actual[i],e=cases[i];try {assert.deepEqual(a.preferences,JSON.parse(JSON.stringify(e.preferences)));assert.deepEqual(a.characters.map(c=>{const {score,...v}=c;return v}),e.characters.map(c=>{const {score,...v}=c;return v}));assert.deepEqual(a.sections.map(s=>[s.title,s.items.map(x=>[x[0],x[2]])]),e.sections.map(s=>[s.title,s.items.map(x=>[x[0],x[2]])]));for(let j=0;j<a.characters.length;j++){const x=a.characters[j].score,y=e.characters[j].score;if(y!==undefined)assert.ok(Math.abs(x-y)<1e-11,`score ${x} != ${y}`);}}catch(err){throw new Error(`case ${i}: ${JSON.stringify(e.request)}\n${err.stack}`);}}
  for(const full of [false,true]){const request={dumpAll:true,preferences:{viewType:'grid',dataSet:full?'full':'minimal'}};const dump=cp.spawnSync(bin,[],{input:JSON.stringify(request)+'\n',encoding:'utf8',maxBuffer:15e6});assert.equal(dump.status,0,dump.stderr);assert.deepEqual(JSON.parse(dump.stdout).characters,JSON.parse(fs.readFileSync(path.join(own,'assets',full?'full-dataset.json':'dataset.json'))).characters);}
  console.log(`Both full original datasets matched (46,321 records). Rust matched ${cases.length} original cases: exact records, order, sections, preferences, scores`);
 }
})().catch(e=>{console.error(e);process.exitCode=1;});
