import { useState, useMemo } from "react";
import { Play, Terminal, Upload } from "lucide-react";
import type { RequestData, KeyValuePair, HttpMethod } from "../../lib/types";
import { methodColor, HTTP_METHODS } from "../../lib/types";
import { CodeEditor } from "./CodeEditor";
import { VariableInput } from "../shared/VariableInput";

interface RequestEditorProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  onSend: () => void;
  onCopyCurl: () => void;
  onImportCurl: (curlStr: string) => void;
  isLoading: boolean;
  basePath: string;
  requestName: string;
  variables?: Record<string, string>;
}

export function RequestEditor({ data, onChange, onSend, onCopyCurl, onImportCurl, isLoading, basePath, requestName, variables = {} }: RequestEditorProps) {
  const [bodyTab, setBodyTab] = useState<"none" | "json" | "form" | "raw">(
    data.body_type === "Json" ? "json" : data.body_type === "FormUrlEncoded" ? "form" : data.body_type === "Raw" ? "raw" : "none"
  );
  const [showImport, setShowImport] = useState(false);
  const [importText, setImportText] = useState("");

  const updateField = <K extends keyof RequestData>(field: K, value: RequestData[K]) => {
    onChange({ ...data, [field]: value });
  };

  const handleBodyTabChange = (tab: "none" | "json" | "form" | "raw") => {
    setBodyTab(tab);
    const btMap = { none: "None", json: "Json", form: "FormUrlEncoded", raw: "Raw" } as const;
    updateField("body_type", btMap[tab]);
  };

  const showBasePath = basePath && !(data.url.startsWith("http://") || data.url.startsWith("https://"));

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* URL Bar */}
      <div className="p-4 border-b border-gray-800 bg-[#121212] flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="flex rounded-md overflow-hidden border border-gray-700/60 flex-1 bg-[#1a1a1a]">
            {/* Method selector */}
            <select
              value={data.method}
              onChange={(e) => updateField("method", e.target.value as HttpMethod)}
              className="bg-transparent font-semibold text-sm pl-3 pr-8 py-2 outline-none border-r border-gray-700/60 cursor-pointer"
              style={{ color: methodColor(data.method), appearance: "none", borderRadius: 0, border: "none", borderRight: "1px solid rgba(55,65,81,0.6)" }}
            >
              {HTTP_METHODS.map(m => (
                <option key={m} value={m} style={{ color: "#d1d5db" }}>{m}</option>
              ))}
            </select>

            {/* Base path + URL input */}
            <div className="flex-1 flex items-center px-3 py-2 font-mono text-sm relative overflow-visible">
              {showBasePath && (
                <span className="text-gray-500 shrink-0 select-none mr-px" title={basePath}>
                  {basePath.replace(/\/+$/, "")}
                </span>
              )}
              <VariableInput
                value={data.url}
                onChange={(v) => updateField("url", v)}
                variables={variables}
                wrapperClassName="flex-1 min-w-[100px]"
                className="bg-transparent text-gray-200 outline-none"
                inputStyle={{ border: "none", borderRadius: 0, padding: 0, fontSize: "inherit" }}
                placeholder={showBasePath ? "" : "https://api.example.com/path"}
                onKeyDown={(e) => { if (e.key === "Enter") onSend(); }}
              />
            </div>
          </div>

          {/* Send button */}
          <button
            onClick={onSend}
            disabled={isLoading}
            className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 shadow-sm disabled:opacity-50 disabled:cursor-wait"
          >
            {isLoading ? "Sending…" : <><span>Send</span> <Play size={14} className="fill-white" /></>}
          </button>

          {/* cURL actions */}
          <button
            onClick={onCopyCurl}
            title="Copy as cURL"
            className="p-2 rounded-md text-gray-400 hover:text-gray-200 hover:bg-gray-700/50 transition-colors border border-gray-700/60"
          >
            <Terminal size={15} />
          </button>
          <button
            onClick={() => { setImportText(""); setShowImport(true); }}
            title="Import from cURL"
            className="p-2 rounded-md text-gray-400 hover:text-gray-200 hover:bg-gray-700/50 transition-colors border border-gray-700/60"
          >
            <Upload size={15} />
          </button>
        </div>
      </div>

      {/* Import cURL modal */}
      {showImport && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm px-4"
          onClick={() => setShowImport(false)}
        >
          <div
            className="w-full max-w-2xl bg-[#161616] border border-gray-700 rounded-xl shadow-2xl overflow-hidden flex flex-col"
            onClick={e => e.stopPropagation()}
          >
            <div className="px-5 py-4 border-b border-gray-800 flex items-center gap-2">
              <Terminal size={15} className="text-gray-400" />
              <span className="text-sm font-semibold text-gray-200">Import from cURL</span>
            </div>
            <div className="p-5">
              <textarea
                autoFocus
                value={importText}
                onChange={e => setImportText(e.target.value)}
                placeholder={"curl 'https://api.example.com/data' \\\n  -H 'Authorization: Bearer token' \\\n  -H 'Content-Type: application/json' \\\n  --data '{\"key\":\"value\"}'"}
                className="w-full h-40 bg-[#0d0d0d] border border-gray-700 rounded-md p-3 font-mono text-xs text-gray-200 placeholder-gray-600 outline-none focus:border-gray-600 resize-none"
              />
            </div>
            <div className="px-5 pb-5 flex justify-end gap-3">
              <button
                onClick={() => setShowImport(false)}
                className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (importText.trim()) {
                    onImportCurl(importText.trim());
                    setShowImport(false);
                  }
                }}
                disabled={!importText.trim()}
                className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-md font-medium transition-colors disabled:opacity-40"
              >
                Import
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Body toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-[#121212] flex-shrink-0">
        <div className="flex items-center gap-3">
          <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Request Body</span>
          {bodyTab === "json" && (
            <button
              onClick={() => {
                try {
                  const formatted = JSON.stringify(JSON.parse(data.body), null, 2);
                  updateField("body", formatted);
                } catch { /* invalid JSON, ignore */ }
              }}
              className="text-[10px] px-2 py-0.5 rounded text-gray-500 hover:text-gray-300 hover:bg-gray-700/50 border border-gray-700/50 transition-colors font-mono"
              title="Format JSON"
            >
              {"{ }"}
            </button>
          )}
        </div>
        <div className="flex bg-[#1a1a1a] rounded p-0.5 border border-gray-800">
          {(["none", "json", "form", "raw"] as const).map(tab => (
            <button
              key={tab}
              onClick={() => handleBodyTabChange(tab)}
              className={`text-xs px-3 py-1 rounded-sm transition-colors ${
                bodyTab === tab
                  ? "bg-gray-700 text-gray-200 shadow-sm"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              {tab === "none" ? "None" : tab === "form" ? "Form" : tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Body editor */}
      <div className="flex-1 overflow-hidden bg-[#0d0d0d]">
        {bodyTab === "none" && (
          <div className="flex items-center justify-center h-full text-sm text-gray-600">
          </div>
        )}

        {bodyTab === "json" && (
          <div className="h-full overflow-hidden">
            <CodeEditor
              value={data.body}
              onChange={(v) => updateField("body", v)}
              language="json"
              variables={variables}
            />
          </div>
        )}

        {bodyTab === "form" && (
          <FormEditor
            body={data.body}
            onChange={(body) => updateField("body", body)}
          />
        )}

        {bodyTab === "raw" && (
          <CodeEditor
            value={data.body}
            onChange={(v) => updateField("body", v)}
            language="text"
            variables={variables}
          />
        )}
      </div>
    </div>
  );
}

function FormEditor({ body, onChange }: { body: string; onChange: (body: string) => void }) {
  const pairs: KeyValuePair[] = useMemo(() => {
    if (!body.trim()) return [];
    return body.split("&").map(segment => {
      const [k, ...rest] = segment.split("=");
      return { key: decodeURIComponent(k ?? ""), value: decodeURIComponent(rest.join("=") ?? ""), enabled: true };
    });
  }, [body]);

  const encodePairs = (p: KeyValuePair[]) => {
    return p
      .filter(kv => kv.key || kv.value)
      .map(kv => `${encodeURIComponent(kv.key)}=${encodeURIComponent(kv.value)}`)
      .join("&");
  };

  const update = (i: number, field: keyof KeyValuePair, value: string | boolean) => {
    const next = [...pairs];
    next[i] = { ...next[i], [field]: value };
    onChange(encodePairs(next));
  };

  const remove = (i: number) => onChange(encodePairs(pairs.filter((_, idx) => idx !== i)));
  const add = () => onChange(encodePairs([...pairs, { key: "", value: "", enabled: true }]));

  const last = pairs[pairs.length - 1];
  const needsEmpty = !last || last.key !== "" || last.value !== "";

  return (
    <div className="p-2">
      {pairs.map((p, i) => (
        <div key={i} className="kv-row">
          <button className="kv-action always-visible" onClick={() => update(i, "enabled", !p.enabled)}>
            <div className={`w-3.5 h-3.5 rounded-sm flex items-center justify-center border transition-colors ${
              p.enabled ? "bg-blue-500 border-blue-500" : "border-gray-600"
            }`}>
              {p.enabled && <span className="text-white text-[10px]">✓</span>}
            </div>
          </button>
          <input value={p.key} onChange={(e) => update(i, "key", e.target.value)}
            placeholder="key" className="kv-cell" />
          <input value={p.value} onChange={(e) => update(i, "value", e.target.value)}
            placeholder="value" className="kv-cell" />
          {(p.key || p.value) && (
            <button className="kv-action" onClick={() => remove(i)}>×</button>
          )}
        </div>
      ))}
      {needsEmpty && (
        <div className="kv-row placeholder-row" onClick={add}>
          <div style={{ width: 20 }} />
          <div className="kv-cell" style={{ color: "#4b5563" }}>key</div>
          <div className="kv-cell" style={{ color: "#4b5563" }}>value</div>
        </div>
      )}
    </div>
  );
}
