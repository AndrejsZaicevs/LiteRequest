import { useState, useMemo } from "react";
import { Send } from "lucide-react";
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
      <div
        className="flex items-center border-b flex-shrink-0"
        style={{ borderColor: "var(--border)", background: "var(--surface-0)" }}
      >
        {/* Method selector */}
        <select
          value={data.method}
          onChange={(e) => updateField("method", e.target.value as HttpMethod)}
          className="h-12 px-3 font-mono text-base font-bold border-r cursor-pointer flex-shrink-0"
          style={{
            background: "var(--surface-1)",
            color: methodColor(data.method),
            borderColor: "var(--border)",
            borderRadius: 0,
            outline: "none",
            border: "none",
            borderRight: "1px solid var(--border)",
            minWidth: 100,
          }}
        >
          {HTTP_METHODS.map(m => (
            <option key={m} value={m} style={{ color: methodColor(m) }}>{m}</option>
          ))}
        </select>

        {/* Base path prefix (non-editable) */}
        {showBasePath && (
          <span
            className="h-12 flex items-center px-2.5 font-mono text-sm flex-shrink-0 select-none"
            style={{ color: "var(--text-muted)", background: "var(--surface-1)", borderRight: "1px solid var(--border-subtle)" }}
            title={basePath}
          >
            {basePath.replace(/\/+$/, "")}
          </span>
        )}

        {/* URL input */}
        <input
          value={data.url}
          onChange={(e) => updateField("url", e.target.value)}
          placeholder={showBasePath ? "/path..." : "https://api.example.com/path"}
          className="flex-1 h-12 px-3 font-mono text-base bg-transparent outline-none"
          style={{ color: "var(--text-primary)", border: "none", borderRadius: 0 }}
          onKeyDown={(e) => { if (e.key === "Enter") onSend(); }}
        />

        {/* Send button */}
        <button
          onClick={onSend}
          disabled={isLoading}
          className="h-12 px-7 font-semibold text-base text-white transition-colors flex-shrink-0 flex items-center gap-2"
          style={{
            background: isLoading ? "var(--surface-2)" : "var(--accent)",
            cursor: isLoading ? "wait" : "pointer",
          }}
        >
          {isLoading ? "Sending…" : <><Send size={16} /> Send</>}
        </button>
      </div>

      {/* Body tabs */}
      <div
        className="flex items-center border-b flex-shrink-0 gap-1 px-3"
        style={{ borderColor: "var(--border)", background: "var(--surface-1)" }}
      >
        {(["none", "json", "form", "raw"] as const).map(tab => (
          <button
            key={tab}
            onClick={() => handleBodyTabChange(tab)}
            className="px-3 py-3 text-sm font-medium capitalize transition-colors"
            style={{
              color: bodyTab === tab ? "var(--accent)" : "var(--text-muted)",
              borderBottom: bodyTab === tab ? "2px solid var(--accent)" : "2px solid transparent",
            }}
          >
            {tab === "none" ? "No Body" : tab === "form" ? "Form" : tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
        <div className="flex-1" />
        <span className="text-xs truncate" style={{ color: "var(--text-muted)" }}>
          {requestName}
        </span>
      </div>

      {/* Body editor */}
      <div className="flex-1 overflow-auto">
        {bodyTab === "none" && (
          <div className="flex items-center justify-center h-full text-sm" style={{ color: "var(--text-muted)" }}>
            This request has no body
          </div>
        )}

        {bodyTab === "json" && (
          <textarea
            value={data.body}
            onChange={(e) => updateField("body", e.target.value)}
            className="w-full h-full p-4 font-mono text-base leading-relaxed resize-none bg-transparent outline-none"
            style={{ color: "var(--text-primary)", border: "none" }}
            placeholder='{"key": "value"}'
            spellCheck={false}
          />
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
            className="w-full h-full p-4 font-mono text-base leading-relaxed resize-none bg-transparent outline-none"
            style={{ color: "var(--text-primary)", border: "none" }}
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
    <div>
      {pairs.map((p, i) => (
        <div key={i} className="kv-row">
          <button className="kv-action" onClick={() => update(i, "enabled", !p.enabled)}>
            <span style={{ color: p.enabled ? "var(--accent)" : "var(--text-muted)", fontSize: 14 }}>
              {p.enabled ? "✓" : "○"}
            </span>
          </button>
          <input value={p.key} onChange={(e) => update(i, "key", e.target.value)}
            placeholder="key"
            className="kv-cell"
            style={{ border: "none", borderRadius: 0 }} />
          <div className="kv-divider" />
          <input value={p.value} onChange={(e) => update(i, "value", e.target.value)}
            placeholder="value"
            className="kv-cell"
            style={{ border: "none", borderRadius: 0 }} />
          {(p.key || p.value) && (
            <button className="kv-action" style={{ color: "var(--text-muted)" }} onClick={() => remove(i)}>×</button>
          )}
        </div>
      ))}
      {needsEmpty && (
        <div className="kv-row placeholder-row" onClick={add}>
          <div style={{ width: 34 }} />
          <div className="kv-cell" style={{ color: "var(--text-muted)" }}>key</div>
          <div className="kv-divider" />
          <div className="kv-cell" style={{ color: "var(--text-muted)" }}>value</div>
        </div>
      )}
    </div>
  );
}
