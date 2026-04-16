import { useState, useRef, useMemo, useCallback } from "react";
import { ChevronRight, ChevronDown, Folder, FolderOpen, FileText, Plus, Upload, Download } from "lucide-react";
import {
  DndContext, DragOverlay, PointerSensor, useSensor, useSensors,
  useDraggable, useDroppable,
} from "@dnd-kit/core";
import type { DragMoveEvent, DragEndEvent, DragStartEvent } from "@dnd-kit/core";
import type { Collection, Folder as FolderType, Request, HttpMethod } from "../../lib/types";
import { METHOD_STYLES } from "../../lib/types";
import * as api from "../../lib/api";
import { open as dialogOpen, save as dialogSave } from "@tauri-apps/plugin-dialog";

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
    <div style={{ position: "absolute", [side]: -1, left: indent, right: 4, height: 2, background: "#3b82f6", borderRadius: 2, zIndex: 30, pointerEvents: "none" }}>
      <div style={{ position: "absolute", left: -3, top: -2, width: 6, height: 6, borderRadius: "50%", background: "#3b82f6" }} />
    </div>
  );
}

// ── DnD wrapper ───────────────────────────────────────────────

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
  const indent   = item.depth * 12 + 8;

  return (
    <div ref={setRef} style={{ position: "relative", opacity: isDragging ? 0.3 : 1 }} {...attributes} {...listeners}>
      {isBefore && <DropLine side="top"    indent={indent} />}
      {children(isInside)}
      {isAfter  && <DropLine side="bottom" indent={indent} />}
    </div>
  );
}

// ── Drop zone at the bottom of a collection (allows dragging to root) ──

function CollectionEndZone({ collectionId, dropState }: { collectionId: string; dropState: DropState | null }) {
  const zoneId = `col-end-${collectionId}`;
  const { setNodeRef } = useDroppable({ id: zoneId, data: { type: "col-end", collectionId } });
  const isTarget = dropState?.overId === zoneId;
  return (
    <div ref={setNodeRef} style={{ height: 10, position: "relative", marginLeft: 8 }}>
      {isTarget && <DropLine side="top" indent={0} />}
    </div>
  );
}

// ── Main sidebar ──────────────────────────────────────────────

export function Sidebar({
  collections, folders, requests,
  selectedRequestId, selectedCollectionId, requestMeta,
  onSelectRequest, onSelectCollection, onDataChange,
}: SidebarProps) {
  const [collapsed, setCollapsed] = useState<Set<string>>(() => {
    try {
      const saved = localStorage.getItem("lr.collapsed");
      return saved ? new Set(JSON.parse(saved) as string[]) : new Set();
    } catch { return new Set(); }
  });
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; type: string; id: string } | null>(null);
  const [renaming,    setRenaming]    = useState<{ type: string; id: string; value: string } | null>(null);
  const [activeDragId, setActiveDragId] = useState<string | null>(null);
  const [dropState,   setDropState]   = useState<DropState | null>(null);
  const renameRef = useRef<HTMLInputElement>(null);
  // Keep a always-current ref to folders so stable callbacks can read them
  const foldersRef = useRef(folders);
  foldersRef.current = folders;

  // Returns true if setting folderId's parent to targetParentId would create a cycle.
  // Checks whether folderId is already an ancestor of targetParentId.
  const wouldCreateCycle = useCallback((folderId: string, targetParentId: string | null): boolean => {
    if (!targetParentId) return false;
    if (targetParentId === folderId) return true;
    let cur = foldersRef.current.find(f => f.id === targetParentId);
    while (cur) {
      if (cur.parent_folder_id === folderId) return true;
      if (!cur.parent_folder_id) return false;
      cur = foldersRef.current.find(f => f.id === cur!.parent_folder_id);
    }
    return false;
  }, []);

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 8 } }));

  const flatItems = useMemo(
    () => buildFlatTree(collections, folders, requests, collapsed),
    [collections, folders, requests, collapsed],
  );

  // ── Helpers ─────────────────────────────────────────────────

  const toggle = (id: string) => setCollapsed(prev => {
    const next = new Set(prev);
    if (next.has(id)) next.delete(id); else next.add(id);
    try { localStorage.setItem("lr.collapsed", JSON.stringify([...next])); } catch { /* ignore */ }
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

  const handleImportPostman = async () => {
    try {
      const selected = await dialogOpen({ multiple: false, directory: false, filters: [{ name: "JSON", extensions: ["json"] }] });
      if (!selected) return;
      const path = typeof selected === "string" ? selected : selected[0];
      const summary = await api.importPostmanCollection(path);
      onDataChange();
      alert(`Imported "${summary.collection_name}": ${summary.requests} requests, ${summary.folders} folders.`);
    } catch (e: unknown) {
      console.error(e);
      alert(`Import failed: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleExportCollection = async (collectionId: string) => {
    closeCtx();
    try {
      const col = collections.find(c => c.id === collectionId);
      const defaultName = (col?.name ?? "collection").replace(/[^a-z0-9_-]/gi, "_");
      const savePath = await dialogSave({
        defaultPath: `${defaultName}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!savePath) return;
      const json = await api.exportCollectionToPostman(collectionId);
      await api.saveFile(savePath, json, false);
    } catch (e: unknown) {
      console.error(e);
      alert(`Export failed: ${e instanceof Error ? e.message : String(e)}`);
    }
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

    const overData = over.data.current as any;

    // col-end drop zones
    if (overData?.type === "col-end") {
      setDropState({ overId: String(over.id), position: "after" });
      return;
    }

    const overItem = overData as TreeItem | undefined;
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

    // Guard: suppress drop indicator for folder moves that would create a circular reference.
    // This covers both "inside" (newParent = overItem.id) and "before"/"after"
    // (newParent = overItem.parentFolderId — which could be inside the dragged folder's subtree).
    const activeItemData = active.data.current as TreeItem | undefined;
    if (activeItemData?.type === "folder") {
      const potentialParent = position === "inside" && overItem.type === "folder"
        ? overItem.id
        : overItem.parentFolderId;
      if (wouldCreateCycle(activeItemData.id, potentialParent)) {
        setDropState(null);
        return;
      }
    }

    setDropState({ overId: String(over.id), position });
  }, [wouldCreateCycle]);

  const handleDragEnd = useCallback(async (event: DragEndEvent) => {
    const ds = dropState;
    setActiveDragId(null);
    setDropState(null);
    if (!ds) return;

    const activeItem = event.active.data.current as TreeItem | undefined;
    if (!activeItem) return;

    // Handle drop onto collection-end zone → move to collection root
    if (ds.overId.startsWith("col-end-")) {
      const colId = ds.overId.slice("col-end-".length);
      if (activeItem.type === "request") {
        const req = activeItem.request!;
        await api.moveRequest(req.id, colId, null);
        const sibs = requests
          .filter(r => r.collection_id === colId && !r.folder_id && r.id !== req.id)
          .sort((a, b) => a.sort_order - b.sort_order);
        await api.reorderRequests([...sibs.map(r => r.id), req.id]);
      }
      onDataChange();
      return;
    }

    const overItem   = flatItems.find(i => i.id === ds.overId);
    if (!overItem || activeItem.id === overItem.id) return;

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
        // Safety guard — should normally be prevented by handleDragMove, but double-check
        if (wouldCreateCycle(fld.id, newParent)) return;
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
  }, [dropState, flatItems, requests, folders, wouldCreateCycle, onDataChange]);

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
      className="flex-1 bg-transparent outline-none text-sm text-gray-200"
      style={{ border: "none", borderBottom: "1px solid #3b82f6", borderRadius: 0, padding: "1px 3px" }}
      autoFocus
      onClick={e => e.stopPropagation()}
      onPointerDown={e => e.stopPropagation()}
    />
  );

  const renderFolder = (folder: FolderType, depth: number): React.ReactNode => {
    const isCollapsed = collapsed.has(folder.id);
    const item = flatItems.find(i => i.id === folder.id);
    const pl = depth * 12 + 8;

    const subfolders = folders.filter(f => f.parent_folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order);
    const folderReqs = requests.filter(r => r.folder_id === folder.id).sort((a, b) => a.sort_order - b.sort_order);

    const content = item ? (
      <DnDRow key={folder.id} item={item} dropState={dropState}>
        {(isDropInside) => (
          <div
            className={`flex items-center gap-2 px-2 py-1.5 cursor-pointer text-sm rounded-md mx-1.5 transition-colors group ${
              isDropInside ? "bg-blue-500/15 ring-1 ring-blue-500/40" : "hover:bg-gray-800"
            }`}
            style={{ paddingLeft: pl }}
            onClick={e => { e.stopPropagation(); toggle(folder.id); }}
            onContextMenu={e => handleContextMenu(e, "folder", folder.id)}
            onDoubleClick={e => { e.stopPropagation(); setRenaming({ type: "folder", id: folder.id, value: folder.name }); }}
          >
            <span className={`shrink-0 ${isDropInside ? "text-blue-400" : "text-gray-500"}`}>
              {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
            </span>
            <span className={`shrink-0 ${isDropInside ? "text-blue-400" : "text-amber-400"}`}>
              {isCollapsed ? <Folder size={14} /> : <FolderOpen size={14} />}
            </span>
            {renaming?.id === folder.id
              ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
              : <span className="truncate flex-1 text-gray-400">{folder.name}</span>
            }
          </div>
        )}
      </DnDRow>
    ) : null;

    return (
      <div key={folder.id}>
        {content}
        {!isCollapsed && (
          <div className="relative">
            <div className="absolute top-0 bottom-2 w-px bg-gray-700/40" style={{ left: depth * 12 + 21 }} />
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
    const pl = depth * 12 + 8;
    const method = meta?.method ?? "GET";
    const colors = METHOD_STYLES[method];

    if (!item) return null;

    return (
      <DnDRow key={req.id} item={item} dropState={dropState}>
        {(_isDropInside) => (
          <div
            className={`flex items-center gap-2 px-2 py-1.5 cursor-pointer text-sm rounded-md mx-1.5 transition-colors group ${
              isSelected ? "bg-blue-500/10 text-blue-400" : "text-gray-400 hover:bg-gray-800 hover:text-gray-200"
            }`}
            style={{ paddingLeft: pl }}
            onClick={e => { e.stopPropagation(); onSelectRequest(req); }}
            onContextMenu={e => handleContextMenu(e, "request", req.id)}
            onDoubleClick={e => { e.stopPropagation(); setRenaming({ type: "request", id: req.id, value: req.name }); }}
          >
            {/* Spacer to align with folder chevrons */}
            <span className="w-3.5 shrink-0" />
            <FileText size={13} className={`shrink-0 ${isSelected ? "text-blue-400" : "text-gray-500"}`} />
            {renaming?.id === req.id
              ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
              : <span className="truncate flex-1">{req.name}</span>
            }
            {/* Always-visible plain-text method badge */}
            <span className={`text-[10px] font-mono font-semibold shrink-0 ${colors.text}`}>
              {method}
            </span>
          </div>
        )}
      </DnDRow>
    );
  };

  // Active drag item for DragOverlay
  const activeDragItem = activeDragId ? flatItems.find(i => i.id === activeDragId) : null;

  // ── Render ───────────────────────────────────────────────────

  return (
    <div className="h-full flex flex-col bg-[#161616]">
      {/* Header */}
      <div className="h-12 border-b border-gray-800 flex items-center px-4 justify-between shrink-0">
        <span className="font-semibold text-sm text-gray-200">Collections</span>
        <div className="flex items-center gap-2">
          <button
            onClick={handleImportPostman}
            title="Import Postman collection"
            className="text-gray-400 hover:text-gray-200 transition-colors"
          >
            <Upload size={15} />
          </button>
          <button
            onClick={handleNewCollection}
            title="New collection"
            className="text-gray-400 hover:text-gray-200 transition-colors"
          >
            <Plus size={16} />
          </button>
        </div>
      </div>

      {/* Tree */}
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragMove={handleDragMove}
        onDragEnd={handleDragEnd}
        onDragCancel={handleDragCancel}
      >
        <div className="flex-1 overflow-y-auto p-2 flex flex-col gap-0.5" onClick={closeCtx}>
          {collections.map(col => {
            const isCollapsed = collapsed.has(col.id);
            const isSelected  = col.id === selectedCollectionId;
            const colFolders  = folders.filter(f => f.collection_id === col.id && !f.parent_folder_id).sort((a, b) => a.sort_order - b.sort_order);
            const orphans     = requests.filter(r => r.collection_id === col.id && !r.folder_id).sort((a, b) => a.sort_order - b.sort_order);
            const reqCount    = requests.filter(r => r.collection_id === col.id).length;

            return (
              <div key={col.id} className="mb-1 relative">
                {/* Guide line from chevron center down through all children */}
                {!isCollapsed && (
                  <div className="absolute w-px bg-gray-700/40 pointer-events-none" style={{ left: 21, top: 16, bottom: 8 }} />
                )}
                {/* Collection header */}
                <div
                  className={`flex items-center gap-2 px-2 py-1.5 cursor-pointer rounded-md mx-1.5 transition-colors ${
                    isSelected ? "bg-[#242424]" : "hover:bg-gray-800"
                  }`}
                  onClick={() => toggle(col.id)}
                  onContextMenu={e => handleContextMenu(e, "collection", col.id)}
                  onDoubleClick={() => onSelectCollection(col.id)}
                >
                  <span className="shrink-0 text-gray-500">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                  </span>
                  <Folder size={14} className="shrink-0 text-violet-400" />
                  {renaming?.id === col.id
                    ? renameInput(renaming.value, v => setRenaming({ ...renaming, value: v }))
                    : <span className="truncate flex-1 text-sm font-medium text-gray-200">{col.name}</span>
                  }
                  <span className="text-[10px] bg-gray-800 text-gray-500 px-1.5 py-0.5 rounded-full tabular-nums shrink-0">
                    {reqCount}
                  </span>
                </div>

                {!isCollapsed && (
                  <div className="mt-0.5">
                    {colFolders.map(f => renderFolder(f, 1))}
                    {orphans.map(r => renderRequest(r, 1))}
                    <CollectionEndZone collectionId={col.id} dropState={dropState} />
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Floating drag preview */}
        <DragOverlay dropAnimation={null}>
          {activeDragItem && (() => {
            const meta = activeDragItem.request ? requestMeta.get(activeDragItem.request.id) : undefined;
            const method = meta?.method ?? "GET";
            const colors = METHOD_STYLES[method];
            return (
              <div className="flex items-center gap-2 px-3 py-1.5 bg-[#242424] border border-gray-700 rounded-md opacity-80 shadow-xl pointer-events-none">
                {activeDragItem.type === "folder" ? (
                  <>
                    <FolderOpen size={14} className="text-amber-400 shrink-0" />
                    <span className="text-sm text-gray-300">{activeDragItem.folder?.name}</span>
                  </>
                ) : (
                  <>
                    {colors && (
                      <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono font-semibold border shrink-0 ${colors.bg} ${colors.text} ${colors.border}`}>
                        {method}
                      </span>
                    )}
                    <span className="text-sm text-gray-300">{activeDragItem.request?.name}</span>
                  </>
                )}
              </div>
            );
          })()}
        </DragOverlay>
      </DndContext>

      {/* Context menu */}
      {contextMenu && (
        <div className="fixed z-50 rounded-lg shadow-2xl overflow-hidden bg-[#1a1a1a] border border-gray-700 py-1" style={{ left: contextMenu.x, top: contextMenu.y, minWidth: 180 }}>
          {contextMenu.type === "collection" && (<>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => handleNewRequest(contextMenu.id)}>New Request</button>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => handleNewFolder(contextMenu.id)}>New Folder</button>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { onSelectCollection(contextMenu.id); closeCtx(); }}>Settings</button>
            <div className="my-1 border-t border-gray-800" />
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100 flex items-center gap-2" onClick={() => handleExportCollection(contextMenu.id)}>
              <Download size={13} className="opacity-60" /> Export as Postman
            </button>
            <div className="my-1 border-t border-gray-800" />
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-3 py-2 text-sm text-red-400 hover:bg-[#242424] hover:text-red-300" onClick={() => handleDelete("collection", contextMenu.id)}>Delete</button>
          </>)}
          {contextMenu.type === "folder" && (<>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewRequest(f.collection_id, f.id); }}>New Request</button>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewFolder(f.collection_id, f.id); }}>New Subfolder</button>
            <div className="my-1 border-t border-gray-800" />
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-3 py-2 text-sm text-red-400 hover:bg-[#242424] hover:text-red-300" onClick={() => handleDelete("folder", contextMenu.id)}>Delete</button>
          </>)}
          {contextMenu.type === "request" && (<>
            <button className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-[#242424] hover:text-gray-100" onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeCtx(); }}>Rename</button>
            <button className="w-full text-left px-3 py-2 text-sm text-red-400 hover:bg-[#242424] hover:text-red-300" onClick={() => handleDelete("request", contextMenu.id)}>Delete</button>
          </>)}
        </div>
      )}
    </div>
  );
}
