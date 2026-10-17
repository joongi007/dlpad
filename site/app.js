'use strict';
(() => {
  const root = document.documentElement;
  const system = matchMedia('(prefers-color-scheme: dark)');
  const theme = document.getElementById('theme');
  let preference = null;
  try { preference = localStorage.getItem('dlpad-site-theme'); } catch { /* Storage is optional. */ }
  function applyTheme() {
    const dark = preference === 'dark' || (preference !== 'light' && system.matches);
    root.dataset.theme = dark ? 'dark' : 'light';
    theme.setAttribute('aria-label', `${dark ? '밝은' : '어두운'} 테마로 변경`);
    theme.title = theme.getAttribute('aria-label');
  }
  applyTheme();
  system.addEventListener('change', applyTheme);
  theme.addEventListener('click', () => {
    preference = root.dataset.theme === 'dark' ? 'light' : 'dark';
    try { localStorage.setItem('dlpad-site-theme', preference); } catch { /* Keep the in-memory choice. */ }
    applyTheme();
  });
  const fallback = 'https://github.com/joongi007/dlpad/releases';
  const status = document.getElementById('release-state');
  async function loadRelease() {
    const abort = new AbortController();
    const timer = setTimeout(() => abort.abort(), 5000);
    try {
      const response = await fetch('release.json', { signal: abort.signal, credentials: 'omit', cache: 'no-cache' });
      if (!response.ok) throw new Error('unavailable');
      const reader = response.body.getReader();
      const chunks = [];
      let count = 0;
      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        count += value.byteLength;
        if (count > 8192) { await reader.cancel(); throw new Error('oversized'); }
        chunks.push(value);
      }
      const bytes = new Uint8Array(count);
      let offset = 0;
      for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
      const body = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
      const data = JSON.parse(body);
      if (data.schema !== 1) throw new Error('schema');
      if (data.release === null) {
        status.textContent = '첫 공개 릴리스를 준비하고 있습니다. 공개되면 여기에서 받을 수 있습니다.';
        return;
      }
      const r = data.release;
      if (!/^\d+\.\d+\.\d+$/.test(r.version) || typeof r.preview !== 'boolean') throw new Error('version');
      for (const kind of ['installer', 'portable']) {
        const expected = `https://github.com/joongi007/dlpad/releases/download/v${r.version}/dlpad-${r.version}-windows-x86_64-${kind}.zip`;
        if (r[kind] !== expected) throw new Error('asset');
      }
      status.textContent = `v${r.version}${r.preview ? ' · 미리보기 버전' : ''} · Windows 64비트${r.signed === true ? '' : ' · 코드 서명 없음'}`;
      for (const [kind, label] of [['installer', '설치형 ZIP 받기'], ['portable', '포터블 ZIP 받기']]) {
        const link = document.getElementById(`${kind}-link`);
        link.href = r[kind];
        link.textContent = `${label} ↓`;
      }
    } catch {
      status.textContent = '릴리스 정보를 불러오지 못했습니다. GitHub에서 공개 버전을 확인하세요.';
      for (const kind of ['installer', 'portable']) document.getElementById(`${kind}-link`).href = fallback;
    } finally { clearTimeout(timer); }
  }
  loadRelease();
})();
