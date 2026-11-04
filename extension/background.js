// fastuse browser: service worker.
// Talks to the `fastuse` program on this computer through Chrome's native messaging.
//
// What the agent may touch:
//   - tabs it opens itself (they go into a "fastuse" tab group, in your own profile, so your logins apply),
//   - tabs you share with it (popup > "Share this tab"),
//   and nothing else, never sites on the blocklist, and nothing at all while it is paused in the popup.

import { readPage } from './reader.js';
import { runSteps } from './actions.js';

const HOST = 'com.fastuse.browser';
const VERSION = chrome.runtime.getManifest().version;

let port = null;
let backoff = 1000;
let browserName = 'Chrome';
let lastError = '';

// Sites the agent may not open or control, to keep it away from logins, payments and secrets.
// Matches the host name and any subdomain. Editable later; this is the built-in minimum.
const BLOCKED_HOSTS = [
  'accounts.google.com', 'login.live.com', 'login.microsoftonline.com', 'appleid.apple.com', 'id.apple.com',
  'paypal.com', 'pay.google.com', 'wallet.google.com', 'stripe.com', 'checkout.stripe.com', 'klarna.com',
  '1password.com', 'bitwarden.com', 'lastpass.com', 'dashlane.com', 'pass.proton.me', 'account.proton.me',
  'coinbase.com', 'binance.com', 'kraken.com', 'metamask.io',
];
const BLOCKED_WORDS = /(^|\.)(bank|banking|onlinebanking|ebank)[a-z0-9-]*\./i;

function blockedReason(url) {
  let u;
  try { u = new URL(url); } catch (e) { return ''; }
  if (u.protocol !== 'http:' && u.protocol !== 'https:') return '';
  const h = u.hostname.toLowerCase();
  if (BLOCKED_HOSTS.some((b) => h === b || h.endsWith('.' + b)) || BLOCKED_WORDS.test(h)) {
    return h + ' is on the sensitive-site blocklist (logins, payments, banking, passwords): ask the user to do this part';
  }
  return '';
}

// ---- state: what is the agent allowed to use -------------------------------------------------

async function settings() {
  const { paused, shared } = await chrome.storage.local.get(['paused', 'shared']);
  return { paused: !!paused, shared: Array.isArray(shared) ? shared : [] };
}

async function sessionGet() {
  const s = await chrome.storage.session.get(['owned', 'last']);
  return { owned: Array.isArray(s.owned) ? s.owned : [], last: s.last ?? null };
}

async function setOwned(owned, last) {
  const cur = await sessionGet();
  await chrome.storage.session.set({ owned, last: last === undefined ? cur.last : last });
}

function originOf(url) {
  try {
    const u = new URL(url);
    return u.protocol === 'http:' || u.protocol === 'https:' ? u.origin : '';
  } catch (e) {
    return '';
  }
}

async function controllable(tab) {
  const st = await settings();
  if (st.paused || !tab) return false;
  if (blockedReason(tab.url || '')) return false;
  const ses = await sessionGet();
  return ses.owned.includes(tab.id) || st.shared.includes(originOf(tab.url || ''));
}

async function paintBadge(tab) {
  if (!tab || tab.id === undefined) return;
  const ses = await sessionGet();
  const st = await settings();
  const text = st.paused ? '' : ses.owned.includes(tab.id) ? 'AI' : st.shared.includes(originOf(tab.url || '')) ? 'ON' : '';
  chrome.action.setBadgeText({ tabId: tab.id, text });
  if (text) chrome.action.setBadgeBackgroundColor({ tabId: tab.id, color: '#2F6BFF' });
}

chrome.tabs.onUpdated.addListener((id, info, tab) => {
  if (info.url || info.status === 'complete') paintBadge(tab);
});

chrome.tabs.onRemoved.addListener(async (id) => {
  const ses = await sessionGet();
  if (ses.owned.includes(id)) await setOwned(ses.owned.filter((x) => x !== id), ses.last === id ? null : ses.last);
});

// a link in an agent tab that opens a new tab (target=_blank) stays under the agent's control
chrome.tabs.onCreated.addListener(async (tab) => {
  if (tab.openerTabId === undefined) return;
  const ses = await sessionGet();
  if (ses.owned.includes(tab.openerTabId)) await setOwned([...ses.owned, tab.id]);
});

// ---- tabs ------------------------------------------------------------------------------------

async function groupTab(tabId, windowId) {
  const groups = await chrome.tabGroups.query({ windowId, title: 'fastuse' }).catch(() => []);
  let gid = groups.length ? groups[0].id : undefined;
  gid = await chrome.tabs.group(gid === undefined ? { tabIds: [tabId] } : { tabIds: [tabId], groupId: gid });
  await chrome.tabGroups.update(gid, { title: 'fastuse', color: 'blue' }).catch(() => {});
}

async function newOwnedTab(url) {
  const why = blockedReason(url);
  if (why) throw new Error(why);
  const win = await chrome.windows.getLastFocused({ windowTypes: ['normal'] }).catch(() => null);
  const tab = await chrome.tabs.create({ url, active: false, windowId: win ? win.id : undefined });
  await groupTab(tab.id, tab.windowId).catch(() => {});
  const ses = await sessionGet();
  await setOwned([...ses.owned, tab.id], tab.id);
  return tab;
}

function waitLoaded(tabId, limit = 15000) {
  return new Promise((resolve) => {
    const t0 = Date.now();
    const tick = async () => {
      const t = await chrome.tabs.get(tabId).catch(() => null);
      if (!t || t.status === 'complete' || Date.now() - t0 > limit) return resolve(t);
      setTimeout(tick, 60);
    };
    tick();
  });
}

// after an action: if the page started to navigate, wait for it
async function settle(tabId) {
  await new Promise((r) => setTimeout(r, 90));
  let t = await chrome.tabs.get(tabId).catch(() => null);
  if (t && t.status === 'loading') t = await waitLoaded(tabId, 10000);
  await new Promise((r) => setTimeout(r, 40));
  return t;
}

async function listTabs() {
  const tabs = await chrome.tabs.query({});
  const ses = await sessionGet();
  const mine = [];
  for (const t of tabs) if (await controllable(t)) mine.push(t);
  return {
    tabs: mine.map((t) => ({ id: t.id, title: t.title || '', url: t.url || '', active: t.active, owned: ses.owned.includes(t.id) })),
    hidden: tabs.length - mine.length,
  };
}

async function pickTab(id) {
  if (id !== undefined && id !== null) {
    const t = await chrome.tabs.get(id).catch(() => null);
    if (!t) throw new Error('no tab ' + id);
    const why = blockedReason(t.url || '');
    if (why) throw new Error(why);
    if (!(await controllable(t))) throw new Error('tab ' + id + ' is not one the agent may use: open pages with browser_act (navigate), or ask the user to share that tab in the fastuse popup');
    return t;
  }
  const t = await currentTab();
  if (!t) {
    const st = await settings();
    throw new Error(st.paused ? 'the agent is paused in the fastuse popup' : 'no tab yet: use browser_act with a navigate step to open a page (it opens in the "fastuse" tab group)');
  }
  return t;
}

async function currentTab() {
  const ses = await sessionGet();
  if (ses.last !== null) {
    const t = await chrome.tabs.get(ses.last).catch(() => null);
    if (t && (await controllable(t))) return t;
  }
  const all = await chrome.tabs.query({});
  for (const o of ses.owned) {
    const t = all.find((x) => x.id === o);
    if (t && (await controllable(t))) return t;
  }
  const act = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
  for (const t of [...act, ...all]) if (await controllable(t)) return t;
  return null;
}

// ---- reading and acting ----------------------------------------------------------------------

async function read(tab, p) {
  let res;
  try {
    res = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func: readPage, args: [{ max: p.max, text: p.text, snap: p.snap_next || p.snap || '' }] });
  } catch (e) {
    throw new Error('cannot read tab ' + tab.id + ': ' + (e && e.message ? e.message : e));
  }
  const r = res && res[0] && res[0].result;
  if (!r) throw new Error('the page returned nothing (it may still be loading)');
  await setOwned((await sessionGet()).owned, tab.id);
  const t = await chrome.tabs.get(tab.id).catch(() => tab);
  return { tab: { id: t.id, title: t.title || '', url: t.url || '' }, page: r };
}

async function pageState(p) {
  return read(await pickTab(p.tab), p);
}

const TAB_TOOLS = new Set(['navigate', 'new_tab', 'close_tab', 'back', 'forward', 'reload', 'switch_tab']);

async function tabStep(s, tab) {
  const t = s.tool;
  if (t === 'new_tab') {
    if (!s.url) throw new Error('new_tab needs url');
    const n = await newOwnedTab(normalizeUrl(s.url));
    const done = await waitLoaded(n.id);
    return { tab: done || n, msg: 'ok: new_tab ' + (done ? done.url : s.url) };
  }
  if (t === 'navigate') {
    if (!s.url) throw new Error('navigate needs url');
    const url = normalizeUrl(s.url);
    const why = blockedReason(url);
    if (why) throw new Error(why);
    if (!tab) return tabStep({ ...s, tool: 'new_tab' }, null);
    await chrome.tabs.update(tab.id, { url });
    const done = await waitLoaded(tab.id);
    return { tab: done || tab, msg: 'ok: navigate ' + url };
  }
  if (!tab) throw new Error(t + ' needs a tab: start with navigate or new_tab');
  if (t === 'back') await chrome.tabs.goBack(tab.id);
  else if (t === 'forward') await chrome.tabs.goForward(tab.id);
  else if (t === 'reload') await chrome.tabs.reload(tab.id);
  else if (t === 'switch_tab') {
    const n = await chrome.tabs.get(Number(s.tab)).catch(() => null);
    if (!n || !(await controllable(n))) throw new Error('tab ' + s.tab + ' is not one the agent may use');
    await setOwned((await sessionGet()).owned, n.id);
    if (s.focus) await chrome.tabs.update(n.id, { active: true });
    return { tab: n, msg: 'ok: switch_tab ' + n.id };
  } else if (t === 'close_tab') {
    const ses = await sessionGet();
    if (!ses.owned.includes(tab.id)) throw new Error('only tabs the agent opened can be closed by it');
    await chrome.tabs.remove(tab.id);
    return { tab: null, msg: 'ok: close_tab ' + tab.id };
  }
  const done = await waitLoaded(tab.id);
  return { tab: done || tab, msg: 'ok: ' + t };
}

function normalizeUrl(u) {
  u = String(u).trim();
  return /^[a-z][a-z0-9+.-]*:/i.test(u) ? u : 'https://' + u;
}

async function actRun(p) {
  const steps = Array.isArray(p.steps) ? p.steps : [];
  if (!steps.length) throw new Error('steps is empty');
  let tab = p.tab !== undefined && p.tab !== null ? await pickTab(p.tab) : await currentTab();
  const log = [];
  let failed = false;
  let i = 0;
  while (i < steps.length && !failed) {
    const s = steps[i];
    if (TAB_TOOLS.has(s.tool)) {
      try {
        const r = await tabStep(s, tab);
        tab = r.tab;
        if (tab) await setOwned((await sessionGet()).owned, tab.id);
        log.push(i + 1 + '. ' + r.msg);
      } catch (e) {
        log.push(i + 1 + '. FAILED ' + s.tool + ': ' + (e && e.message ? e.message : e));
        failed = true;
      }
      i++;
      continue;
    }
    let j = i;
    while (j < steps.length && !TAB_TOOLS.has(steps[j].tool)) j++;
    if (!tab) {
      log.push(i + 1 + '. FAILED ' + s.tool + ': there is no page yet: start with a navigate step');
      failed = true;
      break;
    }
    const why = blockedReason(tab.url || '');
    if (why || !(await controllable(tab))) {
      log.push(i + 1 + '. FAILED ' + s.tool + ': ' + (why || 'the agent may not use this tab'));
      failed = true;
      break;
    }
    let out;
    try {
      const res = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func: runSteps, args: [steps.slice(i, j)] });
      out = res && res[0] && res[0].result;
    } catch (e) {
      out = { log: ['FAILED ' + s.tool + ': ' + (e && e.message ? e.message : e)], failed: true, done: 0 };
    }
    if (!out) out = { log: ['FAILED ' + s.tool + ': the page did not answer'], failed: true, done: 0 };
    out.log.forEach((l, k) => log.push(i + k + 1 + '. ' + l));
    failed = out.failed;
    i = j;
    if (!failed && tab) tab = (await settle(tab.id)) || tab;
  }
  const result = { log, failed };
  if (p.observe !== false && tab) {
    const live = await chrome.tabs.get(tab.id).catch(() => null);
    if (live) {
      try {
        const why = blockedReason(live.url || '');
        if (why) result.note = 'the page moved to a blocked site: ' + why;
        else result.state = await read(live, { ...p, snap: p.snap_next });
      } catch (e) {
        result.note = String(e && e.message ? e.message : e);
      }
    }
  }
  return result;
}

const METHODS = {
  ping: async () => ({ pong: true }),
  'tabs.list': listTabs,
  'page.state': pageState,
  'act.run': actRun,
};

async function onMessage(msg) {
  if (!msg || typeof msg.id !== 'number') return;
  try {
    const fn = METHODS[msg.method];
    if (!fn) throw new Error('unknown method ' + msg.method);
    const result = await fn(msg.params || {});
    port && port.postMessage({ id: msg.id, result: { ...result, browser: { name: browserName, extension: VERSION } } });
  } catch (e) {
    port && port.postMessage({ id: msg.id, error: String(e && e.message ? e.message : e) });
  }
}

// ---- popup -----------------------------------------------------------------------------------

chrome.runtime.onMessage.addListener((m, sender, reply) => {
  (async () => {
    const st = await settings();
    const ses = await sessionGet();
    if (m.type === 'toggle-pause') await chrome.storage.local.set({ paused: !st.paused });
    if (m.type === 'toggle-share' && m.origin) {
      const next = st.shared.includes(m.origin) ? st.shared.filter((x) => x !== m.origin) : [...st.shared, m.origin];
      await chrome.storage.local.set({ shared: next });
    }
    const now = await settings();
    const tabs = await chrome.tabs.query({});
    for (const t of tabs) paintBadge(t);
    reply({ paused: now.paused, shared: now.shared, owned: ses.owned.length, connected: !!port, error: lastError, browser: browserName, version: VERSION });
  })();
  return true;
});

// ---- connection to the native host -----------------------------------------------------------

async function detectBrowser() {
  try {
    if (navigator.brave && (await navigator.brave.isBrave())) return 'Brave';
  } catch (e) { /* not Brave */ }
  const brands = (navigator.userAgentData && navigator.userAgentData.brands) || [];
  if (brands.some((b) => /Edge/.test(b.brand))) return 'Edge';
  if (brands.some((b) => /Opera/.test(b.brand))) return 'Opera';
  if (brands.some((b) => /Vivaldi/.test(b.brand))) return 'Vivaldi';
  return 'Chrome';
}

let pingTimer = null;

function connect() {
  if (port) return;
  try {
    port = chrome.runtime.connectNative(HOST);
  } catch (e) {
    lastError = String(e);
    schedule();
    return;
  }
  port.onMessage.addListener(onMessage);
  port.onDisconnect.addListener(() => {
    lastError = (chrome.runtime.lastError && chrome.runtime.lastError.message) || 'disconnected';
    port = null;
    clearInterval(pingTimer);
    schedule();
  });
  detectBrowser().then((name) => {
    browserName = name;
    port && port.postMessage({ event: 'hello', browser: name, extension: VERSION, id: chrome.runtime.id });
  });
  backoff = 1000;
  lastError = '';
  // traffic on the port also counts as activity, so the worker is not put to sleep while connected
  clearInterval(pingTimer);
  pingTimer = setInterval(() => port && port.postMessage({ event: 'ping' }), 20000);
}

function schedule() {
  setTimeout(connect, backoff);
  backoff = Math.min(backoff * 2, 15000);
}

// the worker can be stopped by the browser; these wake it up again and reconnect
chrome.alarms.create('fastuse-reconnect', { periodInMinutes: 0.5 });
chrome.alarms.onAlarm.addListener(() => connect());
chrome.runtime.onStartup.addListener(connect);
chrome.runtime.onInstalled.addListener(connect);

connect();
