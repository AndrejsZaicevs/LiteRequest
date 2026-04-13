import { useState, useRef } from "react";
import type { Collection, Folder, Request, HttpMethod } from "../../lib/types";
import { methodColor } from "../../lib/types";
import * as api from "../../lib/api";

interface SidebarProps {
  collections: Collection[];
  folders: Folder[];
  requests: Request[];
  selectedRequestId: string | null;
  selectedCollectionId: string | null;
  requestMeta: Map<string, { method: HttpMethod; url: string }>;
  onSelectRequest: (req: Request) => void;
  onSelectCollection: (id: string) => void;
  onDataChange: () => void;
}

export function Sidebar({
  collections, folders, requests,
  selectedRequestId, selectedCollectionId, requestMeta,
  onSelectRequest, onSelectCollection, onDataChange,
}: SidebarProps) {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; type: string; id: string } | null>(null);
  const [renaming, setRenaming] = useState<{ type: string; id: string; value: string } | null>(null);
  const renameRef = useRef<HTMLInputElement>(null);

  const toggle = (id: string) => {
    setCollapsed(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const handleContextMenu = (e: React.MouseEvent, type: string, id: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, type, id });
  };

  const closeContextMenu = () => setContextMenu(null);

  const handleRename = async () => {
    if (!renaming) return;
    try {
      if (renaming.type === "collection") await api.renameCollection(renaming.id, renaming.value);
      else if (renaming.type === "folder") await api.renameFolder(renaming.id, renaming.value);
      else if (renaming.type === "request") await api.renameRequest(renaming.id, renaming.value);
      onDataChange();
    } catch (e) { console.error(e); }
    setRenaming(null);
  };

  const handleDelete = async (type: string, id: string) => {
    try {
      if (type === "collection") await api.deleteCollection(id);
      else if (type === "folder") await api.deleteFolder(id);
      else if (type === "request") await api.deleteRequest(id);
      onDataChange();
    } catch (e) { console.error(e); }
    closeContextMenu();
  };

  const handleNewCollection = async () => {
    const col: Collection = {
      id: crypto.randomUUID(),
      name: "New Collection",
      base_path: "",
      auth_config: null,
      headers_config: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      await api.insertCollection(col);
      onDataChange();
      setRenaming({ type: "collection", id: col.id, value: col.name });
    } catch (e) { console.error(e); }
  };

  const handleNewFolder = async (collectionId: string, parentFolderId?: string) => {
    const folder: Folder = {
      id: crypto.randomUUID(),
      collection_id: collectionId,
      parent_folder_id: parentFolderId ?? null,
      name: "New Folder",
      path_prefix: "",
      auth_override: null,
      sort_order: 0,
    };
    try {
      await api.insertFolder(folder);
      onDataChange();
      setRenaming({ type: "folder", id: folder.id, value: folder.name });
    } catch (e) { console.error(e); }
    closeContextMenu();
  };

  const handleNewRequest = async (collectionId: string, folderId?: string) => {
    const req: Request = {
      id: crypto.randomUUID(),
      collection_id: collectionId,
      folder_id: folderId ?? null,
      name: "New Request",
      current_version_id: null,
      sort_order: 0,
    };
    try {
      await api.insertRequest(req);
      onDataChange();
      setRenaming({ type: "request", id: req.id, value: req.name });
    } catch (e) { console.error(e); }
    closeContextMenu();
  };

  const renderFolder = (folder: Folder, depth: number) => {
    const isCollapsed = collapsed.has(folder.id);
    const subfolders = folders.filter(f => f.parent_folder_id === folder.id);
    const folderRequests = requests.filter(r => r.folder_id === folder.id);

    return (
      <div key={folder.id}>
        <div
          className="flex items-center gap-1.5 py-1.5 cursor-pointer hover:bg-[var(--surface-2)] transition-colors"
          style={{ paddingLeft: 12 + depth * 16, paddingRight: 8 }}
          onClick={() => toggle(folder.id)}
          onContextMenu={(e) => handleContextMenu(e, "folder", folder.id)}
        >
          <span className="text-[9px] w-3 text-center flex-shrink-0" style={{ color: "var(--text-muted)" }}>
            {isCollapsed ? "▶" : "▼"}
          </span>
          <span style={{ fontSize: 12 }}>📁</span>
          {renaming?.id === folder.id ? (
            <input
              ref={renameRef}
              value={renaming.value}
              onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
              onBlur={handleRename}
              onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
              className="flex-1 bg-transparent outline-none text-xs"
              style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "0 2px" }}
              autoFocus
            />
          ) : (
            <span className="truncate flex-1 text-xs" style={{ color: "var(--text-secondary)" }}>
              {folder.name}
            </span>
          )}
        </div>
        {!isCollapsed && (
          <div>
            {subfolders.map(f => renderFolder(f, depth + 1))}
            {folderRequests.map(r => renderRequest(r, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  const renderRequest = (req: Request, depth: number) => {
    const isSelected = req.id === selectedRequestId;
    const meta = requestMeta.get(req.id);
    return (
      <div
        key={req.id}
        className="flex items-center gap-1.5 py-1.5 cursor-pointer transition-colors"
        style={{
          paddingLeft: 12 + depth * 16 + 14,
          paddingRight: 8,
          background: isSelected ? "var(--surface-2)" : "transparent",
          borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
        }}
        onClick={() => onSelectRequest(req)}
        onContextMenu={(e) => handleContextMenu(e, "request", req.id)}
        onDoubleClick={() => setRenaming({ type: "request", id: req.id, value: req.name })}
        onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
        onMouseLeave={(e) => { if (!isSelected) e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
      >
        {meta && (
          <span
            className="font-mono text-[9px] font-bold flex-shrink-0"
            style={{ color: methodColor(meta.method), width: 28, textAlign: "right" }}
          >
            {meta.method.length > 3 ? meta.method.slice(0, 3) : meta.method}
          </span>
        )}
        {!meta && <span style={{ width: 28 }} />}
        {renaming?.id === req.id ? (
          <input
            ref={renameRef}
            value={renaming.value}
            onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
            onBlur={handleRename}
            onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
            className="flex-1 bg-transparent outline-none text-xs"
            style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "0 2px" }}
            autoFocus
          />
        ) : (
          <span className="truncate flex-1 text-xs">{req.name}</span>
        )}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col" style={{ background: "var(--surface-1)" }}>
      {/* Header */}
      <div
        className="flex items-center justify-between px-3 py-2.5 border-b flex-shrink-0"
        style={{ borderColor: "var(--border)" }}
      >
        <span className="text-[11px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
          Collections
        </span>
        <button
          onClick={handleNewCollection}
          className="btn-pill accent"
          style={{ padding: "2px 8px", fontSize: 10 }}
          title="New Collection"
        >
          + New
        </button>
      </div>

      {/* Tree */}
      <div className="flex-1 overflow-y-auto py-1" onClick={closeContextMenu}>
        {collections.map(col => {
          const isCollapsed = collapsed.has(col.id);
          const isSelected = col.id === selectedCollectionId;
          const colFolders = folders.filter(f => f.collection_id === col.id && !f.parent_folder_id);
          const orphanRequests = requests.filter(r => r.collection_id === col.id && !r.folder_id);

          return (
            <div key={col.id}>
              {/* Collection header */}
              <div
                className="flex items-center gap-1.5 py-2 cursor-pointer transition-colors"
                style={{
                  paddingLeft: 8,
                  paddingRight: 8,
                  background: isSelected ? "var(--surface-2)" : "transparent",
                  borderBottom: "1px solid var(--border-subtle)",
                }}
                onClick={() => toggle(col.id)}
                onContextMenu={(e) => handleContextMenu(e, "collection", col.id)}
                onDoubleClick={() => onSelectCollection(col.id)}
                onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                onMouseLeave={(e) => { e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
              >
                <span className="text-[9px] w-3 text-center flex-shrink-0" style={{ color: "var(--text-muted)" }}>
                  {isCollapsed ? "▶" : "▼"}
                </span>
                <span style={{ fontSize: 13 }}>📦</span>
                {renaming?.id === col.id ? (
                  <input
                    ref={renameRef}
                    value={renaming.value}
                    onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
                    onBlur={handleRename}
                    onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
                    className="flex-1 bg-transparent outline-none text-xs font-medium"
                    style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "0 2px" }}
                    autoFocus
                  />
                ) : (
                  <span className="truncate flex-1 text-xs font-medium">{col.name}</span>
                )}
                <span className="text-[10px] flex-shrink-0" style={{ color: "var(--text-muted)" }}>
                  {requests.filter(r => r.collection_id === col.id).length}
                </span>
              </div>
              {!isCollapsed && (
                <div className="pb-1">
                  {colFolders.map(f => renderFolder(f, 0))}
                  {orphanRequests.map(r => renderRequest(r, 0))}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <div
          className="fixed z-50 rounded shadow-lg text-xs overflow-hidden"
          style={{
            left: contextMenu.x, top: contextMenu.y,
            background: "var(--surface-2)", border: "1px solid var(--border)",
            minWidth: 160,
          }}
        >
          {contextMenu.type === "collection" && (
            <>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { handleNewRequest(contextMenu.id); }}>
                New Request
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { handleNewFolder(contextMenu.id); }}>
                New Folder
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { onSelectCollection(contextMenu.id); closeContextMenu(); }}>
                Settings
              </button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("collection", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "folder" && (
            <>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewRequest(f.collection_id, f.id);
                }}>
                New Request
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewFolder(f.collection_id, f.id);
                }}>
                New Subfolder
              </button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("folder", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "request" && (
            <>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-2 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("request", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
