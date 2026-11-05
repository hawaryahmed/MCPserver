import { build } from 'esbuild';
import { mkdir, writeFile } from 'node:fs/promises';
const result=await build({entryPoints:['widget.js'],bundle:true,format:'esm',platform:'browser',write:false,outdir:'dist',minify:true});
const js=result.outputFiles.find(x=>x.path.endsWith('.js')).text.replace(/<\/script/gi,'<\\/script');
const css=result.outputFiles.find(x=>x.path.endsWith('.css')).text;
await mkdir('dist',{recursive:true});
await writeFile('dist/assessment.html',`<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Assessment overview</title><style>${css}</style></head><body><main id="root"></main><script type="module">${js}</script></body></html>`);
