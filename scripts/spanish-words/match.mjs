import fs from 'fs'; import path from 'path'; 
import sharp from 'sharp';
const D = process.argv[2], MEDIA = process.argv[3];
const groups = JSON.parse(fs.readFileSync(path.join(D,'ambiguous.json')));
const ids = [...new Set(groups.flatMap(g=>g.ids))];
async function get(id){ const f=path.join(D,'img',`${id}.png`); if(fs.existsSync(f)&&fs.statSync(f).size>0) return;
  for(let t=0;t<3;t++){ try{ const r=await fetch(`https://static.arasaac.org/pictograms/${id}/${id}_300.png`); if(r.ok){fs.writeFileSync(f,Buffer.from(await r.arrayBuffer()));return;} }catch{} } }
let i=0; await Promise.all(Array.from({length:8},async()=>{ while(i<ids.length){ await get(ids[i++]); } }));
console.error('downloaded');
const vec = async f => { try { return await sharp(f).flatten({background:'#fff'}).resize(24,24,{fit:'contain',background:'#fff'}).grayscale().raw().toBuffer(); } catch { return null; } };
const dist=(a,b)=>{let s=0;for(let k=0;k<a.length;k++){const d=a[k]-b[k];s+=d*d;}return s;};
const result={}; const sure=[];
for(const g of groups){
  const fv=await Promise.all(g.files.map(f=>vec(path.join(MEDIA,f))));
  const iv=await Promise.all(g.ids.map(id=>vec(path.join(D,'img',`${id}.png`))));
  const pairs=[]; fv.forEach((a,x)=>iv.forEach((b,y)=>{ if(a&&b) pairs.push([dist(a,b),x,y]); }));
  pairs.sort((p,q)=>p[0]-q[0]); const ux=new Set(), uy=new Set();
  for(const [d,x,y] of pairs){ if(ux.has(x)||uy.has(y)) continue; ux.add(x); uy.add(y); result[g.files[x]]={id:g.ids[y],d}; }
}
fs.writeFileSync(path.join(D,'matched.json'),JSON.stringify(result));
const ds=Object.values(result).map(r=>r.d).sort((a,b)=>a-b); console.log(Object.keys(result).length, 'median',ds[ds.length>>1],'p90',ds[Math.floor(ds.length*.9)],'max',ds[ds.length-1]);
