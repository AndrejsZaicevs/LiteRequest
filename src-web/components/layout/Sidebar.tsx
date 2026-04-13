import { useState, useRef, useMemo, useCallback } from "react";
import { ChevronRight, ChevronDown, Folder, FolderOpen, Package, GripVertical } from "lucide-react";
import {
  DndContext, DragOverlay, PointerSensor, useSensor, useSensors,
  useDraggable, useDroppable,
} from "@dnd-kit/core";
import type { DragMoveEvent, DragEndEvent, DragStartEvent } from "@dnd-kit/core";
import type { Collection, Folder as FolderType, Request, HttpMethod } from "../../lib/types";
import { methodColor } from "../../lib/types";
import * as api from "../../lib/api";

// ── Types ─────────────────────────────────────────────────────

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

type TreeItem = {
  id: string;
  type: "folder" | "request";
  depth: number;
  parentFolderId: string | null;
  collectionId: string;
  folder?: FolderType;
  request?: Request;
};

type DropState = {
  overId: string;
  position: "before" | "after" | "inside";
};

// ── Flat tree builder (used for DnD logic) ────────────────────

function buildFlatTree(
  collections: Collection[],
  folders: FolderType[],
  requests: Request[],
  collapsed: Set<string>,
): TreeItem[] {
  const items: TreeItem[] = [];
  for (const col of collections) {
    if (collapsed.has(col.id)) continue;
    const addFolder = (folder: FolderType, depth: number) => {
      items.push({ id: folder.id, type: "folder", depth, parentFolderId: folder.parent_folder_id, collectionId: folder.collection_id, folder });
      if (!collapsed.has(folder.id)) {
        folders.filter(f => f.parent_folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order).forEach(f => addFolder(f, depth + 1));
        requests.filter(r => r.folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order)
          .forEach(r => items.push({ id: r.id, type: "request", depth: depth + 1, parentFolderId: folder.id, collectionId: folder.collection_id, request: r }));
      }
    };
    folders.filter(f => f.collection_id === col.id && !f.parent_folder_id).sort((a, b) => a.sort_order - b.sort_order).forEach(f => addFolder(f, 0));
    requests.filter(r => r.collection_id === col.id && !r.folder_id).sort((a, b) => a.sort_order - b.sort_order)
      .forEach(r => items.push({ id: r.id, type: "request", depth: 0, parentFolderId: null, collectionId: col.id, request: r }));
  }
  return items;
}

// ── Drop indicator line ───────────────────────────────────────

function DropLine({ side, indent }: { side: "top" | "bottom"; indent: number }) {
  return (
    <div style={{ position: "absolute", [side]: -1, left: indent, right: 0, height: 2, background: "var(--accent)", borderRadius: 2, zIndex: 30, pointerEvents: "none" }}>
      <div style={{ position: "absolute", left: -3, top: -2, width: 6, height: 6, borderRadius: "50%", background: "var(--accent)" }} />
    </div>
  );
}

// ── DnD wrapper — no transform animation, just indicator lines ─

function DnDRow({ item, dropState, children }: {
  item: TreeItem;
  dropState: DropState | null;
  children: (isDropInside: boolean) => React.ReactNode;
}) {
  const { attributes, listeners, setNodeRef: setDragRef, isDragging } = useDraggable({ id: item.id, data: item });
  const { setNodeRef: setDropRef } = useDroppable({ id: item.id, data: item });

  const setRef = (node: HTMLElement | null) => { setDragRef(node); setDropRef(node); };

  const isBefore = dropState?.overId === item.id && dropState.position === "before";
  const isAfter  = dropState?.overId === item.id && dropState.position === "after";
  const isInside = dropState?.overId === item.id && dropState.position === "inside";
  const indent   = 28 + item.depth * 20;

  return (
    <div ref={setRef} style={{ position: "relative", opacity: isDragging ? 0 : 1 }} {...attributes} {...listeners}>
      {isBefore && <DropLine side="top"    indent={indent} />}
      {children(isInside)}
      {isAfter  && <DropLine side="bottom" indent={indent} />}
    </div>
  );
}

// ── Main sidebar ──────────────────────────────────────────────

export function Sidebar({
  collections, folders, requests,
  selectedRequestId, selectedCollectionId, requestMeta,
  onSelectRequest, onSelectCollection, onDataChange,
}: SidebarProps) {
  const [collapsed,   setCollapsed]   = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; type: string; id: string } | null>(null);
  const [renaming,    setRenaming]    = useState<{ type: string; id: string; value: string } | null>(null);
  const [activeDragId, setActiveDragId] = useState<string | null>(null);
  const [dropState,   setDropState]   = useState<DropState | null>(null);
  const renameRef = useRef<HTMLInputElement>(null);

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 8 } }));

  const flatItems = useMemo(
    () => buildFlatTree(collections, folders, requests, collapsed),
    [collections, folders, requests, collapsed],
  );

  // ── Helpers ─────────────────────────────────────────────────

  const toggle = (id: string) => setCollapsed(prev => {
    const next = new Set(prev);
    if (next.has(id)) next.delete(id); else next.add(id);
    return next;
  });

  const handleContextMenu = (e: React.MouseEvent, type: string, id: string) => {
    e.preventDefault(); e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, type, id });
  };
  const closeCtx = () => setContextMenu(null);

  const handleRename = async () => {
    if (!renaming) return;
    try {
      if (renaming.type === "collection") await api.renameCollection(renaming.id, renaming.value);
      else if (renaming.type === "folder")  await api.renameFolder(renaming.id, renaming.value);
      else if (renaming.type === "request") await api.renameRequest(renaming.id, renaming.value);
      onDataChange();
    } catch (e) { console.error(e); }
    setRenaming(null);
  };

  const handleDelete = async (type: string, id: string) => {
    try {
      if (type === "collection") await api.deleteCollection(id);
      else if (type === "folder")  await api.deleteFolder(id);
      else if (type === "request") await api.deleteRequest(id);
      onDataChange();
    } catch (e) { console.error(e); }
    closeCtx();
  };

  const handleNewCollection = async () => {
    const col: Collection = { id: crypto.randomUUID(), name: "New Collection", base_path: "", auth_config: null, headers_config: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() };
    try { await api.insertCollection(col); onDataChange(); setRenaming({ type: "collection", id: col.id, value: col.name }); } catch (e) { console.error(e); }
  };

  const handleNewFolder = async (collectionId: string, parentFolderId?: string) => {
    const f: FolderType = { id: crypto.randomUUID(), collection_id: collectionId, parent_folder_id: parentFolderId ?? null, name: "New Folder", path_prefix: "", auth_override: null, sort_order: 0 };
    try { await api.insertFolder(f); onDataChange(); setRenaming({ type: "folder", id: f.id, value: f.name }); } catch (e) { console.error(e); }
    closeCtx();
  };

  const handleNewRequest = async (collectionId: string, folderId?: string) => {
    const r: Request = { id: crypto.randomUUID(), collection_id: collectionId, folder_id: folderId ?? null, name: "New Request", current_version_id: null, sort_order: 0 };
    try { await api.insertRequest(r); onDataChange(); setRenaming({ type: "request", id: r.id, value: r.name }); } catch (e) { console.error(e); }
    closeCtx();
  };

  // ── DnD handlers ────────────────────────────────────────────

  const handleDragStart = useCallback((event: DragStartEvent) => {
    setActiveDragId(String(event.active.id));
  }, []);

  // onDragMove fires on every pointer move — gives smooth before/after/inside feedback
  const handleDragMove = useCallback((event: DragMoveEvent) => {
    const { active, over } = event;
    if (!over || over.id === active.id) { setDropState(null); return; }

    const overItem = over.data.current as TreeItem | undefined;
    if (!overItem) { setDropState(null); return; }

    const activeDragRect = active.rect.current.translated;
    const overRect = over.rect;
    let position: "before" | "after" | "inside" = "after";

    if (activeDragRect) {
      const activeMidY = activeDragRect.top + activeDragRect.height / 2;
      const overTop    = overRect.top;
      const overH      = overRect.height;

      if (overItem.type === "folder") {
        const zone = overH * 0.28;
        if (activeMidY < overTop + zone) position = "before";
        else if (activeMidY > overTop + overH - zone) position = "after";
        else position = "inside";
      } else {
        position = activeMidY < overTop + overH / 2 ? "before" : "after";
      }
    }

    setDropState({ overId: String(over.id), position });
  }, []);

  const handleDragEnd = useCallback(async (event: DragEndEvent) => {
    const ds = dropState;
    setActiveDragId(null);
    setDropState(null);
    if (!ds) return;

    const activeItem = event.active.data.current as TreeItem | undefined;
    const overItem   = flatItems.find(i => i.id === ds.overId);
    if (!activeItem || !overItem || activeItem.id === overItem.id) return;

    if (activeItem.type === "request") {
      const req = activeItem.request!;
      let newFolder: string | null;
      let newCol: string;

      if (ds.position === "inside" && overItem.type === "folder") {
        newFolder = overItem.id; newCol = overItem.collectionId;
      } else {
        newFolder = overItem.parentFolderId; newCol = overItem.collectionId;
      }

      if (req.folder_id !== newFolder || req.collection_id !== newCol) {
        await api.moveRequest(req.id, newCol, newFolder);
      }

      // Build new sibling order
      const sibs = requests
        .filter(r => r.collection_id === newCol && r.folder_id === newFolder)
        .sort((a, b) => a.sort_order - b.sort_order);
      const ids = sibs.map(r => r.id);
      if (!ids.includes(req.id)) ids.push(req.id);
      ids.splice(ids.indexOf(req.id), 1); // remove from old pos

      if (ds.position === "inside") {
        ids.push(req.id);
      } else if (ds.position === "before") {
        const ref = ids.indexOf(overItem.id);
        ids.splice(ref === -1 ? 0 : ref, 0, req.id);
      } else {
        const ref = ids.indexOf(overItem.id);
        ids.splice(ref === -1 ? ids.length : ref + 1, 0, req.id);
      }
      await api.reorderRequests(ids);

    } else if (activeItem.type === "folder") {
      const fld = activeItem.folder!;
      let newParent: string | null;
      let newCol: string;

      if (ds.position === "inside" && overItem.type === "folder") {
        newParent = overItem.id; newCol = overItem.collectionId;
      } else {
        newParent = overItem.parentFolderId; newCol = overItem.collectionId;
      }

      if (fld.parent_folder_id !== newParent || fld.collection_id !== newCol) {
        await api.moveFolder(fld.id, newCol, newParent);
      }

      const sibs = folders
        .filter(f => f.collection_id === newCol && f.parent_folder_id === newParent && f.id !== fld.id)
        .sort((a, b) => a.sort_order - b.sort_order);
      const ids = sibs.map(f => f.id);

      if (ds.position === "before") {
        const ref = ids.indexOf(overItem.id);
        ids.splice(ref === -1 ? 0 : ref, 0, fld.id);
      } else {
        const ref = ids.indexOf(overItem.id);
        ids.splice(ref === -1 ? ids.length : ref + 1, 0, fld.id);
      }
      await api.reorderFolders(ids);
    }

    onDataChange();
  }, [dropState, flatItems, requests, folders, onDataChange]);

  const handleDragCancel = useCallback(() => {
    setActiveDragId(null);
    setDropState(null);
  }, []);

  // ── Row renderers ────────────────────────────────────────────

  const renameInput = (value: string, onChange: (v: string) => void) => (
    <input
      ref={renameRef}
      value={value}
      onChange={e => onChange(e.target.value)}
      onBlur={handleRename}
      onKeyDown={e => { if (e.key === "Enter") handleRename(); if (e.key === "Escape") setRenaming(null); }}
      className="flex-1 bg-transparent outline-none text-sm"
      style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "1px 3px" }}
      autoFocus
      onClick={e => e.stopPropagation()}
      onPointerDown={e => e.stopPropagation()}
    />
  );

  const renderFolder = (folder: FolderType, depth: number): React.ReactNode => {
    const isCollapsed = collapsed.has(folder.id);
    const item = flatItems.find(i => i.id === folder.id);
    const pl = 28 + depth * 20;

    const subfolders = folders.filter(f => f.parent_folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order);
    const folderReqs = requests.filter(r => r.folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order);

    const content = item ? (
      <DnDRow key={folder.id} item={item} dropState={dropState}>
        {(isDropInside) => (
          <div
            className="flex items-center gap-2 py-2.5 cursor-pointer transition-colors group"
            style={{
              paddingLeft: pl, paddingRight: 10,
              background: isDropInside ? "color-mix(in srgb, var(--accent) 18%, transparent)" : "transparent",
              boxShadow: isDropInside ? "inset 0 0 0 1px var(--accent)" : "none",
            }}
            onClick={e => { e.stopPropagation(); toggle(folder.id); }}
            onContextMenu={e => handleContextMenu(e, "folder", folder.id)}
            onDoubleClick={e => { e.stopPropagation(); setRenaming({ type: "folder", id: folder.id, value: folder.name }); }}
            onMouseEnter={e => { if (!isDropInside) e.currentTarget.style.background = "var(--row-hover)"; }}
            onMouseLeave={e => { if (!isDropInside) e.currentTarget.style.background = "transparent"; }}
          >
            <GripVertical size={13} className="flex-shrink-0 opacity-0 group-hover:opacity-30" style={{ color: "var(--text-muted)" }} />
            <span className="flex-shrink-0" style={{ color: isDropInside ? "var(--accent)" : "var(--text-muted)" }}>
              {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
            </span>
            <span style={{ color: isDropInside ? "var(--accent)" : "var(--text-muted)" }}>
              {isCollapsed ? <Folder size={15} /> : <FolderOpen size={15} />}
            </span>
            {renaming?.id === folder.id
              ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
              : <span className="truncate flex-1 text-sm" style={{ color: "var(--text-secondary)" }}>{folder.name}</span>
            }
          </div>
        )}
      </DnDRow>
    ) : null;

    return (
      <div key={folder.id}>
        {content}
        {!isCollapsed && (
          <div>
            {subfolders.map(f => renderFolder(f, depth + 1))}
            {folderReqs.map(r => renderRequest(r, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  const renderRequest = (req: Request, depth: number): React.ReactNode => {
    const isSelected = req.id === selectedRequestId;
    const meta = requestMeta.get(req.id);
    const item = flatItems.find(i => i.id === req.id);
    const pl = 28 + depth * 20;

    if (!item) return null;

    return (
      <DnDRow key={req.id} item={item} dropState={dropState}>
        {(_isDropInside) => (
          <div
            className="flex items-center gap-2 py-2.5 cursor-pointer transition-colors group"
            style={{
              paddingLeft: pl, paddingRight: 10,
              background: isSelected ? "var(--surface-2)" : "transparent",
              borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
            }}
            onClick={e => { e.stopPropagation(); onSelectRequest(req); }}
            onContextMenu={e => handleContextMenu(e, "request", req.id)}
            onDoubleClick={e => { e.stopPropagation(); setRenaming({ type: "request", id: req.id, value: req.name }); }}
            onMouseEnter={e => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
            onMouseLeave={e => { if (!isSelected) e.currentTarget.style.background = "transparent"; }}
          >
            <GripVertical size={13} className="flex-shrink-0 opacity-0 group-hover:opacity-30" style={{ color: "var(--text-muted)" }} />
            {meta
              ? <span className="font-mono text-xs font-bold flex-shrink-0" style={{ color: methodColor(meta.method), width: 36, textAlign: "right" }}>{meta.method.slice(0, 3)}</span>
              : <span style={{ width: 36 }} />
            }
            {renaming?.id === req.id
              ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
              : <span className="truncate flex-1 text-sm">{req.name}</span>
            }
          </div>
        )}
      </DnDRow>
    );
  };

  // Active drag item for DragOverlay
  const activeDragItem = activeDragId ? flatItems.find(i => i.id === activeDragId) : null;

  // ── Render ───────────────────────────────────────────────────

  return (
    <div className="h-full flex flex-col" style={{ background: "var(--surface-1)" }}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b flex-shrink-0" style={{ borderColor: "var(--border)" }}>
        <span className="text-sm font-semibold uppercase tracking-wider" style={{ color: "var(--text-muted)" }}>Collections</span>
        <button onClick={handleNewCollection} className="btn-pill accent" style={{ padding: "8px 18px", fontSize: 13 }}>+ New</button>
      </div>

      {/* Tree */}
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragMove={handleDragMove}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        <div className="flex-1 overflow-y-auto py-1.5" onClick={closeCtx}>
          {collections.map(col => {
            const isCollapsed = collapsed.has(col.id);
            const isSelected  = col.id === selectedCollectionId;
            const colFolders  = folders.filter(f => f.collection_id === col.id && !f.parent_folder_id).sort((a, b) => a.sort_order - b.sort_order);
            const orphans     = requests.filter(r => r.collection_id === col.id && !r.folder_id).sort((a, b) => a.sort_order - b.sort_order);

            return (
              <div key={col.id}>
                {/* Collection header — not draggable */}
                <div
                  className="flex items-center gap-2 py-3 cursor-pointer transition-colors"
                  style={{ paddingLeft: 10, paddingRight: 10, background: isSelected ? "var(--surface-2)" : "transparent", borderBottom: "1px solid var(--border-subtle)" }}
                  onClick={() => toggle(col.id)}
                  onContextMenu={e => handleContextMenu(e, "collection", col.id)}
                  onDoubleClick={() => onSelectCollection(col.id)}
                  onMouseEnter={e => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                  onMouseLeave={e => { e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
                >
                  <span className="flex-shrink-0" style={{ color: "var(--text-muted)" }}>
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                  </span>
                  <Package size={17} style={{ color: "var(--text-secondary)", flexShrink: 0 }} />
                  {renaming?.id === col.id
                    ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
                    : <span className="truncate flex-1 text-sm font-medium">{col.name}</span>
                  }
                  <span className="text-xs flex-shrink-0 tabular-nums" style={{ color: "var(--text-muted)" }}>
                    {requests.filter(r => r.collection_id === col.id).length}
                  </span>
                </div>

                {!isCollapsed && (
                  <div className="pb-1.5">
                    {colFolders.map(f => renderFolder(f, 0))}
                    {orphans.map(r => renderRequest(r, 0))}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Floating drag preview */}
        <DragOverlay>
          {activeDragItem && (
            <div className="flex items-center gap-2 rounded text-sm px-3 py-2 shadow-xl" style={{ background: "var(--surface-3)", border: "1px solid var(--accent)", opacity: 0.95, minWidth: 160 }}>
              {activeDragItem.type === "folder" ? (
                <><Folder size={14} style={{ color: "var(--accent)" }} /><span>{activeDragItem.folder?.name}</span></>
              ) : (
                <>
                  {activeDragItem.request && requestMeta.get(activeDragItem.request.id) && (
                    <span className="font-mono text-xs font-bold" style={{ color: methodColor(requestMeta.get(activeDragItem.request.id)!.method) }}>
                      {requestMeta.get(activeDragItem.request.id)!.method.slice(0, 3)}
                    </span>
                  )}
                  <span>{activeDragItem.request?.name}</span>
                </>
              )}
            </div>
          )}
        </DragOverlay>
      </DndContext>

      {/* Context menu */}
      {contextMenu && (
        <div className="fixed z-50 rounded-md shadow-lg text-sm overflow-hidden" style={{ left: contextMenu.x, top: contextMenu.y, background: "var(--surface-2)", border: "1px solid var(--border)", minWidth: 180 }}>
          {contextMenu.type === "collection" && (<>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => handleNewRequest(contextMenu.id)}>New Request</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => handleNewFolder(contextMenu.id)}>New Folder</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { onSelectCollection(contextMenu.id); closeCtx(); }}>Settings</button>
            <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }} onClick={() => handleDelete("collection", contextMenu.id)}>Delete</button>
          </>)}
          {contextMenu.type === "folder" && (<>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewRequest(f.collection_id, f.id); }}>New Request</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewFolder(f.collection_id, f.id); }}>New Subfolder</button>
            <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }} onClick={() => handleDelete("folder", contextMenu.id)}>Delete</button>
          </>)}
          {contextMenu.type === "request" && (<>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)]" style={{ color: "var(--danger)" }} onClick={() => handleDelete("request", contextMenu.id)}>Delete</button>
          </>)}
        </div>
      )}
    </div>
  );
}
