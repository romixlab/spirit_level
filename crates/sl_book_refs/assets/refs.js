// spl book refs: keeps an open card (spl-refs.css) inside the window, written by `spl book refs` (do not edit). A card
// opens below its reference, left-aligned with it; one that would run past the window's right edge is aligned with
// that edge instead. One listener on the document, so pages that instant navigation loads later need nothing.
(function () {
  function place(ref) {
    var card = ref.querySelector(":scope > .spl-card");
    if (!card) return;
    card.style.left = "0";
    var box = card.getBoundingClientRect();
    var room = document.documentElement.clientWidth - 8;
    if (box.right > room) {
      card.style.left = Math.round(room - box.right) + "px";
    }
  }
  function on(event) {
    var ref = event.target.closest && event.target.closest(".spl-ref");
    if (ref) place(ref);
  }
  document.addEventListener("mouseover", on);
  document.addEventListener("focusin", on);
})();
