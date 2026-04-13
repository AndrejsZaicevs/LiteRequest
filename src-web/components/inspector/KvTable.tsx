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

  const displayRows = [...rows];
  const lastRow = displayRows[displayRows.length - 1];
  const needsEmptyRow = !fixedKeys && (!lastRow || lastRow.key !== "" || lastRow.value !== "");

  return (
    <div>
      {displayRows.map((row, i) => (
        <div
          key={i}
          className="kv-row"
          style={{ opacity: row.enabled ? 1 : 0.45 }}
        >
          {!fixedKeys && (
            <button
              className="kv-action"
              onClick={() => update(i, "enabled", !row.enabled)}
            >
              <span style={{ color: row.enabled ? "var(--accent)" : "var(--text-muted)", fontSize: 14 }}>
                {row.enabled ? "✓" : "○"}
              </span>
            </button>
          )}
          {fixedKeys && <div style={{ width: 10 }} />}

          <input
            value={row.key}
            onChange={(e) => update(i, "key", e.target.value)}
            placeholder={placeholder?.key ?? "key"}
            className="kv-cell"
            style={{ borderRight: "none", borderRadius: 0, border: "none" }}
            readOnly={fixedKeys}
          />

          <div className="kv-divider" />

          <input
            value={row.value}
            onChange={(e) => update(i, "value", e.target.value)}
            placeholder={placeholder?.value ?? "value"}
            className="kv-cell"
            style={{ border: "none", borderRadius: 0 }}
          />

          {!fixedKeys && (row.key !== "" || row.value !== "") ? (
            <button
              className="kv-action"
              onClick={() => remove(i)}
              style={{ color: "var(--text-muted)" }}
            >
              ×
            </button>
          ) : (
            <div style={{ width: 34 }} />
          )}
        </div>
      ))}

      {needsEmptyRow && (
        <div className="kv-row placeholder-row" onClick={addRow}>
          {!fixedKeys && <div style={{ width: 34 }} />}
          <div className="kv-cell" style={{ color: "var(--text-muted)" }}>
            {placeholder?.key ?? "key"}
          </div>
          <div className="kv-divider" />
          <div className="kv-cell" style={{ color: "var(--text-muted)" }}>
            {placeholder?.value ?? "value"}
          </div>
          <div style={{ width: 34 }} />
        </div>
      )}
    </div>
  );
}
