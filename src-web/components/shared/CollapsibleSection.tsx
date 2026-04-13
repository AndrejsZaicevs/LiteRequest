import { ChevronDown, ChevronRight } from "lucide-react";

interface CollapsibleSectionProps {
  title: string;
  count?: number;
  isOpen: boolean;
  onToggle: () => void;
  children: React.ReactNode;
  /** Optional node rendered on the right side of the header (e.g. an Add button) */
  action?: React.ReactNode;
}

export function CollapsibleSection({ title, count, isOpen, onToggle, children, action }: CollapsibleSectionProps) {
  return (
    <div className="border-b border-gray-700/50 last:border-0">
      <button
        onClick={onToggle}
        className="w-full h-10 flex items-center justify-between px-4 bg-[#1e1e1e] hover:bg-[#252525] transition-colors select-none"
      >
        <div className="flex items-center gap-2">
          {isOpen
            ? <ChevronDown size={13} className="text-gray-400" />
            : <ChevronRight size={13} className="text-gray-400" />}
          <span className="text-xs font-semibold tracking-wide text-gray-200 uppercase">{title}</span>
        </div>
        <div className="flex items-center gap-2" onClick={e => e.stopPropagation()}>
          {count !== undefined && count > 0 && (
            <span className="text-[10px] leading-none bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded-full font-medium">
              {count}
            </span>
          )}
          {action}
        </div>
      </button>
      {isOpen && (
        <div className="p-4">
          {children}
        </div>
      )}
    </div>
  );
}
