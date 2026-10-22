// Tint verdict cells in the generated grids (exact, close, OFF, gaps, --;
// kramdown typesets "--" as an en dash).
document.querySelectorAll(".markdown-body td").forEach(function (td) {
  var t = td.textContent.trim();
  var v = /^exact/.test(t) ? "exact" : /^close/.test(t) ? "close" : /^OFF/.test(t) ? "off"
        : /^gaps/.test(t) ? "gaps" : /^(--|\u2013)/.test(t) ? "none" : null;
  if (v) td.classList.add("v-" + v);
});
