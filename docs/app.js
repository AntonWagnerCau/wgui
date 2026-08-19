// Docs page behaviour: Rust syntax highlighting, copy buttons, scrollspy.

(function () {
  'use strict';

  // ── Syntax highlighting ────────────────────────────────────────────

  var KEYWORDS = new Set([
    'let', 'mut', 'fn', 'pub', 'struct', 'impl', 'enum', 'trait', 'for', 'in',
    'if', 'else', 'while', 'loop', 'match', 'use', 'as', 'return', 'move',
    'ref', 'const', 'static', 'true', 'false', 'self', 'crate', 'mod', 'where',
    'dyn', 'type', 'unsafe', 'async', 'await', 'break', 'continue'
  ]);

  var TOKEN = new RegExp([
    '(//[^\\n]*)',                                   // 1 line comment
    '("(?:\\\\.|[^"\\\\])*")',                       // 2 string
    '(#!?\\[[^\\]]*\\])',                            // 3 attribute
    '([A-Za-z_][A-Za-z0-9_]*!)',                     // 4 macro
    '\\b(\\d[\\d_]*(?:\\.\\d+)?(?:f32|f64|u\\d+|i\\d+|usize|isize)?)\\b', // 5 number
    '\\b([A-Z][A-Za-z0-9_]*)\\b',                    // 6 type
    '\\b([a-z_][a-z0-9_]*)\\b(\\s*\\()',             // 7+8 fn call
    '\\b([a-z_][a-z0-9_]*)\\b'                       // 9 word
  ].join('|'), 'g');

  function esc(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  function span(cls, text) {
    return '<span class="tok-' + cls + '">' + esc(text) + '</span>';
  }

  function highlight(src) {
    var out = '';
    var last = 0;
    var m;
    TOKEN.lastIndex = 0;
    while ((m = TOKEN.exec(src)) !== null) {
      out += esc(src.slice(last, m.index));
      last = m.index + m[0].length;
      if (m[1]) out += span('cmt', m[1]);
      else if (m[2]) out += span('str', m[2]);
      else if (m[3]) out += span('attr', m[3]);
      else if (m[4]) out += span('mac', m[4]);
      else if (m[5]) out += span('num', m[5]);
      else if (m[6]) out += span('ty', m[6]);
      else if (m[7]) out += span('fn', m[7]) + esc(m[8]);
      else if (m[9]) out += KEYWORDS.has(m[9]) ? span('kw', m[9]) : esc(m[9]);
    }
    return out + esc(src.slice(last));
  }

  document.querySelectorAll('pre > code').forEach(function (code) {
    if (code.classList.contains('plain')) return;
    code.innerHTML = highlight(code.textContent);
  });

  // ── Copy buttons ───────────────────────────────────────────────────

  document.querySelectorAll('pre').forEach(function (pre) {
    var code = pre.querySelector('code');
    if (!code) return;
    var btn = document.createElement('button');
    btn.className = 'copy';
    btn.type = 'button';
    btn.textContent = 'copy';
    btn.addEventListener('click', function () {
      navigator.clipboard.writeText(code.textContent).then(function () {
        btn.textContent = 'copied';
        btn.classList.add('done');
        setTimeout(function () {
          btn.textContent = 'copy';
          btn.classList.remove('done');
        }, 1200);
      });
    });
    pre.appendChild(btn);
  });

  // ── Sidebar scrollspy ──────────────────────────────────────────────

  var links = Array.prototype.slice.call(
    document.querySelectorAll('#sidebar a[href^="#"]')
  );
  if (!links.length) return;

  var targets = links
    .map(function (a) {
      return { link: a, el: document.getElementById(a.hash.slice(1)) };
    })
    .filter(function (t) {
      return t.el;
    });

  function sync() {
    var line = window.scrollY + 120;
    var current = targets[0];
    targets.forEach(function (t) {
      if (t.el.offsetTop <= line) current = t;
    });
    links.forEach(function (a) {
      a.classList.toggle('active', current && a === current.link);
    });
  }

  var ticking = false;
  window.addEventListener('scroll', function () {
    if (ticking) return;
    ticking = true;
    requestAnimationFrame(function () {
      sync();
      ticking = false;
    });
  });
  sync();
})();
