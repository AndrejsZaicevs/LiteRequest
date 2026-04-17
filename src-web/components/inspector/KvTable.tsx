import { Trash2, Check, Eye, EyeOff } from "lucide-react";
import type { KeyValuePair } from "../../lib/types";
import { VariableInput } from "../shared/VariableInput";

interface KvTableProps {
  rows: KeyValuePair[];
  onChange: (rows: KeyValuePair[]) => void;
  placeholder?: { key: string; value: string };
  fixedKeys?: boolean;
  variables?: Record<string, string>;
  showSecretToggle?: boolean;
}

export function KvTable({ rows, onChange, placeholder, fixedKeys, variables = {}, showSecretToggle }: KvTableProps) {
  const update = (index: number, field: keyof KeyValuePair, value: string | boolean) => {
    const next = [...rows];
    next[index] = { ...next[index], [field]: value };
    onChange(next);
  };

  const remove = (index: number) => {
    onChange(rows.filter((_, i) => i !== index));
  };

  const addRow = () => {
    onChange([...rows, { key: "", value: "", enabled: true }]);
  };

  const displayRows = [...rows];
  const lastRow = displayRows[displayRows.length - 1];
  const needsEmptyRow = !fixedKeys && (!lastRow || lastRow.key !== "" || lastRow.value !== "");

  return (
    <div className="flex flex-col gap-0.5">
      {displayRows.map((row, i) => (
        <div
          key={i}
          className="group flex items-center gap-1.5 h-7 border-b border-transparent hover:border-gray-800"
          style={{ opacity: row.enabled ? 1 : 0.45 }}
        >
          {/* Checkbox */}
          <div className="w-4 flex justify-center shrink-0">
            {!fixedKeys && (
              <button
                className={`w-3.5 h-3.5 rounded-sm flex items-center justify-center border transition-colors ${
                  row.enabled ? "bg-blue-500 border-blue-500" : "border-gray-600 hover:border-gray-400"
                }`}
                onClick={() => update(i, "enabled", !row.enabled)}
              >
                {row.enabled && <Check size={10} className="text-white" />}
              </button>
            )}
            {fixedKeys && (
              <div className="w-3.5 h-3.5 flex items-center justify-center opacity-50">
                <div className="w-1.5 h-1.5 rounded-full bg-blue-500" />
              </div>
            )}
          </div>

          {/* Key input — capped width so value column grows on wide panels */}
          <input
            value={row.key}
            onChange={(e) => update(i, "key", e.target.value)}
            placeholder={placeholder?.key ?? "key"}
            className={`min-w-[80px] max-w-[180px] w-[35%] shrink bg-transparent text-xs outline-none placeholder-gray-600 border border-transparent rounded px-1.5 py-0.5 transition-all text-gray-200 ${
              !fixedKeys ? "focus:border-gray-700 focus:bg-[#1a1a1a]" : "text-gray-500 font-mono cursor-default"
            } ${!row.enabled ? "opacity-40 line-through" : ""}`}
            readOnly={fixedKeys}
          />

          {/* Value input — takes remaining space */}
          {row.is_secret ? (
            <input
              value={row.value}
              type="password"
              onChange={(e) => update(i, "value", e.target.value)}
              className={`flex-1 min-w-0 bg-transparent text-xs outline-none placeholder-gray-600 border border-transparent focus:border-gray-700 focus:bg-[#1a1a1a] rounded px-1.5 py-0.5 transition-all text-gray-200 ${
                !row.enabled ? "opacity-40 line-through" : ""
              }`}
              placeholder={placeholder?.value ?? "value"}
            />
          ) : (
            <VariableInput
              value={row.value}
              onChange={(v) => update(i, "value", v)}
              variables={variables}
              wrapperClassName="flex-1 min-w-0"
              className={`bg-transparent text-xs outline-none placeholder-gray-600 border border-transparent focus:border-gray-700 focus:bg-[#1a1a1a] rounded px-1.5 py-0.5 transition-all text-gray-200 ${
                !row.enabled ? "opacity-40 line-through" : ""
              }`}
              placeholder={placeholder?.value ?? "value"}
            />
          )}

          {/* Secret toggle */}
          {showSecretToggle && (row.key !== "" || row.value !== "") && (
            <div className="w-5 flex justify-center shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onClick={() => update(i, "is_secret", !row.is_secret)}
                className="text-gray-500 hover:text-gray-300 p-0.5 rounded"
                title={row.is_secret ? "Show value" : "Hide value"}
              >
                {row.is_secret ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
            </div>
          )}

          {/* Delete */}
          <div className="w-5 flex justify-center shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
            {!fixedKeys && (row.key !== "" || row.value !== "") && (
              <button onClick={() => remove(i)} className="text-gray-500 hover:text-red-400 p-0.5 rounded">
                <Trash2 size={12} />
              </button>
            )}
          </div>
        </div>
      ))}

      {needsEmptyRow && (
        <div className="group flex items-center gap-1.5 h-7 opacity-40 cursor-text" onClick={addRow}>
          <div className="w-4 shrink-0" />
          <span className="min-w-[80px] max-w-[180px] w-[35%] shrink text-xs px-1.5 py-1 text-gray-600">
            {placeholder?.key ?? "key"}
          </span>
          <span className="flex-1 min-w-0 text-xs px-1.5 py-1 text-gray-600">
            {placeholder?.value ?? "value"}
          </span>
          <div className="w-5 shrink-0" />
        </div>
      )}
    </div>
  );
}
