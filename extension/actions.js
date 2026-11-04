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

  function byName(target) {
    const meta = S.meta;
    let role = null;
    let want = target;
    const c = target.indexOf(':');
    if (c > 0 && meta.some((m) => lc(m.role) === lc(target.slice(0, c)))) {
      role = lc(target.slice(0, c));
      want = target.slice(c + 1);
    }
    const w = lc(want.trim());
    const all = meta.map((_, i) => i).filter((i) => !role || lc(meta[i].role) === role);
    let hits = all.filter((i) => lc(meta[i].name) === w || lc(meta[i].id) === w);
    if (!hits.length) hits = all.filter((i) => lc(meta[i].name).includes(w) || lc(meta[i].id).includes(w));
    if (hits.length === 1) return hits[0];
    if (!hits.length) throw new Error("no element matches target '" + target + "'");
    throw new Error("target '" + target + "' is ambiguous: " + hits.slice(0, 6).map((i) => '[' + i + '] ' + meta[i].role + ' "' + meta[i].name + '"').join(', ') + '. Use Role:Name or an element_token.');
  }

  function resolve(step) {
    if (!S) throw new Error('this page has not been read yet (or it reloaded): read it again first');
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
    if (!el || !el.isConnected) throw new Error('element [' + idx + '] is no longer on the page; read it again');
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
        log.push('ok: ' + t + ' ' + label(r) + note);
      } else if (t === 'hover') {
        const r = resolve(s);
        if (!r) throw new Error('hover needs a target');
        r.el.scrollIntoView({ block: 'center' });
        const b = r.el.getBoundingClientRect();
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
        setText(el, text, t === 'type');
        log.push('ok: ' + t + ' ' + (r ? label(r) : 'focused field') + ' (' + text.length + ' chars)');
      } else if (t === 'select') {
        const r = resolve(s);
        if (!r || r.el.tagName !== 'SELECT') throw new Error('select needs a target that is a dropdown (Combobox)');
        const want = lc(String(s.value !== undefined ? s.value : s.text));
        const opt = Array.from(r.el.options).find((o) => lc(o.text.trim()) === want || lc(o.value) === want) ||
          Array.from(r.el.options).find((o) => lc(o.text).includes(want));
        if (!opt) throw new Error('no option "' + want + '"; options: ' + Array.from(r.el.options).slice(0, 8).map((o) => o.text.trim()).join(' | '));
        r.el.value = opt.value;
        r.el.dispatchEvent(new Event('input', { bubbles: true }));
        r.el.dispatchEvent(new Event('change', { bubbles: true }));
        log.push('ok: select ' + label(r) + ' = "' + cut(opt.text.trim(), 40) + '"');
      } else if (t === 'press') {
        if (!s.key) throw new Error('press needs key, e.g. "Enter"');
        const r = s.target || s.element_token || s.ref !== undefined ? resolve(s) : null;
        const el = r ? r.el : document.activeElement || document.body;
        press(el, s.key);
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
      return { log, failed: true, done: i };
    }
  }
  return { log, failed: false, done: steps.length };
}
