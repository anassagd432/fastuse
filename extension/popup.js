const $ = (id) => document.getElementById(id);
let origin = '';

function ask(m) {
  return chrome.runtime.sendMessage(m);
}

function paint(s) {
  $('dot').className = 'dot' + (s.connected && !s.paused ? ' on' : '');
  $('status').textContent = s.paused
    ? 'Paused: the agent cannot see or use any tab.'
    : s.connected
      ? 'Connected to fastuse. The agent has ' + s.owned + ' tab' + (s.owned === 1 ? '' : 's') + ' of its own.'
      : 'Not connected to fastuse' + (s.error ? ' (' + s.error + ')' : '') + '. Run “fastuse browser-install”, then reload this extension.';
  $('pause').textContent = s.paused ? 'Let the agent work again' : 'Pause the agent';
  $('pause').className = s.paused ? '' : 'stop';
  const unshared = Array.isArray(s.unshared) ? s.unshared : [];
  const shared = origin && !unshared.includes(origin);
  $('share').style.display = origin ? '' : 'none';
  $('share').textContent = shared ? 'Block agent on this site' : 'Allow agent on this site';
}

(async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  try {
    const u = new URL(tab.url);
    if (u.protocol === 'http:' || u.protocol === 'https:') origin = u.origin;
  } catch (e) { /* not a web page */ }
  paint(await ask({ type: 'status' }));
  $('pause').onclick = async () => paint(await ask({ type: 'toggle-pause' }));
  $('share').onclick = async () => paint(await ask({ type: 'toggle-share', origin }));
})();
