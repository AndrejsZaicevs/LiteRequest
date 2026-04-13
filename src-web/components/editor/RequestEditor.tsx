import { useState, useMemo } from "react";
import { Play } from "lucide-react";
import type { RequestData, KeyValuePair, HttpMethod } from "../../lib/types";
import { methodColor, HTTP_METHODS } from "../../lib/types";

interface RequestEditorProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  onSend: () => void;
  isLoading: boolean;
  basePath: string;
  requestName: string;
}

export function RequestEditor({ data, onChange, onSend, isLoading, basePath, requestName }: RequestEditorProps) {
  const [bodyTab, setBodyTab] = useState<"none" | "json" | "form" | "raw">(
    data.body_type === "Json" ? "json" : data.body_type === "FormUrlEncoded" ? "form" : data.body_type === "Raw" ? "raw" : "none"
  );

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
            <div className="flex-1 flex items-center px-3 py-2 font-mono text-sm overflow-hidden">
              {showBasePath && (
                <span className="text-gray-500 shrink-0 select-none mr-px" title={basePath}>
                  {basePath.replace(/\/+$/, "")}
                </span>
              )}
              <input
                value={data.url}
                onChange={(e) => updateField("url", e.target.value)}
                placeholder={showBasePath ? "/endpoint..." : "https://api.example.com/path"}
                className="flex-1 bg-transparent text-gray-200 outline-none w-full min-w-[100px]"
                style={{ border: "none", borderRadius: 0, padding: 0, fontSize: "inherit" }}
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
        </div>
      </div>

      {/* Body toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-[#121212] flex-shrink-0">
        <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Request Body</span>
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
      <div className="flex-1 overflow-auto bg-[#0d0d0d]">
        {bodyTab === "none" && (
          <div className="flex items-center justify-center h-full text-sm text-gray-600">
            This request has no body
          </div>
        )}

        {bodyTab === "json" && (
          <div className="relative h-full">
            <textarea
              value={data.body}
              onChange={(e) => updateField("body", e.target.value)}
              className="w-full h-full p-4 font-mono text-sm leading-relaxed resize-none bg-transparent outline-none text-gray-300"
              style={{ border: "none" }}
              placeholder={'{\n  "key": "value"\n}'}
              spellCheck={false}
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
          <textarea
            value={data.body}
            onChange={(e) => updateField("body", e.target.value)}
            className="w-full h-full p-4 font-mono text-sm leading-relaxed resize-none bg-transparent outline-none text-gray-300"
            style={{ border: "none" }}
            placeholder="Raw body content..."
            spellCheck={false}
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
