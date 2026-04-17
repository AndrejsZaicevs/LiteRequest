import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// macOS WKWebView converts typed " to smart quotes ("") in text inputs.
// CodeMirror search panels are injected dynamically, so we patch them via MutationObserver.
function patchCmSearchInputs(root: Element) {
  root.querySelectorAll<HTMLInputElement>('.cm-search input[type="text"]').forEach(el => {
    el.setAttribute("autocorrect", "off");
    el.setAttribute("autocapitalize", "off");
    el.setAttribute("spellcheck", "false");
  });
}

// Collapse the replace row in CodeMirror search panels behind a toggle arrow.
function patchCmSearchPanel(root: Element) {
  root.querySelectorAll<HTMLElement>('.cm-search').forEach(panel => {
    if (panel.dataset.patched) return;
    panel.dataset.patched = "1";

    // Find the <br> that separates find row from replace row
    const br = Array.from(panel.childNodes).find(n => n.nodeName === "BR");
    if (!br) return;

    // Collect replace row nodes: everything after <br> except button[name="close"]
    const closeBtn = panel.querySelector<HTMLButtonElement>('button[name="close"]');
    const afterBr: ChildNode[] = [];
    let node: ChildNode | null = br.nextSibling;
    while (node) {
      if (node !== closeBtn) afterBr.push(node);
      node = node.nextSibling;
    }

    // Wrap replace row in a collapsible div
    const replaceRow = document.createElement("div");
    replaceRow.className = "cm-replace-row";
    replaceRow.style.cssText = "display:none;align-items:center;gap:4px;width:100%;flex-wrap:wrap;";
    afterBr.forEach(n => replaceRow.appendChild(n));

    // Toggle button (▸ / ▾)
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.textContent = "▸";
    toggle.title = "Toggle replace";
    toggle.className = "cm-replace-toggle";
    toggle.style.cssText = [
      "background:transparent", "border:none", "color:#6b7280",
      "font-size:10px", "padding:2px 4px", "cursor:pointer",
      "border-radius:3px", "line-height:1", "transition:color 0.15s",
    ].join(";");
    toggle.onmouseenter = () => { toggle.style.color = "#e5e7eb"; };
    toggle.onmouseleave = () => { toggle.style.color = "#6b7280"; };
    toggle.onclick = () => {
      const open = replaceRow.style.display !== "none";
      replaceRow.style.display = open ? "none" : "flex";
      toggle.textContent = open ? "▸" : "▾";
      if (!open) {
        const inp = replaceRow.querySelector<HTMLInputElement>('input[name="replace"]');
        inp?.focus();
      }
    };

    // Remove the <br>, insert toggle at start, append replace row at end
    br.remove();
    panel.insertBefore(toggle, panel.firstChild);
    panel.appendChild(replaceRow);
    if (closeBtn) panel.appendChild(closeBtn);
  });
}

new MutationObserver(mutations => {
  for (const m of mutations) {
    for (const node of m.addedNodes) {
      if (node instanceof Element) {
        patchCmSearchInputs(node);
        patchCmSearchPanel(node);
      }
    }
  }
}).observe(document.documentElement, { subtree: true, childList: true });

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
