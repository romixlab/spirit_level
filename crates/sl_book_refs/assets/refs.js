// spl book refs: opens a reference's card (spl-refs.css) beside it, written by `spl book refs` (do not edit). A
// reference is a link naming its card in `data-spl-card`; without this script it stays a plain link. A card opens on
// hover, on keyboard focus and on a first tap (a second tap follows the link), and closes on Escape, on a tap
// elsewhere and when the pointer or focus leaves both reference and card. Listeners sit on the document, so pages
// that instant navigation loads later need nothing.
(function () {
  var open = null;
  var timer = null;
  var touch = false;

  function refOf(node) {
    return node && node.closest ? node.closest("a.spl-ref[data-spl-card]") : null;
  }
  function cardOf(ref) {
    return document.getElementById(ref.getAttribute("data-spl-card"));
  }
  function place(ref, card) {
    var r = ref.getBoundingClientRect();
    var room = document.documentElement.clientWidth;
    card.style.left = "0px";
    card.style.top = "0px";
    var w = card.offsetWidth;
    var h = card.offsetHeight;
    var left = Math.max(8, Math.min(r.left, room - w - 8));
    var top = r.bottom + 4;
    if (top + h > window.innerHeight - 8 && r.top - h - 4 > 8) top = r.top - h - 4;
    card.style.left = Math.round(left) + "px";
    card.style.top = Math.round(top) + "px";
  }
  function show(ref) {
    var card = cardOf(ref);
    if (!card) return;
    clearTimeout(timer);
    if (open && open.card !== card) hide();
    card.classList.add("spl-open");
    ref.setAttribute("aria-expanded", "true");
    place(ref, card);
    open = { ref: ref, card: card };
  }
  function hide() {
    clearTimeout(timer);
    if (!open) return;
    open.card.classList.remove("spl-open");
    open.ref.setAttribute("aria-expanded", "false");
    open = null;
  }
  function later() {
    clearTimeout(timer);
    timer = setTimeout(hide, 300);
  }
  function inOpen(node) {
    return open && node && (open.ref.contains(node) || open.card.contains(node));
  }

  document.addEventListener("pointerdown", function (e) {
    touch = e.pointerType === "touch" || e.pointerType === "pen";
    if (open && !inOpen(e.target)) hide();
  });
  document.addEventListener("click", function (e) {
    var ref = refOf(e.target);
    if (!ref || !touch) return;
    if (!open || open.ref !== ref) {
      e.preventDefault();
      show(ref);
    }
  });
  document.addEventListener("mouseover", function (e) {
    if (touch) return;
    var ref = refOf(e.target);
    if (ref) show(ref);
    else if (inOpen(e.target)) clearTimeout(timer);
  });
  document.addEventListener("mouseout", function (e) {
    if (!touch && inOpen(e.target) && !inOpen(e.relatedTarget)) later();
  });
  document.addEventListener("focusin", function (e) {
    var ref = refOf(e.target);
    if (ref) show(ref);
    else if (inOpen(e.target)) clearTimeout(timer);
  });
  document.addEventListener("focusout", function (e) {
    if (inOpen(e.target) && !inOpen(e.relatedTarget)) later();
  });
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && open) hide();
  });
  window.addEventListener("scroll", function () {
    if (open) place(open.ref, open.card);
  }, { passive: true });
  window.addEventListener("resize", function () {
    if (open) place(open.ref, open.card);
  });
})();
