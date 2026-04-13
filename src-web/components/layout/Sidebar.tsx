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

  const renderFolder = (folder: Folder) => {
    const isCollapsed = collapsed.has(folder.id);
    const subfolders = folders.filter(f => f.parent_folder_id === folder.id);
    const folderRequests = requests.filter(r => r.folder_id === folder.id);

    return (
      <div key={folder.id}>
        <div
          className="flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-[var(--surface-2)] text-xs"
          onClick={() => toggle(folder.id)}
          onContextMenu={(e) => handleContextMenu(e, "folder", folder.id)}
        >
          <span className="text-[10px]">{isCollapsed ? "▶" : "▼"}</span>
          <span className="text-[11px]">📁</span>
          {renaming?.id === folder.id ? (
            <input
              ref={renameRef}
              value={renaming.value}
              onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
              onBlur={handleRename}
              onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
              className="flex-1 bg-transparent border-b border-[var(--accent)] outline-none text-xs px-0"
              autoFocus
            />
          ) : (
            <span className="truncate flex-1" style={{ color: "var(--text-secondary)" }}>{folder.name}</span>
          )}
        </div>
        {!isCollapsed && (
          <div className="ml-3">
            {subfolders.map(renderFolder)}
            {folderRequests.map(renderRequest)}
          </div>
        )}
      </div>
    );
  };

  const renderRequest = (req: Request) => {
    const isSelected = req.id === selectedRequestId;
    const meta = requestMeta.get(req.id);
    return (
      <div
        key={req.id}
        className="flex items-center gap-1.5 px-2 py-1 cursor-pointer text-xs"
        style={{
          background: isSelected ? "var(--surface-2)" : "transparent",
          borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
        }}
        onClick={() => onSelectRequest(req)}
        onContextMenu={(e) => handleContextMenu(e, "request", req.id)}
        onDoubleClick={() => setRenaming({ type: "request", id: req.id, value: req.name })}
      >
        {meta && (
          <span
            className="font-mono text-[10px] font-bold w-8 text-right"
            style={{ color: methodColor(meta.method) }}
          >
            {meta.method.slice(0, 3)}
          </span>
        )}
        {renaming?.id === req.id ? (
          <input
            ref={renameRef}
            value={renaming.value}
            onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
            onBlur={handleRename}
            onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
            className="flex-1 bg-transparent border-b border-[var(--accent)] outline-none text-xs px-0"
            autoFocus
          />
        ) : (
          <span className="truncate flex-1">{req.name}</span>
        )}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col" style={{ background: "var(--surface-1)" }}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b" style={{ borderColor: "var(--border)" }}>
        <span className="text-xs font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
          Collections
        </span>
        <button
          onClick={handleNewCollection}
          className="text-sm hover:opacity-80"
          style={{ color: "var(--accent)" }}
          title="New Collection"
        >
          +
        </button>
      </div>

      {/* Tree */}
      <div className="flex-1 overflow-y-auto py-1" onClick={closeContextMenu}>
        {collections.map(col => {
          const isCollapsed = collapsed.has(col.id);
          const colFolders = folders.filter(f => f.collection_id === col.id && !f.parent_folder_id);
          const orphanRequests = requests.filter(r => r.collection_id === col.id && !r.folder_id);

          return (
            <div key={col.id}>
              <div
                className="flex items-center gap-1 px-2 py-1.5 cursor-pointer hover:bg-[var(--surface-2)] text-xs font-medium"
                onClick={() => toggle(col.id)}
                onContextMenu={(e) => handleContextMenu(e, "collection", col.id)}
                onDoubleClick={() => onSelectCollection(col.id)}
              >
                <span className="text-[10px]">{isCollapsed ? "▶" : "▼"}</span>
                {renaming?.id === col.id ? (
                  <input
                    ref={renameRef}
                    value={renaming.value}
                    onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
                    onBlur={handleRename}
                    onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
                    className="flex-1 bg-transparent border-b border-[var(--accent)] outline-none text-xs px-0"
                    autoFocus
                  />
                ) : (
                  <span className="truncate flex-1">{col.name}</span>
                )}
              </div>
              {!isCollapsed && (
                <div className="ml-2">
                  {colFolders.map(renderFolder)}
                  {orphanRequests.map(renderRequest)}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <div
          className="fixed z-50 py-1 rounded shadow-lg text-xs"
          style={{
            left: contextMenu.x, top: contextMenu.y,
            background: "var(--surface-2)", border: "1px solid var(--border)",
            minWidth: 160,
          }}
        >
          {contextMenu.type === "collection" && (
            <>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { handleNewRequest(contextMenu.id); }}>
                New Request
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { handleNewFolder(contextMenu.id); }}>
                New Folder
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { onSelectCollection(contextMenu.id); closeContextMenu(); }}>
                Settings
              </button>
              <hr className="my-1" style={{ borderColor: "var(--border)" }} />
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("collection", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "folder" && (
            <>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewRequest(f.collection_id, f.id);
                }}>
                New Request
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewFolder(f.collection_id, f.id);
                }}>
                New Subfolder
              </button>
              <hr className="my-1" style={{ borderColor: "var(--border)" }} />
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("folder", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "request" && (
            <>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]"
                onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-3 py-1.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }}
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
