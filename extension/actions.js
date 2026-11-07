// Runs a batch of page steps inside the page. Injected with chrome.scripting.executeScript({func: runSteps}),
// so it must be completely self-contained. It resolves targets against the snapshot that reader.js left
// in this frame (globalThis.__fastuse) and stops at the first failing step.
//
// Steps: click, double_click, right_click, hover, fill (replace text), type (append text), select,
//        press (key), scroll, wait. All events are synthetic (isTrusted=false); sites that insist on
//        trusted input need the debugger route that comes later.

export async function runSteps(steps) {
  const S = globalThis.__fastuse;
  const log = [];
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const lc = (s) => (s || '').toLowerCase();
  const cut = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);

  function moveOverlay(x, y, isClick) {
    try {
      let ui = globalThis.__fastuseOverlay;
      if (!ui || !ui.host || !ui.host.isConnected) {
        const host = document.createElement('div');
        host.setAttribute('data-fastuse-overlay', '1');
        host.setAttribute('aria-hidden', 'true');
        host.style.cssText = 'position:fixed;inset:0;pointer-events:none;z-index:2147483647;overflow:hidden;';
        const root = host.attachShadow({ mode: 'closed' });
        root.innerHTML = `
          <style>
            .frame { position:fixed; inset:0; pointer-events:none; box-shadow:inset 0 0 28px rgba(47,107,255,0.42), inset 0 0 2px rgba(47,107,255,0.85); transition:opacity .25s ease; }
            .pill { position:fixed; top:10px; left:50%; transform:translateX(-50%); background:rgba(29,66,158,0.92); border:1px solid rgba(151,181,255,0.65); color:#fff; font:600 12px/28px system-ui,-apple-system,sans-serif; padding:0 16px; border-radius:999px; box-shadow:0 4px 14px rgba(0,0,0,0.22); white-space:nowrap; }
            .cur { position:fixed; left:0; top:0; width:28px; height:28px; pointer-events:none; transition:transform .16s cubic-bezier(.22,1,.36,1), opacity .25s ease; filter:drop-shadow(0 2px 5px rgba(47,107,255,0.55)); }
            .ring { position:fixed; left:0; top:0; width:26px; height:26px; margin:-13px 0 0 -13px; border:2px solid #2F6BFF; border-radius:50%; opacity:0; pointer-events:none; }
            .ring.pop { animation:fu-pop .32s ease-out; }
            @keyframes fu-pop { 0% { transform:translate(var(--x),var(--y)) scale(.35); opacity:.85; } 100% { transform:translate(var(--x),var(--y)) scale(1.45); opacity:0; } }
          </style>
          <div class="frame"><div class="pill">This browser tab is being controlled by fastuse</div></div>
          <div class="ring"></div>
          <svg class="cur" viewBox="0 0 24 24"><path d="M2 2 L2 19 L6.2 14.8 L9.3 21.5 L12.2 20.2 L9.2 13.5 L15 13.5 Z" fill="#2F6BFF" stroke="#fff" stroke-width="1.6" stroke-linejoin="round"/></svg>
        `;
        (document.documentElement || document.body).appendChild(host);
        ui = globalThis.__fastuseOverlay = {
          host,
          frame: root.querySelector('.frame'),
          cur: root.querySelector('.cur'),
          ring: root.querySelector('.ring'),
          timer: null,
        };
      }
      ui.frame.style.opacity = '1';
      ui.cur.style.opacity = '1';
      ui.cur.style.transform = `translate(${Math.round(x - 2)}px, ${Math.round(y - 2)}px)`;
      if (isClick) {
        ui.ring.style.setProperty('--x', `${Math.round(x)}px`);
        ui.ring.style.setProperty('--y', `${Math.round(y)}px`);
        ui.ring.classList.remove('pop');
        void ui.ring.offsetWidth;
        ui.ring.classList.add('pop');
      }
      clearTimeout(ui.timer);
      ui.timer = setTimeout(() => {
        if (ui.frame) ui.frame.style.opacity = '0';
        if (ui.cur) ui.cur.style.opacity = '0';
      }, 6000);
    } catch (e) { /* overlay is cosmetic */ }
  }

  function retryErr(msg) {
    const e = new Error(msg);
    e.retryAfterRead = true;
    return e;
  }

  function byName(target) {
    const meta = S.meta;
    const t = String(target).trim();
    const bracket = t.startsWith('[') && t.endsWith(']') ? t.slice(1, -1) : t.startsWith('#') ? t.slice(1) : null;
    if (bracket !== null && /^\d+$/.test(bracket.trim())) {
      const idx = parseInt(bracket.trim(), 10);
      if (idx >= 0 && idx < meta.length) return idx;
      throw retryErr('no element [' + idx + '] in snapshot');
    }
    let role = null;
    let want = t;
    const c = t.indexOf(':');
    if (c > 0) {
      const prefix = t.slice(0, c);
      if (/^s\d+$/.test(prefix) && /^\d+$/.test(t.slice(c + 1).trim())) {
        if (prefix !== S.snap) throw new Error('stale element_token ' + t + ': latest snapshot is ' + S.snap + '; read the page again');
        const idx = parseInt(t.slice(c + 1).trim(), 10);
        if (idx >= 0 && idx < meta.length) return idx;
        throw retryErr('no element [' + idx + '] in snapshot');
      }
      if (meta.some((m) => lc(m.role) === lc(prefix))) {
        role = lc(prefix);
        want = t.slice(c + 1);
      }
    }
    const w = lc(want.trim());
    const all = meta.map((_, i) => i).filter((i) => !role || lc(meta[i].role) === role);
    let hits = all.filter((i) => lc(meta[i].name) === w || lc(meta[i].id) === w);
    if (!hits.length) hits = all.filter((i) => lc(meta[i].name).includes(w) || lc(meta[i].id).includes(w));
    if (hits.length === 1) return hits[0];
    if (!hits.length) throw retryErr("no element matches target '" + target + "'");
    throw new Error("target '" + target + "' is ambiguous: " + hits.slice(0, 6).map((i) => '[' + i + '] ' + meta[i].role + ' "' + meta[i].name + '"').join(', ') + '. Use Role:Name or an element_token.');
  }

  function resolve(step) {
    if (!S) throw retryErr('this page has not been read yet (or it reloaded): read it again first');
    const tok = step.element_token !== undefined ? step.element_token : step.ref;
    let idx;
    if (tok !== undefined && tok !== null) {
      const s = String(tok);
      const parts = s.includes(':') ? s.split(':') : [S.snap, s];
      if (parts[0] !== S.snap) throw new Error('stale element_token ' + s + ': latest snapshot is ' + S.snap + '; read the page again');
      idx = parseInt(parts[1], 10);
    } else if (step.target) {
      idx = byName(String(step.target));
    } else {
      return null;
    }
    const el = S.refs[idx];
    if (!el || !el.isConnected) throw retryErr('element [' + idx + '] is no longer on the page; read it again');
    return { el, idx, m: S.meta[idx] };
  }

  const label = (r) => '[' + r.idx + '] ' + r.m.role + (r.m.name ? ' "' + cut(r.m.name, 40) + '"' : '');

  function covered(el) {
    const r = el.getBoundingClientRect();
    const doc = el.ownerDocument;
    const top = doc.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
    return !!top && top !== el && !el.contains(top) && !top.contains(el);
  }

  async function click(el, kind) {
    el.scrollIntoView({ block: 'center', inline: 'nearest' });
    const r = el.getBoundingClientRect();
    const x = r.left + r.width / 2;
    const y = r.top + r.height / 2;
    moveOverlay(x, y, true);
    const win = el.ownerDocument.defaultView || window;
    const o = { bubbles: true, cancelable: true, composed: true, view: win, clientX: x, clientY: y, button: kind === 'right' ? 2 : 0, buttons: 1 };
    const fire = (t, C) => el.dispatchEvent(new C(t, o));
    fire('pointerover', win.PointerEvent);
    fire('mouseover', win.MouseEvent);
    fire('pointermove', win.PointerEvent);
    fire('mousemove', win.MouseEvent);
    fire('pointerdown', win.PointerEvent);
    fire('mousedown', win.MouseEvent);
    if (el.focus) el.focus({ preventScroll: true });
    fire('pointerup', win.PointerEvent);
    fire('mouseup', win.MouseEvent);
    if (kind === 'right') fire('contextmenu', win.MouseEvent);
    else if (kind === 'double') {
      el.click();
      el.click();
      fire('dblclick', win.MouseEvent);
    } else el.click();
  }

  function guardSecret(el) {
    const ty = lc(el.type || '');
    const ac = lc(el.getAttribute && el.getAttribute('autocomplete'));
    if (ty === 'password') throw new Error('refusing to type into a password field: ask the user to enter it');
    if (ac.includes('cc-') || ac.includes('one-time-code')) throw new Error('refusing to type into a payment or one-time-code field: ask the user to enter it');
  }

  function setText(el, text, append) {
    const tag = el.tagName;
    const win = el.ownerDocument.defaultView || window;
    if (tag === 'INPUT' || tag === 'TEXTAREA') {
      guardSecret(el);
      el.focus({ preventScroll: true });
      const proto = tag === 'INPUT' ? win.HTMLInputElement.prototype : win.HTMLTextAreaElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(proto, 'value').set;
      setter.call(el, append ? el.value + text : text);
      el.dispatchEvent(new win.InputEvent('input', { bubbles: true, composed: true, inputType: 'insertText', data: text }));
      el.dispatchEvent(new win.Event('change', { bubbles: true }));
      return;
    }
    if (el.isContentEditable) {
      el.focus({ preventScroll: true });
      const d = el.ownerDocument;
      if (!append) d.execCommand('selectAll', false);
      d.execCommand('insertText', false, text);
      return;
    }
    throw new Error('element is not a text field');
  }

  const KEYS = {
    enter: ['Enter', 'Enter', 13], tab: ['Tab', 'Tab', 9], escape: ['Escape', 'Escape', 27], esc: ['Escape', 'Escape', 27],
    space: [' ', 'Space', 32], backspace: ['Backspace', 'Backspace', 8], delete: ['Delete', 'Delete', 46],
    arrowup: ['ArrowUp', 'ArrowUp', 38], arrowdown: ['ArrowDown', 'ArrowDown', 40], arrowleft: ['ArrowLeft', 'ArrowLeft', 37],
    arrowright: ['ArrowRight', 'ArrowRight', 39], home: ['Home', 'Home', 36], end: ['End', 'End', 35],
    pageup: ['PageUp', 'PageUp', 33], pagedown: ['PageDown', 'PageDown', 34],
  };

  function press(el, spec) {
    const parts = String(spec).split('+').map((p) => p.trim());
    const k = parts.pop();
    const mods = parts.map(lc);
    const known = KEYS[lc(k)];
    const key = known ? known[0] : k;
    const code = known ? known[1] : k.length === 1 ? 'Key' + k.toUpperCase() : k;
    const kc = known ? known[2] : k.length === 1 ? k.toUpperCase().charCodeAt(0) : 0;
    const win = el.ownerDocument.defaultView || window;
    const o = {
      key, code, keyCode: kc, which: kc, bubbles: true, cancelable: true, composed: true,
      ctrlKey: mods.includes('ctrl') || mods.includes('control'), shiftKey: mods.includes('shift'),
      altKey: mods.includes('alt'), metaKey: mods.includes('meta') || mods.includes('cmd'),
    };
    const down = new win.KeyboardEvent('keydown', o);
    el.dispatchEvent(down);
    if (!down.defaultPrevented && (key === 'Enter' || key.length === 1)) el.dispatchEvent(new win.KeyboardEvent('keypress', o));
    if (!down.defaultPrevented && key === 'Enter') {
      // a synthetic Enter does not submit forms or press buttons by itself
      const tag = el.tagName;
      if (tag === 'INPUT' && el.form && !['button', 'submit', 'reset', 'checkbox', 'radio'].includes(lc(el.type))) {
        if (el.form.requestSubmit) el.form.requestSubmit();
        else el.form.submit();
      } else if (tag === 'BUTTON' || tag === 'A' || lc(el.getAttribute('role')) === 'button') el.click();
    }
    el.dispatchEvent(new win.KeyboardEvent('keyup', o));
  }

  for (let i = 0; i < steps.length; i++) {
    const s = steps[i];
    const t = s.tool;
    try {
      if (t === 'click' || t === 'double_click' || t === 'right_click') {
        const r = resolve(s);
        if (!r) throw new Error('click needs a target or element_token');
        const note = covered(r.el) ? ' (note: another element covers it; the click was sent to the element itself)' : '';
        await click(r.el, t === 'double_click' ? 'double' : t === 'right_click' ? 'right' : 'single');
        if (i + 1 < steps.length) await sleep(30);
        log.push('ok: ' + t + ' ' + label(r) + note);
      } else if (t === 'hover') {
        const r = resolve(s);
        if (!r) throw new Error('hover needs a target');
        r.el.scrollIntoView({ block: 'center' });
        const b = r.el.getBoundingClientRect();
        moveOverlay(b.left + b.width / 2, b.top + b.height / 2, false);
        const o = { bubbles: true, composed: true, clientX: b.left + b.width / 2, clientY: b.top + b.height / 2 };
        r.el.dispatchEvent(new MouseEvent('mouseover', o));
        r.el.dispatchEvent(new MouseEvent('mousemove', o));
        log.push('ok: hover ' + label(r));
      } else if (t === 'fill' || t === 'type') {
        const text = s.text !== undefined ? String(s.text) : s.value !== undefined ? String(s.value) : null;
        if (text === null) throw new Error(t + ' needs text');
        const r = resolve(s);
        const el = r ? r.el : S && document.activeElement;
        if (!el || el === document.body) throw new Error(t + ' needs a target (or a focused field)');
        const b = el.getBoundingClientRect();
        if (b.width > 0) moveOverlay(b.left + b.width / 2, b.top + b.height / 2, false);
        setText(el, text, t === 'type');
        log.push('ok: ' + t + ' ' + (r ? label(r) : 'focused field') + ' (' + text.length + ' chars)');
      } else if (t === 'select') {
        const r = resolve(s);
        if (!r) throw new Error('select needs a target');
        const want = lc(String(s.value !== undefined ? s.value : s.text).trim());
        if (r.el.tagName === 'SELECT') {
          const opt = Array.from(r.el.options).find((o) => lc(o.text.trim()) === want || lc(o.value) === want) ||
            Array.from(r.el.options).find((o) => lc(o.text).includes(want));
          if (!opt) throw new Error('no option "' + want + '"; options: ' + Array.from(r.el.options).slice(0, 8).map((o) => o.text.trim()).join(' | '));
          const b = r.el.getBoundingClientRect();
          moveOverlay(b.left + b.width / 2, b.top + b.height / 2, true);
          r.el.value = opt.value;
          r.el.dispatchEvent(new Event('input', { bubbles: true }));
          r.el.dispatchEvent(new Event('change', { bubbles: true }));
          log.push('ok: select ' + label(r) + ' = "' + cut(opt.text.trim(), 40) + '"');
        } else {
          // Custom ARIA combobox/dropdown: click trigger, wait for options, click matching option
          await click(r.el, 'single');
          await sleep(80);
          const opts = Array.from(document.querySelectorAll('[role="option"], [role="menuitem"], [role="menuitemradio"], listbox option'));
          const opt = opts.find((o) => lc(o.textContent.trim()) === want) || opts.find((o) => lc(o.textContent).includes(want));
          if (!opt) throw new Error('no option "' + want + '" in custom dropdown');
          await click(opt, 'single');
          log.push('ok: select ' + label(r) + ' = "' + cut(opt.textContent.trim(), 40) + '"');
        }
      } else if (t === 'press') {
        if (!s.key) throw new Error('press needs key, e.g. "Enter"');
        const r = s.target || s.element_token || s.ref !== undefined ? resolve(s) : null;
        const el = r ? r.el : document.activeElement || document.body;
        press(el, s.key);
        if (i + 1 < steps.length) await sleep(25);
        log.push('ok: press ' + s.key + (r ? ' on ' + label(r) : ''));
      } else if (t === 'scroll') {
        const r = s.target || s.element_token || s.ref !== undefined ? resolve(s) : null;
        const dir = lc(s.direction || 'down');
        const amt = s.amount !== undefined ? Number(s.amount) : 600;
        if (r && !s.direction) {
          r.el.scrollIntoView({ block: 'center' });
          log.push('ok: scrolled to ' + label(r));
        } else {
          const box = r ? r.el : null;
          const dx = dir === 'left' ? -amt : dir === 'right' ? amt : 0;
          const dy = dir === 'up' ? -amt : dir === 'down' ? amt : 0;
          if (box) box.scrollBy(dx, dy);
          else window.scrollBy(dx, dy);
          log.push('ok: scroll ' + dir + ' ' + amt + 'px (scrollY=' + Math.round(window.scrollY) + ')');
        }
      } else if (t === 'wait') {
        if (s.text) {
          const limit = Math.min(Number(s.timeout || 8000), 30000);
          const t0 = Date.now();
          while (Date.now() - t0 < limit && !document.body.innerText.includes(String(s.text))) await sleep(100);
          if (!document.body.innerText.includes(String(s.text))) throw new Error('text "' + s.text + '" did not appear within ' + limit + ' ms');
          log.push('ok: waited for text "' + cut(String(s.text), 40) + '"');
        } else {
          const ms = Math.min(Number(s.ms || 500), 10000);
          await sleep(ms);
          log.push('ok: waited ' + ms + ' ms');
        }
      } else {
        throw new Error('unknown step tool ' + t);
      }
    } catch (e) {
      log.push('FAILED ' + t + ': ' + (e && e.message ? e.message : e));
      return { log, failed: true, done: i, retryAfterRead: !!(e && e.retryAfterRead && (i > 0 || !S)) };
    }
  }
  return { log, failed: false, done: steps.length };
}
