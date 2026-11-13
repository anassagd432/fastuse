// Reads a page into the compact text tree fastuse uses everywhere:
//   - [N] Role "name" value="..." id=x @(left,top,w,h)      ([N] and @ only for things the agent can act on)
// The elements and their names stay in this frame (globalThis.__fastuse) so actions.js can resolve [N].
// readPage is injected with chrome.scripting.executeScript({func: readPage}), so it must be
// completely self-contained: it may not use anything defined outside its own body.

export function readPage(opts) {
  const t0 = performance.now();
  const max = Math.max(10, Math.min(opts.max || 45, 1000));
  const scanCap = Math.min(Math.max(max * 10, 600), 2500);
  const withText = !!opts.text;
  const refs = [];
  const meta = [];
  const rows = [];
  const items = [];
  let truncated = false;
  let interactive = 0;

  const vw = window.innerWidth || 1280;
  const vh = window.innerHeight || 800;
  const SKIP = new Set(['SCRIPT', 'STYLE', 'NOSCRIPT', 'TEMPLATE', 'HEAD', 'META', 'LINK', 'BR', 'WBR']);
  const ACTION_ROLES = new Set([
    'button', 'link', 'checkbox', 'radio', 'tab', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'option',
    'switch', 'combobox', 'textbox', 'searchbox', 'slider', 'spinbutton', 'treeitem',
  ]);
  const ROLE_LABEL = {
    button: 'Button', link: 'Link', checkbox: 'Checkbox', radio: 'Radio', tab: 'Tab', menuitem: 'MenuItem',
    menuitemcheckbox: 'MenuItem', menuitemradio: 'MenuItem', option: 'Option', switch: 'Switch', combobox: 'Combobox',
    textbox: 'Textbox', searchbox: 'Textbox', slider: 'Slider', spinbutton: 'Spinbutton', treeitem: 'TreeItem',
  };
  const LANDMARK = {
    MAIN: 'Main', NAV: 'Navigation', HEADER: 'Header', FOOTER: 'Footer', ASIDE: 'Aside', FORM: 'Form',
    DIALOG: 'Dialog', SECTION: '', ARTICLE: 'Article',
  };
  const TEXT_TAGS = new Set(['P', 'LI', 'TD', 'TH', 'DT', 'DD', 'FIGCAPTION', 'BLOCKQUOTE', 'PRE', 'SUMMARY', 'LABEL']);
  const txt = (s) => (s || '').replace(/\s+/g, ' ').trim();
  const cut = (s, n) => (s.length > n ? s.slice(0, n - 1) + '…' : s);
  const rectInView = (r) => r.bottom >= -20 && r.top <= vh + 20 && r.right >= 0 && r.left <= vw;

  function visible(el) {
    try {
      if (!el.checkVisibility || el.checkVisibility({ checkVisibilityCSS: true })) return true;
      const cs = getComputedStyle(el);
      return cs.display === 'contents' && cs.visibility !== 'hidden' && cs.opacity !== '0';
    } catch (e) {
      return true;
    }
  }

  function roleOf(el) {
    const r = (el.getAttribute('role') || '').split(/\s+/)[0].toLowerCase();
    if (r && ACTION_ROLES.has(r)) return r;
    const t = el.tagName;
    if (t === 'A') return el.hasAttribute('href') ? 'link' : (el.hasAttribute('onclick') ? 'button' : '');
    if (t === 'BUTTON' || t === 'SUMMARY') return 'button';
    if (t === 'SELECT') return 'combobox';
    if (t === 'TEXTAREA') return 'textbox';
    if (t === 'INPUT') {
      const ty = (el.type || 'text').toLowerCase();
      if (ty === 'hidden') return '';
      if (ty === 'checkbox') return 'checkbox';
      if (ty === 'radio') return 'radio';
      if (ty === 'range') return 'slider';
      if (ty === 'number') return 'spinbutton';
      if (['button', 'submit', 'reset', 'image', 'file', 'color'].includes(ty)) return 'button';
      return 'textbox';
    }
    if (el.isContentEditable && el.parentElement && !el.parentElement.isContentEditable) return 'textbox';
    if (el.hasAttribute('onclick')) return 'button';
    return '';
  }

  function nameOf(el, role) {
    let n = el.getAttribute('aria-label');
    if (!n) {
      const ids = el.getAttribute('aria-labelledby');
      if (ids) {
        n = ids.split(/\s+/).map((i) => {
          const e = el.getRootNode().getElementById ? el.getRootNode().getElementById(i) : null;
          return e ? e.textContent : '';
        }).join(' ');
      }
    }
    if (!n && el.labels && el.labels.length) n = Array.from(el.labels).map((l) => l.textContent).join(' ');
    const tag = el.tagName;
    if (!n && tag === 'INPUT') {
      const ty = (el.type || '').toLowerCase();
      if (['button', 'submit', 'reset'].includes(ty)) n = el.value;
      else if (ty === 'image') n = el.alt;
    }
    if (!n && (role === 'textbox' || role === 'combobox' || role === 'searchbox')) {
      n = el.getAttribute('placeholder') || el.getAttribute('aria-placeholder');
      if (!n && el.previousElementSibling && el.previousElementSibling.tagName === 'LABEL') {
        n = el.previousElementSibling.textContent;
      }
    }
    if (!n) n = el.getAttribute('title');
    if (!n && tag !== 'INPUT' && tag !== 'SELECT' && tag !== 'TEXTAREA') {
      n = el.textContent;
      if (!txt(n)) {
        const img = el.querySelector('img[alt]');
        if (img) n = img.getAttribute('alt');
        else {
          const t = el.querySelector('svg title');
          if (t) n = t.textContent;
        }
      }
    }
    n = txt(n);
    if (!n) {
      // last resorts so a model never gets a nameless element
      if (tag === 'A') {
        try { n = new URL(el.href, location.href).pathname.split('/').filter(Boolean).pop() || ''; } catch (e) { /* ignore */ }
      }
      if (!n) n = el.getAttribute('name') || el.id || '';
    }
    return cut(n, 80);
  }

  function valueOf(el, role) {
    const t = el.tagName;
    if (t === 'INPUT') {
      const ty = (el.type || 'text').toLowerCase();
      if (ty === 'password') return el.value ? '(hidden)' : '';
      if (ty === 'checkbox' || ty === 'radio') return el.checked ? 'checked' : '';
      if (['button', 'submit', 'reset', 'image'].includes(ty)) return '';
      return cut(el.value || '', 100);
    }
    if (t === 'TEXTAREA') return cut(el.value || '', 100);
    if (t === 'SELECT') return cut(Array.from(el.selectedOptions || []).map((o) => o.text).join(', '), 100);
    if (role === 'checkbox' || role === 'radio' || role === 'switch' || role === 'tab' || role === 'option') {
      const c = el.getAttribute('aria-checked') || el.getAttribute('aria-selected');
      return c === 'true' ? 'checked' : '';
    }
    if (el.isContentEditable) return cut(txt(el.textContent), 100);
    return '';
  }

  function walk(node, depth, ox, oy) {
    for (let el = node.firstElementChild; el; el = el.nextElementSibling) {
      if (items.length >= scanCap) {
        truncated = true;
        return;
      }
      visit(el, depth, ox, oy);
    }
  }

  function visit(el, depth, ox, oy) {
    const tag = el.tagName;
    if (SKIP.has(tag) || el.hasAttribute('hidden') || el.getAttribute('aria-hidden') === 'true') return;
    if (!visible(el)) return;

    const role = roleOf(el);
    if (role) {
      const r = el.getBoundingClientRect();
      if (r.width > 0 && r.height > 0) {
        const name = nameOf(el, role);
        const value = valueOf(el, role);
        const exp = el.getAttribute('aria-expanded');
        const flagList = [];
        if (el.disabled || el.getAttribute('aria-disabled') === 'true') flagList.push('disabled');
        if (exp === 'true') flagList.push('expanded');
        else if (exp === 'false') flagList.push('collapsed');
        const flags = flagList.join(' ');
        const id = el.id || el.getAttribute('data-testid') || el.getAttribute('name') || '';
        items.push({
          act: true,
          el,
          inView: rectInView({ left: r.left + ox, top: r.top + oy, right: r.right + ox, bottom: r.bottom + oy }),
          depth,
          role: ROLE_LABEL[role] || role,
          name,
          value,
          id: id !== name ? id : '',
          rect: [Math.round(r.left + ox), Math.round(r.top + oy), Math.round(r.width), Math.round(r.height)],
          flags,
        });
      }
      if (tag === 'A' || tag === 'BUTTON' || tag === 'SELECT' || tag === 'TEXTAREA' || tag === 'INPUT') return;
    }

    let d = depth;
    if (/^H[1-6]$/.test(tag)) {
      const t = cut(txt(el.textContent), 100);
      if (t) {
        const r = el.getBoundingClientRect();
        items.push({ act: false, inView: rectInView(r), depth, role: 'Heading[' + tag[1] + ']', name: t, value: '', id: '', rect: null, flags: '' });
      }
    } else if (LANDMARK[tag] !== undefined || el.getAttribute('role') === 'dialog') {
      const label = LANDMARK[tag] || (el.getAttribute('role') === 'dialog' ? 'Dialog' : '');
      const aria = el.getAttribute('aria-label') || '';
      if (label && (tag !== 'SECTION' || aria)) {
        const r = el.getBoundingClientRect();
        items.push({ act: false, inView: rectInView(r), depth, role: label, name: cut(txt(aria), 60), value: '', id: '', rect: null, flags: '' });
        d = depth + 1;
      }
    } else if (withText && TEXT_TAGS.has(tag)) {
      const t = cut(txt(el.textContent), 120);
      if (t) {
        const r = el.getBoundingClientRect();
        items.push({ act: false, inView: rectInView(r), depth, role: 'Text', name: t, value: '', id: '', rect: null, flags: '' });
      }
    } else if (tag === 'IMG' && withText) {
      const a = txt(el.getAttribute('alt'));
      if (a) {
        const r = el.getBoundingClientRect();
        items.push({ act: false, inView: rectInView(r), depth, role: 'Image', name: cut(a, 80), value: '', id: '', rect: null, flags: '' });
      }
    }

    if (tag === 'IFRAME' || tag === 'FRAME') {
      let doc = null;
      try { doc = el.contentDocument; } catch (e) { doc = null; }
      if (doc && doc.body) {
        const fr = el.getBoundingClientRect();
        items.push({ act: false, inView: rectInView(fr), depth: d, role: 'Frame', name: cut(el.title || el.name || '', 60), value: '', id: '', rect: null, flags: '' });
        walk(doc.body, d + 1, ox + fr.left, oy + fr.top);
      } else {
        let host = '';
        try { host = new URL(el.src, location.href).host; } catch (e) { /* ignore */ }
        items.push({ act: false, inView: true, depth: d, role: 'Frame', name: cut(el.title || host, 60), value: '', id: '', rect: null, flags: 'cross-origin' });
      }
      return;
    }
    if (el.shadowRoot) walk(el.shadowRoot, d, ox, oy);
    walk(el, d, ox, oy);
  }

  walk(document.documentElement, 0, 0, 0);

  let selected = items;
  if (items.length > max) {
    truncated = true;
    const inViewItems = items.filter((x) => x.inView);
    const offViewItems = items.filter((x) => !x.inView);
    selected = inViewItems.concat(offViewItems).slice(0, max);
  }

  for (const it of selected) {
    let idx = null;
    if (it.act) {
      idx = refs.length;
      refs.push(it.el);
      meta.push({ role: it.role, name: it.name, id: it.id || it.name });
      interactive++;
    }
    let s = '  '.repeat(it.depth) + '- ';
    if (idx !== null) s += '[' + idx + '] ';
    s += it.role;
    if (it.name) s += ' "' + it.name.replace(/"/g, "'") + '"';
    if (it.value) s += ' value="' + it.value.replace(/"/g, "'") + '"';
    if (it.flags) s += ' ' + it.flags;
    if (it.id) s += ' id=' + it.id;
    if (it.rect) s += ' @(' + it.rect.join(',') + ')';
    rows.push(s);
  }

  // the isolated world of this frame keeps the elements so later calls can act on [N]
  globalThis.__fastuse = { snap: opts.snap || '', refs, meta };

  const de = document.documentElement;
  return {
    url: location.href,
    title: document.title,
    viewport: { w: vw, h: vh, scrollX: Math.round(window.scrollX), scrollY: Math.round(window.scrollY), pageH: de.scrollHeight },
    count: interactive,
    rows: rows.length,
    truncated,
    text: rows.join('\n'),
    ms: Math.round((performance.now() - t0) * 10) / 10,
  };
}
