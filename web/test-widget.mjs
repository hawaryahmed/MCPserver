import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { spawn } from 'node:child_process';
import assert from 'node:assert/strict';

// Exercise the real bundled app through an MCP Apps host bridge, backed by the real Rust server.
const root=resolve('..');
const temp=await mkdtemp(join(tmpdir(),'sme-widget-'));
const free= createServer();await new Promise(r=>free.listen(0,'127.0.0.1',r));const rustPort=free.address().port;await new Promise(r=>free.close(r));
const child=spawn(resolve(root,process.env.MCP_TEST_BINARY||'server/target/debug/enisa-sme-mcp'),[],{env:{...process.env,HOST:'127.0.0.1',PORT:String(rustPort),ENISA_MCP_DATA_DIR:temp},stdio:['ignore','ignore','pipe']});
let logs='';child.stderr.on('data',b=>logs+=b);
const widgetCalls=[];
const rpc=async(method,params={})=>{
 const res=await fetch(`http://127.0.0.1:${rustPort}/mcp`,{method:'POST',headers:{'content-type':'application/json',accept:'application/json, text/event-stream'},body:JSON.stringify({jsonrpc:'2.0',id:1,method,params})});
 assert.equal(res.status,200);return (await res.json()).result;
};
let host,view,browser;
try{
 for(let i=0;i<100;i++){try{await rpc('ping');break}catch(e){if(i===99)throw new Error(logs);await new Promise(r=>setTimeout(r,100))}}
 const html=(await rpc('resources/read',{uri:'ui://sme-maturity/assessment-v2.html'})).contents[0].text;
 const overview=await rpc('tools/call',{name:'assessment_overview',arguments:{}});
 view=createServer((req,res)=>{res.setHeader('content-type','text/html');res.end(html)});
 await new Promise(r=>view.listen(0,'127.0.0.1',r));
 const viewUrl=`http://127.0.0.1:${view.address().port}`;
 host=createServer(async(req,res)=>{
  if(req.url==='/rpc'){
   let raw='';for await(const b of req)raw+=b;
   const m=JSON.parse(raw);widgetCalls.push(m.params?.name);const result=await rpc(m.method,m.params);
   res.setHeader('content-type','application/json');res.end(JSON.stringify(result));return;
  }
  res.setHeader('content-type','text/html');
  res.end(`<!doctype html><iframe id="app" sandbox="allow-scripts allow-same-origin" src="${viewUrl}" style="border:0;width:100%;height:1400px"></iframe><script>
   const frame=document.getElementById('app');window.appEvents=[];
   addEventListener('message',async e=>{
    if(e.source!==frame.contentWindow)return;
    const m=e.data;if(m?.jsonrpc!=='2.0')return;window.appEvents.push(m);
    const reply=result=>frame.contentWindow.postMessage({jsonrpc:'2.0',id:m.id,result},'${viewUrl}');
    if(m.method==='ui/initialize')reply({protocolVersion:m.params.protocolVersion,hostInfo:{name:'test-host',version:'1'},hostCapabilities:{serverTools:{},updateModelContext:{}},hostContext:{theme:'dark',displayMode:'inline',availableDisplayModes:['inline','fullscreen']}});
    else if(m.method==='ui/notifications/initialized'){
      frame.contentWindow.postMessage({jsonrpc:'2.0',method:'ui/notifications/tool-input',params:{arguments:{}}},'${viewUrl}');
      frame.contentWindow.postMessage({jsonrpc:'2.0',method:'ui/notifications/tool-result',params:${JSON.stringify(overview).replace(/</g,'\\u003c')}},'${viewUrl}');
    } else if(m.method==='ui/notifications/size-changed'){if(m.params.height)frame.style.height=m.params.height+'px'}
    else if(m.method==='tools/call'){const r=await fetch('/rpc',{method:'POST',body:JSON.stringify(m)});reply(await r.json())}
    else if(m.id!=null)reply(m.method==='ui/request-display-mode'?{mode:m.params.mode}:{});
   });</script>`);
 });
 await new Promise(r=>host.listen(0,'127.0.0.1',r));
 browser=await chromium.launch({headless:true,args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:430,height:1000}});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(`http://127.0.0.1:${host.address().port}`);
 const f=page.frameLocator('#app');
 await f.getByRole('button',{name:'Start assessment →'}).waitFor();
 await f.getByLabel('Organization or demo name').fill('Widget test <safe>');
 await f.getByRole('button',{name:'Start assessment →'}).click();
 await f.getByText('Question 1 of 12',{exact:false}).waitFor();
 await f.getByText('Explain this question',{exact:true}).click();
 assert(await f.getByText('Name a person who owns security decisions and can approve time or budget.').isVisible());
 await f.getByText('Show an answer example',{exact:true}).click();
 await f.getByLabel('Your answer / current practice').fill('Our founder owns security decisions.');
 if(process.env.WIDGET_QA_SCREENSHOT)await f.locator('#root').screenshot({path:process.env.WIDGET_QA_SCREENSHOT});
 await f.getByLabel('Implementation level').selectOption('2');
 assert((await f.locator('#follow-prompt').textContent()).includes('What is missing'));
 await f.getByLabel('Implementation level').selectOption('U');
 assert((await f.locator('#follow-prompt').textContent()).includes('Who can check'));
 await f.getByLabel('Implementation level').selectOption('NA');
 await f.getByRole('button',{name:'Save & next →'}).click();
 assert(await f.getByText('Question 1 of 12',{exact:false}).isVisible());
 await f.getByLabel('Why is this not applicable?').fill('Test reason');
 await f.getByLabel('Implementation level').selectOption('3');
 await f.getByText('Further question / clarification',{exact:true}).click();
 await f.getByLabel('Follow-up answer').fill('The operations lead is the backup owner.');
 await f.getByRole('button',{name:'Save & next →'}).click();
 await f.getByText('Question 2 of 12',{exact:false}).waitFor();
 await f.getByRole('button',{name:'← Back'}).click();
 assert.equal(await f.getByLabel('Your answer / current practice').inputValue(),'Our founder owns security decisions.');
 assert.equal(await f.getByLabel('Follow-up answer').inputValue(),'The operations lead is the backup owner.');
 await f.getByRole('button',{name:'Save & next →'}).click();
 for(let n=2;n<=12;n++){
  await f.getByText(`Question ${n} of 12`,{exact:false}).waitFor();
  await f.getByLabel('Implementation level').selectOption('3');
  await f.getByRole('button',{name:n===12?'Save & review':'Save & next →'}).click();
 }
 await f.getByText('75%',{exact:true}).first().waitFor();
 const reportText=await f.locator('#root').innerText();assert(reportText.includes('30 / 60 / 90-day'));
 assert.equal(widgetCalls.filter(n=>n==='get_assessment').length,0,'Create/save should not re-fetch full questionnaires');
 assert.equal(widgetCalls.filter(n=>n==='record_answer').length,13);
 await page.waitForFunction(()=>window.appEvents.some(m=>m.method==='ui/update-model-context'&&JSON.stringify(m.params).includes('completed')));
 await f.getByRole('button',{name:'Summarize in chat'}).click();
 await page.waitForFunction(()=>window.appEvents.some(m=>m.method==='ui/message'));
 const message=await page.evaluate(()=>window.appEvents.find(m=>m.method==='ui/message').params);
 assert(JSON.stringify(message).includes('75'));
 await f.getByRole('button',{name:'Export backup',exact:true}).click();
 const backup=JSON.parse(await f.getByLabel('Backup contents').inputValue());
 assert.equal(backup.assessment.bank_version,'1.0.0');assert.equal(backup.assessment.revision,13);
 await f.getByRole('button',{name:'Download JSON'}).click();
 await page.waitForFunction(()=>window.appEvents.some(m=>m.method==='ui/download-file'));

 await f.getByRole('button',{name:'Return to questions'}).click();
 const text=await f.locator('#root').innerText();const id=text.match(/[0-9a-f]{8}-[0-9a-f-]{27,}/)[0];
 await f.getByRole('button',{name:'Overview',exact:true}).click();
 await f.getByLabel('Assessment ID',{exact:true}).fill(id);
 await f.getByRole('button',{name:'Resume',exact:true}).click();
 await f.getByRole('button',{name:'Return to questions'}).click();
 await f.getByText('Question 1 of 12',{exact:false}).waitFor();
 assert.equal(await f.getByLabel('Your answer / current practice').inputValue(),'Our founder owns security decisions.');

 await f.getByRole('button',{name:'Overview',exact:true}).click();
 await f.getByLabel('Backup JSON',{exact:true}).fill(JSON.stringify(backup));
 await f.getByRole('button',{name:'Restore backup',exact:true}).click();
 await f.getByRole('button',{name:'Return to questions'}).click();
 const restoredText=await f.locator('#root').innerText();const restoredId=restoredText.match(/[0-9a-f]{8}-[0-9a-f-]{27,}/)[0];
 assert.notEqual(restoredId,id);
 // A second editor changes the same record. The widget must reject a stale save and preserve its draft.
 await rpc('tools/call',{name:'record_answer',arguments:{assessment_id:restoredId,question_id:'P01',value:'2',expected_revision:0,answer_text:'Concurrent editor'}});
 await f.getByLabel('Implementation level').selectOption('1');
 await f.getByLabel('Your answer / current practice').fill('My retained draft');
 await f.getByRole('button',{name:'Save & next →'}).click();
 await f.getByText('Another form changed this assessment',{exact:false}).waitFor();
 assert.equal(await f.getByLabel('Your answer / current practice').inputValue(),'My retained draft');
 await f.getByRole('button',{name:'Reload saved version (keep draft)'}).click();
 await f.getByRole('button',{name:'Save & next →'}).click();
 await f.getByText('Question 2 of 12',{exact:false}).waitFor();
 await f.getByRole('button',{name:'Review saved results',exact:true}).click();
 await f.getByText('Assign a security decision owner and deputy; document their authority and escalation route.',{exact:true}).waitFor();
 assert((await f.locator('#root').innerText()).includes('Effort: S'));
 const loaded=(await rpc('tools/call',{name:'get_assessment',arguments:{assessment_id:restoredId}})).structuredContent;
 assert.equal(loaded.answers.P01.answer_text,'My retained draft');
 assert.equal(errors.length,0,errors.join('\n'));
 const overflow=await f.locator('body').evaluate(x=>x.scrollWidth>innerWidth);assert.equal(overflow,false,'Mobile layout overflow');
 console.log('PASS: MCP resource + SDK bridge, overview, create, one-question form, explanations/examples, NA validation, follow-up save, back/edit, completion score, resume, single-call saves, completion chat handoff, backup download/restore, stale-edit recovery, mobile layout, no script errors.');
} finally {
 await browser?.close();await new Promise(r=>host?host.close(r):r());await new Promise(r=>view?view.close(r):r());
 child.kill('SIGTERM');await new Promise(r=>child.exitCode!==null?r():child.once('exit',r));await rm(temp,{recursive:true,force:true});
}
