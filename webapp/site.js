// The site's motion and small interactions. Vanilla, no dependencies, and
// never imported by the editor.
//
// Copied from OpenCalc's `webapp/site.js` at `b23c67c` (same owner,
// Apache-2.0; webapp/README.md). OpenPresent's pages have no hero story, so
// `story` finds nothing to run; it is kept so the two files stay close. Added
// by `SITE-03`: `demo`, the hero's strip of real renders, and `sample`, which
// opens the bundled sample deck in the editor.
//
// - With prefers-reduced-motion, or without IntersectionObserver, nothing here
//   animates: `html.motion` is never set, every reveal sits in its final state,
//   and the hero story keeps the finished markup it ships with.
// - Otherwise: reveals on scroll, counters count up, illustrations loop only
//   while on screen, and the hero story plays while visible, with a Pause
//   button, stopping in a hidden tab.
// - In both modes: the mobile menu closes on Escape and on navigation, and the
//   docs table of contents marks the section in view.
(function () {
  "use strict";
  var root = document.documentElement;
  var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  var IO = "IntersectionObserver" in window;
  var motion = !reduce && IO;

  function all(sel, ctx) { return Array.prototype.slice.call((ctx || document).querySelectorAll(sel)); }
  function inView(el) { var r = el.getBoundingClientRect(); return r.top < innerHeight && r.bottom > 0; }

  // ---------------------------------------------------------------- counters
  function count(el) {
    var to = parseInt(el.getAttribute("data-count"), 10), t0 = null, dur = 1200;
    if (!to) return;
    var fmt = function (n) { return n.toLocaleString("en-US"); };
    el.textContent = "0";
    function tick(t) {
      if (t0 === null) t0 = t;
      var p = Math.min((t - t0) / dur, 1), e = 1 - Math.pow(1 - p, 3);
      el.textContent = fmt(Math.round(to * e));
      if (p < 1) requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  }

  // ---------------------------------------------------------------- reveals
  function reveals() {
    var once = all(".rv, .seq, .pipe, .phases, .il-wrap, .bigmark, [data-count]");
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (!e.isIntersecting) return;
        e.target.classList.add("in");
        if (e.target.hasAttribute("data-count")) count(e.target);
        io.unobserve(e.target);
      });
    }, { rootMargin: "0px 0px -8% 0px", threshold: 0.12 });
    once.forEach(function (el) {
      if (inView(el) && !el.hasAttribute("data-count")) el.classList.add("in");
      else io.observe(el);
    });

    // Loops run only while their drawing is on screen.
    var live = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) { e.target.classList.toggle("live", e.isIntersecting && !document.hidden); });
    }, { threshold: 0.2 });
    all(".il-wrap").forEach(function (el) { live.observe(el); });
    document.addEventListener("visibilitychange", function () {
      if (document.hidden) all(".il-wrap.live").forEach(function (el) { el.classList.remove("live"); });
      else all(".il-wrap.in").forEach(function (el) { if (inView(el)) el.classList.add("live"); });
    });

    // Last, so the elements already on screen were marked first.
    root.classList.add("motion");
  }

  // ---------------------------------------------------------------- hero story
  function story(el) {
    var DUR = [2600, 3000, 3000, 3600, 3200];
    var LAST = DUR.length - 1;
    var btn = el.querySelector(".story-toggle");
    var label = btn && btn.querySelector("span");
    var step = -1, timer = null, paused = false, visible = false;

    function setStep(n) {
      step = n;
      el.setAttribute("data-step", String(n));
      for (var k = 0; k <= LAST; k++) el.classList.toggle("s" + k, k <= n);
    }
    function reset() {
      el.classList.add("reset");
      setStep(-1);
      void el.offsetWidth;
      el.classList.remove("reset");
    }
    function next() {
      var n = step + 1;
      if (n > LAST) { reset(); n = 0; }
      setStep(n);
      timer = setTimeout(next, DUR[n]);
    }
    function sync() {
      var run = visible && !paused && !document.hidden;
      if (run && !timer) timer = setTimeout(next, step < 0 ? 350 : 1200);
      if (!run && timer) { clearTimeout(timer); timer = null; }
    }

    reset();
    if (btn) {
      btn.hidden = false;
      btn.addEventListener("click", function () {
        paused = !paused;
        btn.setAttribute("aria-pressed", paused ? "true" : "false");
        if (label) label.textContent = paused ? "Play" : "Pause";
        btn.classList.toggle("is-paused", paused);
        sync();
      });
    }
    new IntersectionObserver(function (es) { visible = es[0].isIntersecting; sync(); }, { threshold: 0.25 }).observe(el);
    document.addEventListener("visibilitychange", sync);
  }

  // ---------------------------------------------------------------- demo strip
  // OpenPresent's own (SITE-03), not OpenCalc's. The hero's window of real
  // renders: the thumbnails always pick a slide; with motion allowed it also
  // plays, a slide every DWELL ms, only while on screen and in a visible tab,
  // with a Pause button (WCAG 2.2.2).
  var DWELL = 4200;
  function demo(el) {
    var slides = all(".demo-slide", el), thumbs = all(".demo-thumb", el);
    var title = el.querySelector("[data-demo-title]"), counter = el.querySelector("[data-demo-count]");
    var btn = el.querySelector(".demo-toggle"), label = btn && btn.querySelector("span");
    var cur = 0, timer = null, paused = false, visible = false;
    if (slides.length < 2 || slides.length !== thumbs.length) return;

    function show(n) {
      cur = (n + slides.length) % slides.length;
      slides.forEach(function (s, i) { s.classList.toggle("is-on", i === cur); });
      thumbs.forEach(function (t, i) {
        t.classList.toggle("is-on", i === cur);
        t.setAttribute("aria-pressed", i === cur ? "true" : "false");
      });
      if (title) title.textContent = thumbs[cur].getAttribute("data-name") || "";
      if (counter) counter.textContent = "Slide " + (cur + 1) + " of " + slides.length;
      // Restart the progress bar from empty.
      el.classList.remove("playing");
      void el.offsetWidth;
      if (timer) el.classList.add("playing");
    }
    function tick() { timer = setTimeout(tick, DWELL); show(cur + 1); }
    function sync() {
      var run = motion && visible && !paused && !document.hidden;
      if (run && !timer) { timer = setTimeout(tick, DWELL); el.classList.add("playing"); }
      if (!run && timer) { clearTimeout(timer); timer = null; el.classList.remove("playing"); }
    }

    thumbs.forEach(function (t, i) {
      t.addEventListener("click", function () {
        if (timer) { clearTimeout(timer); timer = setTimeout(tick, DWELL); }
        show(i);
      });
    });
    if (!motion) return;
    if (btn) {
      btn.hidden = false;
      btn.addEventListener("click", function () {
        paused = !paused;
        btn.setAttribute("aria-pressed", paused ? "true" : "false");
        if (label) label.textContent = paused ? "Play" : "Pause";
        sync();
      });
    }
    new IntersectionObserver(function (es) { visible = es[0].isIntersecting; sync(); }, { threshold: 0.3 }).observe(el);
    document.addEventListener("visibilitychange", sync);
  }

  // ---------------------------------------------------------------- sample deck
  // "Try a sample deck": opens the editor in a new tab and hands it the bundled
  // sample through the hook the editor already exposes for the desktop shell,
  // `openpresentEditor.openDeck(name, bytes)` (webapp/README.md). The editor
  // fetches the file itself, from this same origin. Without script, or when the
  // tab is refused, the link downloads the deck instead, and the note says so.
  function sample(a) {
    var note = document.querySelector("[data-sample-status]");
    function say(text) { if (note) { note.textContent = text; note.hidden = false; } }
    a.addEventListener("click", function (e) {
      var src = new URL(a.getAttribute("data-sample"), location.href).href;
      var name = a.getAttribute("data-sample-name") || "Sample deck.pptx";
      var w = window.open(new URL("./editor.html", location.href).href, "_blank");
      if (!w) { say("The new tab was blocked, so the sample deck was downloaded. Open it in the editor with File > Open."); return; }
      e.preventDefault();
      say("Opening the sample deck in the editor, in a new tab.");
      var tries = 0;
      (function wait() {
        var ed = null;
        try { if (w.closed) return; ed = w.openpresentEditor; } catch (err) { return; }
        if (!ed) { if (++tries < 200) setTimeout(wait, 150); else say("The editor did not start in time. Download the sample and open it with File > Open."); return; }
        w.fetch(src).then(function (r) {
          if (!r.ok) throw new Error(String(r.status));
          return r.arrayBuffer();
        }).then(function (buf) {
          ed.openDeck(name, buf);
          say("The sample deck is open in the editor's tab.");
        }).catch(function () { say("The sample deck could not be fetched. Download it and open it with File > Open."); });
      })();
    });
  }

  // ---------------------------------------------------------------- mobile menu
  function menu(box) {
    function close() { box.open = false; }
    box.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && box.open) { close(); box.querySelector("summary").focus(); }
    });
    all("a", box).forEach(function (a) { a.addEventListener("click", close); });
    document.addEventListener("click", function (e) { if (box.open && !box.contains(e.target)) close(); });
  }

  // ---------------------------------------------------------------- docs TOC
  function toc(nav) {
    var links = all('a[href^="#"]', nav), queued = false;
    var targets = links.map(function (a) { return document.getElementById(a.getAttribute("href").slice(1)); });
    function update() {
      queued = false;
      var cur = -1;
      targets.forEach(function (t, i) { if (t && t.getBoundingClientRect().top < innerHeight * 0.3) cur = i; });
      links.forEach(function (a, i) {
        if (i === cur) a.setAttribute("aria-current", "true"); else a.removeAttribute("aria-current");
      });
    }
    addEventListener("scroll", function () { if (!queued) { queued = true; requestAnimationFrame(update); } }, { passive: true });
    update();
  }

  function init() {
    if (motion) {
      reveals();
      all("[data-story]").forEach(story);
    }
    all("[data-demo]").forEach(demo);
    all("a[data-sample]").forEach(sample);
    all("details.menu").forEach(menu);
    all("nav.toc").forEach(toc);
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init); else init();
})();
