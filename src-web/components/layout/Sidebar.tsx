import { useState, useRef } from "react";
import { ChevronRight, ChevronDown, Folder, FolderOpen, Package, GripVertical } from "lucide-react";
import { DndContext, closestCenter, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import type { DragEndEvent } from "@dnd-kit/core";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Collection, Folder as FolderType, Request, HttpMethod } from "../../lib/types";
import { methodColor } from "../../lib/types";
import * as api from "../../lib/api";

interface SidebarProps {
  collections: Collection[];
  folders: FolderType[];
  requests: Request[];
  selectedRequestId: string | null;
  selectedCollectionId: string | null;
  requestMeta: Map<string, { method: HttpMethod; url: string }>;
  onSelectRequest: (req: Request) => void;
  onSelectCollection: (id: string) => void;
  onDataChange: () => void;
}

interface SortableRequestProps {
  req: Request;
  depth: number;
  isSelected: boolean;
  meta: { method: HttpMethod; url: string } | undefined;
  renaming: { type: string; id: string; value: string } | null;
  renameRef: React.RefObject<HTMLInputElement | null>;
  setRenaming: (r: { type: string; id: string; value: string } | null) => void;
  handleRename: () => void;
  onSelectRequest: (req: Request) => void;
  handleContextMenu: (e: React.MouseEvent, type: string, id: string) => void;
}

function SortableRequest({
  req, depth, isSelected, meta, renaming, renameRef, setRenaming, handleRename,
  onSelectRequest, handleContextMenu,
}: SortableRequestProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: req.id });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
    paddingLeft: 16 + depth * 20 + 20,
    paddingRight: 10,
    background: isSelected ? "var(--surface-2)" : "transparent",
    borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-2 py-2.5 cursor-pointer transition-colors group"
      onClick={() => onSelectRequest(req)}
      onContextMenu={(e) => handleContextMenu(e, "request", req.id)}
      onDoubleClick={() => setRenaming({ type: "request", id: req.id, value: req.name })}
      onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
      onMouseLeave={(e) => { if (!isSelected) e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
    >
      <span
        {...attributes}
        {...listeners}
        className="flex-shrink-0 opacity-0 group-hover:opacity-50 cursor-grab"
        style={{ color: "var(--text-muted)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <GripVertical size={14} />
      </span>
      {meta && (
        <span
          className="font-mono text-xs font-bold flex-shrink-0"
          style={{ color: methodColor(meta.method), width: 38, textAlign: "right" }}
        >
          {meta.method.length > 3 ? meta.method.slice(0, 3) : meta.method}
        </span>
      )}
      {!meta && <span style={{ width: 38 }} />}
      {renaming?.id === req.id ? (
        <input
          ref={renameRef}
          value={renaming.value}
          onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
          onBlur={handleRename}
          onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
          className="flex-1 bg-transparent outline-none text-sm"
          style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "1px 3px" }}
          autoFocus
        />
      ) : (
        <span className="truncate flex-1 text-sm">{req.name}</span>
      )}
    </div>
  );
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

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

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
    const folder: FolderType = {
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

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const activeReq = requests.find(r => r.id === active.id);
    const overReq = requests.find(r => r.id === over.id);
    if (!activeReq || !overReq) return;

    // Only reorder within the same container (same folder_id and collection_id)
    if (activeReq.folder_id !== overReq.folder_id || activeReq.collection_id !== overReq.collection_id) return;

    const siblings = requests
      .filter(r => r.collection_id === activeReq.collection_id && r.folder_id === activeReq.folder_id)
      .sort((a, b) => a.sort_order - b.sort_order);

    const ids = siblings.map(r => r.id);
    const oldIndex = ids.indexOf(activeReq.id);
    const newIndex = ids.indexOf(overReq.id);
    if (oldIndex === -1 || newIndex === -1) return;

    ids.splice(oldIndex, 1);
    ids.splice(newIndex, 0, activeReq.id);

    try {
      await api.reorderRequests(ids);
      onDataChange();
    } catch (e) { console.error(e); }
  };

  const renderFolder = (folder: FolderType, depth: number) => {
    const isCollapsed = collapsed.has(folder.id);
    const subfolders = folders.filter(f => f.parent_folder_id === folder.id);
    const folderRequests = requests
      .filter(r => r.folder_id === folder.id)
      .sort((a, b) => a.sort_order - b.sort_order);

    return (
      <div key={folder.id}>
        <div
          className="flex items-center gap-2 py-2.5 cursor-pointer hover:bg-[var(--surface-2)] transition-colors"
          style={{ paddingLeft: 16 + depth * 20, paddingRight: 10 }}
          onClick={() => toggle(folder.id)}
          onContextMenu={(e) => handleContextMenu(e, "folder", folder.id)}
        >
          <span className="flex-shrink-0 transition-transform" style={{ color: "var(--text-muted)" }}>
            {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
          </span>
          <span style={{ color: "var(--text-muted)" }}>
            {isCollapsed ? <Folder size={16} /> : <FolderOpen size={16} />}
          </span>
          {renaming?.id === folder.id ? (
            <input
              ref={renameRef}
              value={renaming.value}
              onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
              onBlur={handleRename}
              onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
              className="flex-1 bg-transparent outline-none text-sm"
              style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "1px 3px" }}
              autoFocus
            />
          ) : (
            <span className="truncate flex-1 text-sm" style={{ color: "var(--text-secondary)" }}>
              {folder.name}
            </span>
          )}
        </div>
        {!isCollapsed && (
          <div>
            {subfolders.map(f => renderFolder(f, depth + 1))}
            <SortableContext items={folderRequests.map(r => r.id)} strategy={verticalListSortingStrategy}>
              {folderRequests.map(r => (
                <SortableRequest
                  key={r.id}
                  req={r}
                  depth={depth + 1}
                  isSelected={r.id === selectedRequestId}
                  meta={requestMeta.get(r.id)}
                  renaming={renaming}
                  renameRef={renameRef}
                  setRenaming={setRenaming}
                  handleRename={handleRename}
                  onSelectRequest={onSelectRequest}
                  handleContextMenu={handleContextMenu}
                />
              ))}
            </SortableContext>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col" style={{ background: "var(--surface-1)" }}>
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b flex-shrink-0"
        style={{ borderColor: "var(--border)" }}
      >
        <span className="text-sm font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>
          Collections
        </span>
        <button
          onClick={handleNewCollection}
          className="btn-pill accent"
          title="New Collection"
          style={{ padding: "8px 18px", fontSize: 13 }}
        >
          + New
        </button>
      </div>

      {/* Tree */}
      <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
        <div className="flex-1 overflow-y-auto py-1.5" onClick={closeContextMenu}>
          {collections.map(col => {
            const isCollapsed = collapsed.has(col.id);
            const isSelected = col.id === selectedCollectionId;
            const colFolders = folders.filter(f => f.collection_id === col.id && !f.parent_folder_id);
            const orphanRequests = requests
              .filter(r => r.collection_id === col.id && !r.folder_id)
              .sort((a, b) => a.sort_order - b.sort_order);

            return (
              <div key={col.id}>
                {/* Collection header */}
                <div
                  className="flex items-center gap-2 py-3 cursor-pointer transition-colors"
                  style={{
                    paddingLeft: 10,
                    paddingRight: 10,
                    background: isSelected ? "var(--surface-2)" : "transparent",
                    borderBottom: "1px solid var(--border-subtle)",
                  }}
                  onClick={() => toggle(col.id)}
                  onContextMenu={(e) => handleContextMenu(e, "collection", col.id)}
                  onDoubleClick={() => onSelectCollection(col.id)}
                  onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                  onMouseLeave={(e) => { e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
                >
                  <span className="flex-shrink-0 transition-transform" style={{ color: "var(--text-muted)" }}>
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                  </span>
                  <Package size={17} style={{ color: "var(--text-secondary)", flexShrink: 0 }} />
                  {renaming?.id === col.id ? (
                    <input
                      ref={renameRef}
                      value={renaming.value}
                      onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
                      onBlur={handleRename}
                      onKeyDown={(e) => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
                      className="flex-1 bg-transparent outline-none text-sm font-medium"
                      style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "1px 3px" }}
                      autoFocus
                    />
                  ) : (
                    <span className="truncate flex-1 text-sm font-medium">{col.name}</span>
                  )}
                  <span className="text-xs flex-shrink-0 tabular-nums" style={{ color: "var(--text-muted)" }}>
                    {requests.filter(r => r.collection_id === col.id).length}
                  </span>
                </div>
                {!isCollapsed && (
                  <div className="pb-1.5">
                    {colFolders.map(f => renderFolder(f, 0))}
                    <SortableContext items={orphanRequests.map(r => r.id)} strategy={verticalListSortingStrategy}>
                      {orphanRequests.map(r => (
                        <SortableRequest
                          key={r.id}
                          req={r}
                          depth={0}
                          isSelected={r.id === selectedRequestId}
                          meta={requestMeta.get(r.id)}
                          renaming={renaming}
                          renameRef={renameRef}
                          setRenaming={setRenaming}
                          handleRename={handleRename}
                          onSelectRequest={onSelectRequest}
                          handleContextMenu={handleContextMenu}
                        />
                      ))}
                    </SortableContext>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </DndContext>

      {/* Context menu */}
      {contextMenu && (
        <div
          className="fixed z-50 rounded-md shadow-lg text-sm overflow-hidden"
          style={{
            left: contextMenu.x, top: contextMenu.y,
            background: "var(--surface-2)", border: "1px solid var(--border)",
            minWidth: 180,
          }}
        >
          {contextMenu.type === "collection" && (
            <>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { handleNewRequest(contextMenu.id); }}>
                New Request
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { handleNewFolder(contextMenu.id); }}>
                New Folder
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { onSelectCollection(contextMenu.id); closeContextMenu(); }}>
                Settings
              </button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("collection", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "folder" && (
            <>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewRequest(f.collection_id, f.id);
                }}>
                New Request
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => {
                  const f = folders.find(ff => ff.id === contextMenu.id);
                  if (f) handleNewFolder(f.collection_id, f.id);
                }}>
                New Subfolder
              </button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("folder", contextMenu.id)}>
                Delete
              </button>
            </>
          )}
          {contextMenu.type === "request" && (
            <>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>
                Rename
              </button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
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
