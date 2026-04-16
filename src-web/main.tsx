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
new MutationObserver(mutations => {
  for (const m of mutations) {
    for (const node of m.addedNodes) {
      if (node instanceof Element) patchCmSearchInputs(node);
    }
  }
}).observe(document.documentElement, { subtree: true, childList: true });

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
