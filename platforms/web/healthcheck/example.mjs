import init from "./pkg/fragmentcolor.js";

function qs(name) {
  const url = new URL(location.href);
  return url.searchParams.get(name);
}

async function fetchText(rel) {
  try {
    const res = await fetch(rel);
    if (!res.ok) throw new Error(String(res.status));
    return await res.text();
  } catch (e) {
    return `// Failed to load source: ${e?.message || String(e)}`;
  }
}

(async function start(){
  const rel = qs('rel');
  const meta = document.getElementById('meta');
  const code = document.getElementById('code');

  if (!rel) {
    meta.textContent = 'Missing ?rel=../examples/.. path';
    return;
  }

  meta.textContent = rel;
  code.textContent = await fetchText(rel);

  try {
    const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
    await init({ module_or_path: wasmUrl.href });
  } catch (e) {
    console.warn('init failed', e);
  }

  try {
    await import(rel);
  } catch (e) {
    console.error('example failed', e);
  }
})();
