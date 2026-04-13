import type { KeyValuePair } from "../../lib/types";

interface KvTableProps {
  rows: KeyValuePair[];
  onChange: (rows: KeyValuePair[]) => void;
  placeholder?: { key: string; value: string };
  fixedKeys?: boolean;
}

export function KvTable({ rows, onChange, placeholder, fixedKeys }: KvTableProps) {
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

  // Auto-add empty row at bottom
  const displayRows = [...rows];
  const lastRow = displayRows[displayRows.length - 1];
  const needsEmptyRow = !fixedKeys && (!lastRow || lastRow.key !== "" || lastRow.value !== "");

  return (
    <div className="text-xs">
      {displayRows.map((row, i) => (
        <div
          key={i}
          className="flex items-center border-b"
          style={{ borderColor: "var(--border)", opacity: row.enabled ? 1 : 0.5 }}
        >
          {/* Enable checkbox */}
          {!fixedKeys && (
            <button
              className="w-6 flex-shrink-0 flex items-center justify-center"
              onClick={() => update(i, "enabled", !row.enabled)}
            >
              <span style={{ color: row.enabled ? "var(--accent)" : "var(--text-muted)" }}>
                {row.enabled ? "☑" : "☐"}
              </span>
            </button>
          )}

          {/* Key */}
          <input
            value={row.key}
            onChange={(e) => update(i, "key", e.target.value)}
            placeholder={placeholder?.key ?? "key"}
            className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 min-w-0"
            style={{ color: "var(--text-primary)", borderRight: "1px solid var(--border)" }}
            readOnly={fixedKeys}
          />

          {/* Value */}
          <input
            value={row.value}
            onChange={(e) => update(i, "value", e.target.value)}
            placeholder={placeholder?.value ?? "value"}
            className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 min-w-0"
            style={{ color: "var(--text-primary)" }}
          />

          {/* Delete */}
          {!fixedKeys && (row.key !== "" || row.value !== "") && (
            <button
              className="w-6 flex-shrink-0 flex items-center justify-center hover:opacity-80"
              style={{ color: "var(--text-muted)" }}
              onClick={() => remove(i)}
            >
              ×
            </button>
          )}
        </div>
      ))}

      {/* Auto-add row trigger */}
      {needsEmptyRow && (
        <div
          className="flex items-center border-b cursor-text"
          style={{ borderColor: "var(--border)" }}
          onClick={addRow}
        >
          {!fixedKeys && <div className="w-6 flex-shrink-0" />}
          <div className="flex-1 px-2 py-1.5" style={{ color: "var(--text-muted)", borderRight: "1px solid var(--border)" }}>
            {placeholder?.key ?? "key"}
          </div>
          <div className="flex-1 px-2 py-1.5" style={{ color: "var(--text-muted)" }}>
            {placeholder?.value ?? "value"}
          </div>
        </div>
      )}
    </div>
  );
}
