import { useState, useEffect, useRef, lazy, Suspense } from "react";
import { Paperclip, Maximize2, Minimize2 } from "lucide-react";
import type { RequestData, KeyValuePair, MultipartField } from "../../lib/types";
import { CodeEditor } from "./CodeEditor";
import { VariableInput } from "../shared/VariableInput";
import { open as dialogOpen } from "@tauri-apps/plugin-dialog";

const ScriptEditorLazy = lazy(() =>
  import("./ScriptEditor").then(m => ({ default: m.ScriptEditor }))
);

interface RequestEditorProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  isLoading: boolean;
  basePath: string;
  requestName: string;
  variables?: Record<string, string>;
  isMaximized?: boolean;
  onMaximize?: () => void;
  /** Post-execution script content (TypeScript) */
  postScript?: string;
  onPostScriptChange?: (script: string) => void;
}

type EditorPanel = "body" | "script";
type BodyTab = "none" | "json" | "form" | "raw" | "multipart";

function bodyTypeToTab(bt: string): BodyTab {
  if (bt === "Json") return "json";
  if (bt === "FormUrlEncoded") return "form";
  if (bt === "Raw") return "raw";
  if (bt === "Multipart") return "multipart";
  return "none";
}

export function RequestEditor({ data, onChange, isLoading, basePath, requestName, variables = {}, isMaximized, onMaximize, postScript, onPostScriptChange }: RequestEditorProps) {
  const [bodyTab, setBodyTab] = useState<BodyTab>(() => bodyTypeToTab(data.body_type));
  const [panel, setPanel] = useState<EditorPanel>("body");
  const hasScript = (postScript ?? "").trim().length > 0;

  // Sync tab when switching to a different request
  useEffect(() => {
    setBodyTab(bodyTypeToTab(data.body_type));
  }, [data.body_type]);

  const updateField = <K extends keyof RequestData>(field: K, value: RequestData[K]) => {
    onChange({ ...data, [field]: value });
  };

  const handleBodyTabChange = (tab: BodyTab) => {
    setBodyTab(tab);
    const btMap = { none: "None", json: "Json", form: "FormUrlEncoded", raw: "Raw", multipart: "Multipart" } as const;
    updateField("body_type", btMap[tab]);
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">

      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-[#121212] flex-shrink-0">
        <div className="flex items-center gap-3">
          {/* Panel toggle: Body / Script */}
          <div className="flex bg-[#1a1a1a] rounded p-0.5 border border-gray-800 mr-2">
            <button
              onClick={() => setPanel("body")}
              className={`text-xs px-3 py-1 rounded-sm transition-colors ${
                panel === "body"
                  ? "bg-gray-700 text-gray-200 shadow-sm"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              Body
            </button>
            <button
              onClick={() => setPanel("script")}
              className={`text-xs px-3 py-1 rounded-sm transition-colors relative ${
                panel === "script"
                  ? "bg-gray-700 text-gray-200 shadow-sm"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              Script
              {hasScript && panel !== "script" && (
                <span className="absolute -top-0.5 -right-0.5 w-1.5 h-1.5 rounded-full bg-blue-400" />
              )}
            </button>
          </div>

          {panel === "body" && (
            <>
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
            </>
          )}

          {panel === "script" && (
            <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Post-Execution Script</span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {panel === "body" && (
            <div className="flex bg-[#1a1a1a] rounded p-0.5 border border-gray-800">
              {(["none", "json", "form", "raw", "multipart"] as const).map(tab => (
                <button
                  key={tab}
                  onClick={() => handleBodyTabChange(tab)}
                  className={`text-xs px-3 py-1 rounded-sm transition-colors ${
                    bodyTab === tab
                      ? "bg-gray-700 text-gray-200 shadow-sm"
                      : "text-gray-500 hover:text-gray-300"
                  }`}
                >
                  {tab === "none" ? "None" : tab === "form" ? "Form" : tab === "multipart" ? "Multipart" : tab.charAt(0).toUpperCase() + tab.slice(1)}
                </button>
              ))}
            </div>
          )}
          {onMaximize && (
            <button
              onClick={onMaximize}
              title={isMaximized ? "Restore split" : "Maximize request"}
              className="p-1 rounded text-gray-600 hover:text-gray-300 hover:bg-gray-700/50 transition-colors"
            >
              {isMaximized ? <Minimize2 size={13} /> : <Maximize2 size={13} />}
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden bg-[#0d0d0d]">
        {panel === "body" && (
          <>
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

            {bodyTab === "multipart" && (
              <MultipartEditor
                fields={data.multipart_fields ?? []}
                onChange={(fields) => updateField("multipart_fields", fields)}
                variables={variables}
              />
            )}
          </>
        )}

        {panel === "script" && (
          <Suspense fallback={
            <div className="h-full flex items-center justify-center">
              <span className="text-xs text-gray-600">Loading editor…</span>
            </div>
          }>
            <ScriptEditorLazy
              value={postScript ?? ""}
              onChange={(v) => onPostScriptChange?.(v)}
              mode="post-exec"
            />
          </Suspense>
        )}
      </div>
    </div>
  );
}

function decodePairs(body: string): KeyValuePair[] {
  if (!body.trim()) return [];
  return body.split("&").map(segment => {
    const [k, ...rest] = segment.split("=");
    return {
      key: decodeURIComponent(k ?? ""),
      value: decodeURIComponent(rest.join("=") ?? ""),
      enabled: true,
    };
  });
}

function encodePairs(pairs: KeyValuePair[]): string {
  return pairs
    .filter(kv => kv.key || kv.value)
    .map(kv => `${encodeURIComponent(kv.key)}=${encodeURIComponent(kv.value)}`)
    .join("&");
}

function FormEditor({ body, onChange }: { body: string; onChange: (body: string) => void }) {
  const [pairs, setPairs] = useState<KeyValuePair[]>(() => decodePairs(body));
  // Track the last body string we emitted so we can distinguish external changes
  const lastEmitted = useRef(body);

  useEffect(() => {
    // Only re-initialize from body if it changed externally (not from our own onChange)
    if (body !== lastEmitted.current) {
      lastEmitted.current = body;
      setPairs(decodePairs(body));
    }
  }, [body]);

  const commit = (next: KeyValuePair[]) => {
    setPairs(next);
    const encoded = encodePairs(next);
    lastEmitted.current = encoded;
    onChange(encoded);
  };

  const update = (i: number, field: keyof KeyValuePair, value: string | boolean) => {
    const next = [...pairs];
    next[i] = { ...next[i], [field]: value };
    commit(next);
  };

  const remove = (i: number) => commit(pairs.filter((_, idx) => idx !== i));

  const addRow = () => {
    const next = [...pairs, { key: "", value: "", enabled: true }];
    setPairs(next); // don't encode empty row — just show it locally
  };

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
        <div className="kv-row placeholder-row" onClick={addRow}>
          <div style={{ width: 20 }} />
          <div className="kv-cell" style={{ color: "#4b5563" }}>key</div>
          <div className="kv-cell" style={{ color: "#4b5563" }}>value</div>
        </div>
      )}
    </div>
  );
}

// ── MultipartEditor ───────────────────────────────────────────

function MultipartEditor({
  fields,
  onChange,
  variables = {},
}: {
  fields: MultipartField[];
  onChange: (fields: MultipartField[]) => void;
  variables?: Record<string, string>;
}) {
  const [rows, setRows] = useState<MultipartField[]>(fields);
  const lastEmitted = useRef(JSON.stringify(fields));

  useEffect(() => {
    const incoming = JSON.stringify(fields);
    if (incoming !== lastEmitted.current) {
      lastEmitted.current = incoming;
      setRows(fields);
    }
  }, [fields]);

  const commit = (next: MultipartField[]) => {
    setRows(next);
    lastEmitted.current = JSON.stringify(next);
    onChange(next);
  };

  const update = (i: number, patch: Partial<MultipartField>) => {
    const next = rows.map((r, idx) => idx === i ? { ...r, ...patch } : r);
    commit(next);
  };

  const remove = (i: number) => commit(rows.filter((_, idx) => idx !== i));

  const addRow = () => {
    setRows(prev => [...prev, { key: "", value: "", is_file: false, file_path: "", enabled: true }]);
  };

  const pickFile = async (i: number) => {
    const selected = await dialogOpen({ multiple: false, directory: false });
    if (typeof selected === "string") {
      update(i, { file_path: selected, is_file: true });
    }
  };

  const last = rows[rows.length - 1];
  const needsEmpty = !last || last.key !== "" || last.value !== "" || last.file_path !== "";

  return (
    <div className="p-2">
      {rows.map((f, i) => (
        <div key={i} className="kv-row">
          {/* enabled */}
          <button className="kv-action always-visible" onClick={() => update(i, { enabled: !f.enabled })}>
            <div className={`w-3.5 h-3.5 rounded-sm flex items-center justify-center border transition-colors ${
              f.enabled ? "bg-blue-500 border-blue-500" : "border-gray-600"
            }`}>
              {f.enabled && <span className="text-white text-[10px]">✓</span>}
            </div>
          </button>
          {/* key */}
          <VariableInput
            value={f.key}
            onChange={(v) => update(i, { key: v })}
            placeholder="key"
            className="kv-cell"
            variables={variables}
          />
          {/* type toggle */}
          <button
            title={f.is_file ? "Switch to text" : "Switch to file"}
            onClick={() => update(i, { is_file: !f.is_file, value: "", file_path: "" })}
            className={`kv-action always-visible transition-colors ${f.is_file ? "text-blue-400" : "text-gray-600 hover:text-gray-400"}`}
          >
            <Paperclip size={12} />
          </button>
          {/* value or file path */}
          {f.is_file ? (
            <div className="flex flex-1 min-w-0 items-center gap-1">
              <span
                className="kv-cell truncate cursor-default text-gray-400"
                title={f.file_path}
              >
                {f.file_path ? f.file_path.split(/[\\/]/).pop() : <span style={{ color: "#4b5563" }}>no file chosen</span>}
              </span>
              <button
                onClick={() => pickFile(i)}
                className="flex-shrink-0 text-[10px] px-2 py-0.5 rounded text-gray-500 hover:text-gray-200 hover:bg-gray-700/60 border border-gray-700/50 transition-colors"
              >
                Browse
              </button>
            </div>
          ) : (
            <VariableInput
              value={f.value}
              onChange={(v) => update(i, { value: v })}
              placeholder="value"
              className="kv-cell"
              variables={variables}
            />
          )}
          {(f.key || f.value || f.file_path) && (
            <button className="kv-action" onClick={() => remove(i)}>×</button>
          )}
        </div>
      ))}
      {needsEmpty && (
        <div className="kv-row placeholder-row" onClick={addRow}>
          <div style={{ width: 20 }} />
          <div className="kv-cell" style={{ color: "#4b5563" }}>key</div>
          <div style={{ width: 20 }} />
          <div className="kv-cell" style={{ color: "#4b5563" }}>value or file</div>
        </div>
      )}
    </div>
  );
}
