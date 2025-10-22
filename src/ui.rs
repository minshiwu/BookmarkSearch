use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum IpcMessage {
    #[serde(rename_all = "camelCase")]
    Search { query: String },
    #[serde(rename_all = "camelCase")]
    Open { url: String },
    Hide,
}

pub const HTML_INDEX: &str = r#"<!DOCTYPE html>
<html lang=\"zh-CN\">
<meta charset=\"utf-8\"/>
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>
<title>Bookmark Launcher</title>
<style>
  :root{color-scheme:dark light}
  body{margin:0;padding:0;background:rgba(22,22,22,.85);font:16px/1.4 system-ui,Segoe UI,Roboto,Helvetica,Arial;color:#f6f6f6}
  .wrap{padding:24px 24px 10px}
  .search{width:100%;box-sizing:border-box;border:none;outline:none;border-radius:10px;padding:14px 16px;font-size:20px;background:#111;color:#fff}
  .list{max-height:420px;overflow:auto;margin-top:14px}
  .item{padding:10px 12px;border-radius:8px;cursor:pointer;display:flex;gap:10px;align-items:center}
  .item:hover,.item.active{background:#2a2a2a}
  .title{font-weight:600}
  .url{font-size:12px;color:#999}
  .badge{font-size:12px;color:#ccc;background:#333;padding:2px 6px;border-radius:6px}
</style>
<body>
  <div class=\"wrap\">
    <input id=\"q\" class=\"search\" placeholder=\"输入关键词（中文/英文/拼音），回车打开...\" autofocus />
    <div id=\"list\" class=\"list\"></div>
  </div>
<script>
  const q = document.getElementById('q');
  const list = document.getElementById('list');
  let cursor = -1; let data = [];

  window.__focusInput = () => { q.focus(); q.select(); }
  window.__setResults = (items) => {
    data = items || []; cursor = -1; render();
  }

  function render(){
    list.innerHTML = data.map((it,i)=>`<div class=\"item ${i===cursor?'active':''}\" data-i=\"${i}\">\n <div style=\"flex:1\">\n  <div class=\"title\">${esc(it.title)}</div>\n  <div class=\"url\">${esc(it.url)}</div>\n </div>\n <div class=\"badge\">${esc(it.browser)}</div>\n</div>`).join('');
  }

  function esc(s){ return (s+"").replace(/[&<>]/g, c=>({"&":"&amp;","<":"&lt;",">":"&gt;"}[c])); }

  const post = (obj)=>window.ipc.postMessage(JSON.stringify(obj));

  let timer = 0;
  q.addEventListener('input', ()=>{
    clearTimeout(timer);
    timer = setTimeout(()=>post({Search:{query:q.value}}), 60);
  });

  document.addEventListener('keydown', (e)=>{
    if(e.key==='Escape'){ post('Hide'); }
    if(e.key==='ArrowDown'){ cursor = Math.min(cursor+1, data.length-1); render(); e.preventDefault(); }
    if(e.key==='ArrowUp'){ cursor = Math.max(cursor-1, -1); render(); e.preventDefault(); }
    if(e.key==='Enter'){
      const it = data[cursor>=0?cursor:0]; if(it){ post({Open:{url:it.url}}); }
    }
  });

  list.addEventListener('click', (e)=>{
    const el = e.target.closest('.item'); if(!el) return; const i = +el.dataset.i; const it = data[i]; if(it) post({Open:{url:it.url}});
  });
</script>
</body>
</html>"#;
