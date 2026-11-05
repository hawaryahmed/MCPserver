import { App } from '@modelcontextprotocol/ext-apps';
import './style.css';

const app = new App({name:'SME Cyber Maturity',version:'0.4.0'}, {availableDisplayModes:['inline','fullscreen']});
const root=document.getElementById('root');
const state={connected:false,busy:false,data:null,index:0,drafts:{},report:null,error:'',notice:'',org:'',mode:'quick'};
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const scales=[['0','Not in place'],['1','Ad hoc'],['2','Partly implemented'],['3','Consistently implemented'],['4','Measured, tested and improved'],['U','Unknown'],['NA','Not applicable']];
function el(id){return document.getElementById(id)}
function receive(result){
 const data=result?.structuredContent;
 if(!data)return;
 state.data=data;state.report=null;state.drafts={};state.error='';
 state.index=Number.isInteger(data.next_index)?data.next_index:0;
 render();
}
async function tool(name,args={}){
 if(!state.connected)throw new Error('Connect this assessment inside ChatGPT before saving.');
 const r=await app.callServerTool({name,arguments:args});
 if(r.isError)throw new Error(r.content?.find(x=>x.type==='text')?.text||'The server could not complete this action.');
 if(!r.structuredContent)throw new Error('The server returned no assessment data.');
 return r.structuredContent;
}
async function act(fn){
 if(state.busy)return;
 capture();state.busy=true;state.error='';render();
 try{await fn()}catch(e){state.error=e.message||'Please try again.'}
 finally{state.busy=false;render()}
}
function header(){return `<header><div><span class="eyebrow">SME CYBER MATURITY</span><h1>${state.data?.view==='questionnaire'?'Your assessment':'Assessment overview'}</h1></div><button id="expand" class="icon" title="Expand assessment" aria-label="Expand assessment">⤢</button></header>`}
function overview(){return `
 <section class="card hero"><span class="badge">People · Processes · Technology</span><h2>Know where you stand.<br>Choose what to improve.</h2><p>Answer one question at a time. Get explanations when you need them, save your answers, and review a practical 90-day plan.</p></section>
 <section class="card"><h2>What we’ll cover</h2><div class="domains"><div><b>People</b><p>Ownership, awareness and responsibilities.</p></div><div><b>Processes</b><p>Risk, access, incidents and recovery.</p></div><div><b>Technology</b><p>Accounts, devices, backups and monitoring.</p></div></div></section>
 <section class="card"><h2>Start your assessment</h2><label for="org">Organization or demo name</label><input id="org" maxlength="160" autocomplete="organization" placeholder="For example: 0D demo" value="${esc(state.org)}">
 <label for="mode">Assessment scope</label><select id="mode"><option value="quick" ${state.mode==='quick'?'selected':''}>Quick baseline · 12 questions</option><option value="full" ${state.mode==='full'?'selected':''}>Full assessment · 30 questions</option></select>
 <p class="small">Score each control from 0–4, or choose Unknown / Not applicable. You can go back and update saved answers. Examples are illustrative.</p>
 <button id="start" class="primary" ${!state.connected?'disabled':''}>Start assessment →</button></section>
 <section class="card"><h2>Resume an assessment</h2><label for="resume-id">Assessment ID</label><input id="resume-id" placeholder="Paste your saved assessment ID" autocomplete="off"><button id="resume" class="secondary" ${!state.connected?'disabled':''}>Resume</button></section>
 <p class="fine">Independent ENISA-informed self-assessment. Not an official ENISA score, audit or certification.</p>`}
function current(){return state.data?.questions?.[state.index]}
function capture(){
 if(state.data?.view!=='questionnaire'||state.report)return;
 const q=current();if(!q||!el('answer'))return;
 state.drafts[q.id]={value:el('score').value,answer_text:el('answer').value,evidence:el('evidence').value,owner:el('owner').value,na_justification:el('na').value,follow_up_answer:el('follow-answer').value};
}
function questionnaire(){
 const d=state.data,q=current();if(!q)return '<section class="card">No questions found. Return to the overview.</section>';
 const a=state.drafts[q.id]??d.answers?.[q.id]??{};
 return `<section class="progress-card"><div class="row"><b>${esc(d.organization)}</b><span>${d.answered} / ${d.total} saved</span></div><progress max="${d.total}" value="${d.answered}"></progress><p class="small">${esc(d.mode==='full'?'Full assessment':'Quick baseline')} · Question ${state.index+1} of ${d.total} · ${esc(q.domain)}</p></section>
 <section class="card question"><span class="badge">${esc(q.id)} · ${esc(q.domain)}</span><h2>${esc(q.text)}</h2>
 <details><summary>Explain this question</summary><p>${esc(q.explanation)}</p></details>
 <details><summary>Show an answer example</summary><p>${esc(q.answer_example)}</p><p class="small">Illustrative only. Describe what your organization actually does.</p></details>
 <div id="answer-form" role="form" aria-label="Question answer"><label for="answer">Your answer / current practice</label><textarea id="answer" rows="4" maxlength="12000" placeholder="Describe what is in place, what is missing, and any exceptions…">${esc(a.answer_text)}</textarea>
 <label for="score">Implementation level <span aria-hidden="true">*</span></label><select id="score" required><option value="">Choose a level</option>${scales.map(([v,l])=>`<option value="${v}" ${a.value===v?'selected':''}>${v} · ${esc(l)}</option>`).join('')}</select>
 <div id="na-wrap" ${a.value==='NA'?'':'hidden'}><label for="na">Why is this not applicable? *</label><textarea id="na" rows="2" maxlength="4000" placeholder="Explain why this control does not apply…">${esc(a.na_justification)}</textarea></div>
 <label for="evidence">Evidence reference <span class="optional">optional</span></label><input id="evidence" maxlength="4000" placeholder="Document name, ticket reference or review date" value="${esc(a.evidence)}"><p class="small">${esc(q.evidence_hint)}</p>
 <label for="owner">Responsible person or role <span class="optional">optional</span></label><input id="owner" maxlength="160" placeholder="For example: founder / IT lead" value="${esc(a.owner)}">
 <details id="follow-details" ${a.follow_up_answer?'open':''}><summary>Further question / clarification</summary><p>${esc(q.follow_up)}</p><label for="follow-answer">Follow-up answer <span class="optional">optional</span></label><textarea id="follow-answer" rows="3" maxlength="6000" placeholder="Add details here, or leave this for later…">${esc(a.follow_up_answer)}</textarea></details>
 <div class="actions"><button type="button" id="back" class="secondary" ${state.index===0?'disabled':''}>← Back</button><button type="button" id="save-next" class="primary">${state.index===d.total-1?'Save & review':'Save & next →'}</button></div>
 <button type="button" id="skip" class="text-button">Save as Unknown & continue</button></div></section>
 <div class="row footer-row"><button id="review" class="text-button">Review saved results</button><button id="overview" class="text-button">Overview</button></div><p class="fine">Assessment ID: <span class="selectable">${esc(d.assessment_id)}</span><br>Keep this ID to resume while demo storage is available. “Saved” means accepted by the server; Render Free storage can reset.</p>`;
}
function report(){
 const r=state.report;const missing=r.needs_verification?.length||0;
 return `<section class="card"><span class="badge">Assessment review</span><h2>${esc(r.organization)}</h2><p class="big">${r.overall_index_pct==null?'Provisional':esc(r.overall_index_pct)+'%'}</p><p>${r.overall_index_pct==null?'Answer more questions before an overall score can be assigned.':esc(r.plugin_defined_maturity)+' · independent scoring model'}</p>
 <div class="domains">${r.domains.map(d=>`<div><b>${esc(d.domain)}</b><p>${d.score_pct==null?'—':esc(d.score_pct)+'%'}${d.provisional?' · provisional':''}</p><small>${esc(d.coverage_pct)}% coverage</small></div>`).join('')}</div><p class="small">${missing} unknown or unanswered controls still need verification. Evidence references do not independently verify a control.</p><button class="primary" id="return">Return to questions</button></section>
 <section class="card"><h2>Your 30 / 60 / 90-day starting plan</h2>${r.gaps.length?r.gaps.map(g=>`<article class="gap"><span class="badge">${esc(g.target)} · ${esc(g.priority)}</span><h3>${esc(g.question_id)} · ${esc(g.question)}</h3><p>${esc(g.action)}</p><p class="small">Owner: ${esc(g.owner||'To assign')} · Verify implementation and adapt priority to business impact.</p></article>`).join(''):'<p>No low-scoring gaps were found among the numeric answers. This does not establish that all controls are effective; review unknowns and verify evidence.</p>'}<p class="fine">${esc(r.caveat)}</p></section>`;
}
function render(){
 const notice=state.data?.demo_notice||'Public demo · use fictional data only. Saved answers are shared and temporary.';
 root.innerHTML=header()+`<aside class="demo">${esc(notice)}</aside><div id="status" role="status" aria-live="polite">${state.busy?'Saving / loading…':!state.connected?'Connecting to ChatGPT…':esc(state.notice)}</div><div id="error" role="alert" ${state.error?'':'hidden'}>${esc(state.error)}</div>`+(state.report?report():state.data?.view==='questionnaire'?questionnaire():overview());
 bind();
 if(state.busy) root.querySelectorAll('button,input,select,textarea').forEach(x=>x.disabled=true);
}
async function syncContext(){
 const d=state.data;if(d?.assessment_id)try{await app.updateModelContext({content:[{type:'text',text:`Assessment ${d.assessment_id} (${d.organization}): ${d.answered}/${d.total} answers saved. Current question: ${current()?.id||'review'}. Use get_assessment to read details; do not invent or repeat answers.`}]})}catch{}
}
async function save(unknown=false){
 const q=current(),a=state.drafts[q.id]||{};
 if(!unknown&&!a.value)throw new Error('Choose an implementation level before saving.');
 if(!unknown&&a.value==='NA'&&!a.na_justification?.trim())throw new Error('Explain why this control is not applicable.');
 const args={assessment_id:state.data.assessment_id,question_id:q.id,...a,value:unknown?'U':a.value};
 // A reference does not verify a control. Do not populate verified_at.
 await tool('record_answer',args);
 state.data=await tool('get_assessment',{assessment_id:state.data.assessment_id});
 delete state.drafts[q.id];state.notice=`${q.id} saved.`;
 if(state.index<state.data.questions.length-1)state.index++;
 else state.report=await tool('calculate_maturity',{assessment_id:state.data.assessment_id});
 void syncContext();
}
function bind(){
 if(el('expand'))el('expand').onclick=()=>app.requestDisplayMode({mode:'fullscreen'}).catch(()=>{state.error='Expanded view is unavailable in this host.';render()});
 if(el('org')){el('org').oninput=e=>state.org=e.target.value;el('mode').onchange=e=>state.mode=e.target.value;}
 if(el('start'))el('start').onclick=()=>act(async()=>{if(!state.org.trim())throw new Error('Enter an organization or demo name.');const created=await tool('create_assessment',{organization:state.org.trim(),mode:state.mode});receive({structuredContent:await tool('get_assessment',{assessment_id:created.assessment_id})});state.notice='Assessment started. Answer one question at a time.';void syncContext()});
 if(el('resume'))el('resume').onclick=()=>{const id=el('resume-id').value.trim();act(async()=>{if(!id)throw new Error('Enter an assessment ID.');receive({structuredContent:await tool('get_assessment',{assessment_id:id})});void syncContext()})};
 if(el('save-next'))el('save-next').onclick=()=>act(()=>save());
 if(el('score'))el('score').onchange=()=>{el('na-wrap').hidden=el('score').value!=='NA';el('na').required=el('score').value==='NA';capture()};
 if(el('back'))el('back').onclick=()=>{capture();state.index--;state.notice='Unsaved edits stay in this open form until you save.';render()};
 if(el('skip'))el('skip').onclick=()=>act(()=>save(true));
 if(el('review'))el('review').onclick=()=>act(async()=>{state.report=await tool('calculate_maturity',{assessment_id:state.data.assessment_id})});
 if(el('return'))el('return').onclick=()=>{state.report=null;render()};
 if(el('overview'))el('overview').onclick=()=>{capture();state.org=state.data.organization;state.notice=`Resume using assessment ID ${state.data.assessment_id}.`;state.data=null;state.report=null;render()};
}
app.ontoolresult=receive;
app.onhostcontextchanged=ctx=>{if(ctx.theme)document.documentElement.dataset.theme=ctx.theme};
render();
app.connect().then(()=>{state.connected=true;render()}).catch(e=>{state.error='Could not connect the form to ChatGPT. Refresh the app connection and open Assessment overview again.';render()});
