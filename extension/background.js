// fastuse browser: service worker.
// Talks to the `fastuse` program on this computer through Chrome's native messaging, and answers
// its requests about your tabs. It only sees sites you have allowed (click the toolbar icon on a
// tab to allow or stop allowing that site).

import { readPage } from './reader.js';

const HOST = 'com.fastuse.browser';
const VERSION = chrome.runtime.getManifest().version;

let port = null;
let backoff = 1000;
let browserName = 'Chrome';
let lastError = '';

// ---- allowed sites ---------------------------------------------------------------------------

async function allowedOrigins() {
  const { allowed } = await chrome.storage.local.get('allowed');
  return Array.isArray(allowed) ? allowed : [];
}

function originOf(url) {
  try {
    const u = new URL(url);
    return u.protocol === 'http:' || u.protocol === 'https:' ? u.origin : '';
  } catch (e) {
    return '';
  }
}

async function paintBadge(tabId, url) {
  const on = (await allowedOrigins()).includes(originOf(url));
  chrome.action.setBadgeText({ tabId, text: on ? 'ON' : '' });
  if (on) chrome.action.setBadgeBackgroundColor({ tabId, color: '#2F6BFF' });
}

chrome.action.onClicked.addListener(async (tab) => {
  const o = originOf(tab.url || '');
  if (!o) return;
  const list = await allowedOrigins();
  const next = list.includes(o) ? list.filter((x) => x !== o) : [...list, o];
  await chrome.storage.local.set({ allowed: next });
  const tabs = await chrome.tabs.query({});
  for (const t of tabs) if (originOf(t.url || '') === o) paintBadge(t.id, t.url);
});

chrome.tabs.onUpdated.addListener((id, info, tab) => {
  if (info.url || info.status === 'complete') paintBadge(id, tab.url || '');
});

// ---- requests from fastuse -------------------------------------------------------------------

async function listTabs() {
  const allowed = await allowedOrigins();
  const tabs = await chrome.tabs.query({});
  const mine = tabs.filter((t) => allowed.includes(originOf(t.url || '')));
  return {
    tabs: mine.map((t) => ({ id: t.id, title: t.title || '', url: t.url || '', active: t.active, window: t.windowId })),
    hidden: tabs.length - mine.length,
  };
}

async function pickTab(id) {
  const allowed = await allowedOrigins();
  if (id !== undefined && id !== null) {
    const t = await chrome.tabs.get(id).catch(() => null);
    if (!t) throw new Error('no tab ' + id);
    if (!allowed.includes(originOf(t.url || ''))) {
      throw new Error('tab ' + id + ' is not allowed: click the fastuse icon on that tab to allow ' + (originOf(t.url || '') || 'it'));
    }
    return t;
  }
  const act = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
  const all = await chrome.tabs.query({});
  const t = [...act, ...all].find((x) => allowed.includes(originOf(x.url || '')));
  if (!t) throw new Error('no allowed tab: open a page and click the fastuse icon on it to allow that site');
  return t;
}

async function pageState(p) {
  const t = await pickTab(p.tab);
  let res;
  try {
    res = await chrome.scripting.executeScript({ target: { tabId: t.id }, func: readPage, args: [{ max: p.max, text: p.text }] });
  } catch (e) {
    throw new Error("cannot read tab " + t.id + ": " + (e && e.message ? e.message : e));
  }
  const r = res && res[0] && res[0].result;
  if (!r) throw new Error('the page returned nothing (it may still be loading)');
  return { tab: { id: t.id, title: t.title || '', url: t.url || '' }, page: r };
}

const METHODS = {
  ping: async () => ({ pong: true }),
  'tabs.list': listTabs,
  'page.state': pageState,
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
