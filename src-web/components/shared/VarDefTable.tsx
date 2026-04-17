import { Eye, EyeOff, Trash2 } from "lucide-react";
import type { VarRow } from "../../lib/types";

interface VarDefTableProps {
  defs: Array<{ id: string; key: string }>;
  rows: VarRow[];
  /** Whether an environment is selected, enabling value editing */
  hasEnv: boolean;
  onKeyChange: (defId: string, key: string) => void;
  onValueChange: (row: VarRow, value: string) => void;
  /** Called when the user types a value but no row exists for the def yet */
  onValueCreate: (defId: string, defKey: string, value: string) => void;
  onToggleSecret: (row: VarRow) => void;
  onDelete: (defId: string) => void;
  emptyMessage?: string;
  valuePlaceholder?: string;
}

export function VarDefTable({
  defs,
  rows,
  hasEnv,
  onKeyChange,
  onValueChange,
  onValueCreate,
  onToggleSecret,
  onDelete,
  emptyMessage = "No variables — click + Add",
  valuePlaceholder = "value",
}: VarDefTableProps) {
  return (
    <div className="border border-gray-800 rounded-md overflow-hidden">
      {defs.map(def => {
        const row = rows.find(r => r.def_id === def.id);
        return (
          <div key={def.id} className="kv-row">
            <input
              value={def.key}
              onChange={e => onKeyChange(def.id, e.target.value)}
              className="kv-cell"
              style={{ border: "none", borderRadius: 0, fontWeight: 500 }}
            />
            <div className="kv-divider" />
            <input
              value={row?.value ?? ""}
              type={row?.is_secret ? "password" : "text"}
              onChange={e => {
                if (row) {
                  onValueChange(row, e.target.value);
                } else if (hasEnv) {
                  onValueCreate(def.id, def.key, e.target.value);
                }
              }}
              placeholder={hasEnv ? valuePlaceholder : "—"}
              className="kv-cell"
              style={{ border: "none", borderRadius: 0 }}
              disabled={!hasEnv}
            />
            {row && (
              <button
                onClick={() => onToggleSecret(row)}
                className="kv-action text-gray-600 hover:text-gray-300"
                title={row.is_secret ? "Show value" : "Hide value"}
              >
                {row.is_secret ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
            )}
            <button
              onClick={() => onDelete(def.id)}
              className="kv-action text-gray-600 hover:text-red-400"
              title="Delete variable"
            >
              <Trash2 size={12} />
            </button>
          </div>
        );
      })}
      {defs.length === 0 && (
        <div className="px-4 py-4 text-xs text-center text-gray-600">{emptyMessage}</div>
      )}
    </div>
  );
}
