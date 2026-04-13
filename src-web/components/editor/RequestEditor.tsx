import { useState, useMemo } from "react";
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

  const resolvedUrl = useMemo(() => {
    if (!basePath) return data.url;
    if (data.url.startsWith("http://") || data.url.startsWith("https://")) return data.url;
    const base = basePath.replace(/\/+$/, "");
    const path = data.url.startsWith("/") ? data.url : `/${data.url}`;
    return `${base}${path}`;
  }, [data.url, basePath]);

  const handleBodyTabChange = (tab: "none" | "json" | "form" | "raw") => {
    setBodyTab(tab);
    const btMap = { none: "None", json: "Json", form: "FormUrlEncoded", raw: "Raw" } as const;
    updateField("body_type", btMap[tab]);
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Request name */}
      <div className="px-3 py-1 text-xs" style={{ color: "var(--text-muted)", background: "var(--surface-1)" }}>
        {requestName}
        {basePath && (
          <span className="ml-2 font-mono text-[10px]" style={{ color: "var(--text-muted)" }}>
            {basePath}
          </span>
        )}
      </div>

      {/* URL Bar */}
      <div className="flex items-center gap-0 border-b" style={{ borderColor: "var(--border)", background: "var(--surface-0)" }}>
        {/* Method selector */}
        <select
          value={data.method}
          onChange={(e) => updateField("method", e.target.value as HttpMethod)}
          className="h-9 px-2 font-mono text-xs font-bold border-r cursor-pointer"
          style={{
            background: "var(--surface-1)",
            color: methodColor(data.method),
            borderColor: "var(--border)",
            borderRadius: 0,
            outline: "none",
          }}
        >
          {HTTP_METHODS.map(m => (
            <option key={m} value={m} style={{ color: methodColor(m) }}>{m}</option>
          ))}
        </select>

        {/* URL input */}
        <input
          value={data.url}
          onChange={(e) => updateField("url", e.target.value)}
          placeholder="Enter URL or path..."
          className="flex-1 h-9 px-3 font-mono text-sm bg-transparent border-none outline-none"
          style={{ color: "var(--text-primary)" }}
          onKeyDown={(e) => { if (e.key === "Enter") onSend(); }}
        />

        {/* Send button */}
        <button
          onClick={onSend}
          disabled={isLoading}
          className="h-9 px-4 font-semibold text-xs text-white transition-colors"
          style={{
            background: isLoading ? "var(--surface-2)" : "var(--accent)",
            cursor: isLoading ? "wait" : "pointer",
          }}
        >
          {isLoading ? "Sending…" : "Send"}
        </button>
      </div>

      {/* Body tabs */}
      <div className="flex items-center border-b" style={{ borderColor: "var(--border)", background: "var(--surface-1)" }}>
        {(["none", "json", "form", "raw"] as const).map(tab => (
          <button
            key={tab}
            onClick={() => handleBodyTabChange(tab)}
            className="px-3 py-1.5 text-xs capitalize transition-colors"
            style={{
              color: bodyTab === tab ? "var(--accent)" : "var(--text-muted)",
              borderBottom: bodyTab === tab ? "2px solid var(--accent)" : "2px solid transparent",
            }}
          >
            {tab === "none" ? "No Body" : tab}
          </button>
        ))}
      </div>

      {/* Body editor */}
      <div className="flex-1 overflow-auto">
        {bodyTab === "none" && (
          <div className="flex items-center justify-center h-full text-xs" style={{ color: "var(--text-muted)" }}>
            This request has no body
          </div>
        )}

        {bodyTab === "json" && (
          <textarea
            value={data.body}
            onChange={(e) => updateField("body", e.target.value)}
            className="w-full h-full p-3 font-mono text-xs resize-none bg-transparent border-none outline-none"
            style={{ color: "var(--text-primary)" }}
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
            className="w-full h-full p-3 font-mono text-xs resize-none bg-transparent border-none outline-none"
            style={{ color: "var(--text-primary)" }}
            placeholder="Raw body content..."
            spellCheck={false}
          />
        )}
      </div>
    </div>
  );
}

function FormEditor({ body, onChange }: { body: string; onChange: (body: string) => void }) {
  // Parse URL-encoded body into key-value pairs for editing
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
    <div className="text-xs">
      {pairs.map((p, i) => (
        <div key={i} className="flex items-center border-b" style={{ borderColor: "var(--border)" }}>
          <button className="w-6 flex-shrink-0 flex items-center justify-center"
            onClick={() => update(i, "enabled", !p.enabled)}>
            <span style={{ color: p.enabled ? "var(--accent)" : "var(--text-muted)" }}>
              {p.enabled ? "☑" : "☐"}
            </span>
          </button>
          <input value={p.key} onChange={(e) => update(i, "key", e.target.value)}
            placeholder="key"
            className="flex-1 bg-transparent border-none outline-none px-2 py-1.5"
            style={{ color: "var(--text-primary)", borderRight: "1px solid var(--border)" }} />
          <input value={p.value} onChange={(e) => update(i, "value", e.target.value)}
            placeholder="value"
            className="flex-1 bg-transparent border-none outline-none px-2 py-1.5"
            style={{ color: "var(--text-primary)" }} />
          {(p.key || p.value) && (
            <button className="w-6 flex-shrink-0 hover:opacity-80" style={{ color: "var(--text-muted)" }}
              onClick={() => remove(i)}>×</button>
          )}
        </div>
      ))}
      {needsEmpty && (
        <div className="flex items-center border-b cursor-text" style={{ borderColor: "var(--border)" }} onClick={add}>
          <div className="w-6" />
          <div className="flex-1 px-2 py-1.5" style={{ color: "var(--text-muted)", borderRight: "1px solid var(--border)" }}>key</div>
          <div className="flex-1 px-2 py-1.5" style={{ color: "var(--text-muted)" }}>value</div>
        </div>
      )}
    </div>
  );
}
