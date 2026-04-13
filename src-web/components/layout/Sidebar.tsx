import { useState, useRef, useMemo } from "react";
import { ChevronRight, ChevronDown, Folder, FolderOpen, Package, GripVertical } from "lucide-react";
import {
  DndContext, closestCenter, PointerSensor, useSensor, useSensors, DragOverlay,
} from "@dnd-kit/core";
import type { DragEndEvent, DragStartEvent } from "@dnd-kit/core";
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

// ── Flat tree item ────────────────────────────────────────────
type TreeItem = {
  id: string;
  type: "folder" | "request";
  depth: number;
  parentFolderId: string | null;
  collectionId: string;
  folder?: FolderType;
  request?: Request;
};

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
      items.push({
        id: folder.id, type: "folder", depth,
        parentFolderId: folder.parent_folder_id,
        collectionId: folder.collection_id,
        folder,
      });
      if (!collapsed.has(folder.id)) {
        const subs = folders
          .filter(f => f.parent_folder_id === folder.id)
          .sort((a, b) => a.sort_order - b.sort_order);
        const reqs = requests
          .filter(r => r.folder_id === folder.id)
          .sort((a, b) => a.sort_order - b.sort_order);
        subs.forEach(f => addFolder(f, depth + 1));
        reqs.forEach(r => items.push({
          id: r.id, type: "request", depth: depth + 1,
          parentFolderId: folder.id, collectionId: folder.collection_id, request: r,
        }));
      }
    };

    folders
      .filter(f => f.collection_id === col.id && !f.parent_folder_id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(f => addFolder(f, 0));

    requests
      .filter(r => r.collection_id === col.id && !r.folder_id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(r => items.push({
        id: r.id, type: "request", depth: 0,
        parentFolderId: null, collectionId: col.id, request: r,
      }));
  }

  return items;
}

// ── Sortable row for both folders and requests ────────────────
function SortableRow({ item, children }: { item: TreeItem; children: React.ReactNode }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: item.id });
  return (
    <div
      ref={setNodeRef}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0.35 : 1,
      }}
      {...attributes}
      {...listeners}
    >
      {children}
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
  const [activeDragId, setActiveDragId] = useState<string | null>(null);
  const renameRef = useRef<HTMLInputElement>(null);

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 8 } }));

  const flatItems = useMemo(
    () => buildFlatTree(collections, folders, requests, collapsed),
    [collections, folders, requests, collapsed],
  );
  const sortableIds = flatItems.map(i => i.id);

  const toggle = (id: string) => {
    setCollapsed(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };

  const handleContextMenu = (e: React.MouseEvent, type: string, id: string) => {
    e.preventDefault();
    e.stopPropagation();
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
      id: crypto.randomUUID(), name: "New Collection", base_path: "",
      auth_config: null, headers_config: null,
      created_at: new Date().toISOString(), updated_at: new Date().toISOString(),
    };
    try {
      await api.insertCollection(col);
      onDataChange();
      setRenaming({ type: "collection", id: col.id, value: col.name });
    } catch (e) { console.error(e); }
  };

  const handleNewFolder = async (collectionId: string, parentFolderId?: string) => {
    const folder: FolderType = {
      id: crypto.randomUUID(), collection_id: collectionId,
      parent_folder_id: parentFolderId ?? null, name: "New Folder",
      path_prefix: "", auth_override: null, sort_order: 0,
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
      id: crypto.randomUUID(), collection_id: collectionId,
      folder_id: folderId ?? null, name: "New Request",
      current_version_id: null, sort_order: 0,
    };
    try {
      await api.insertRequest(req);
      onDataChange();
      setRenaming({ type: "request", id: req.id, value: req.name });
    } catch (e) { console.error(e); }
    closeContextMenu();
  };

  const handleDragStart = (event: DragStartEvent) => {
    setActiveDragId(String(event.active.id));
  };

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    setActiveDragId(null);
    if (!over || active.id === over.id) return;

    const activeItem = flatItems.find(i => i.id === String(active.id));
    const overItem = flatItems.find(i => i.id === String(over.id));
    if (!activeItem || !overItem) return;

    if (activeItem.type === "request") {
      const activeReq = activeItem.request!;

      // Determine new parent from drop target
      let newFolderId: string | null;
      let newCollectionId: string;

      if (overItem.type === "folder") {
        // Drop onto folder header → move into that folder
        newFolderId = overItem.id;
        newCollectionId = overItem.collectionId;
      } else {
        // Drop onto request → same parent as that request
        newFolderId = overItem.parentFolderId;
        newCollectionId = overItem.collectionId;
      }

      const needsMove =
        activeReq.folder_id !== newFolderId ||
        activeReq.collection_id !== newCollectionId;

      if (needsMove) {
        await api.moveRequest(activeReq.id, newCollectionId, newFolderId);
      }

      // Reorder siblings in the target group
      const siblings = requests
        .filter(r => r.collection_id === newCollectionId && r.folder_id === newFolderId)
        .sort((a, b) => a.sort_order - b.sort_order);

      const ids = siblings.map(r => r.id);
      // Include the active request if it was just moved in
      if (!ids.includes(activeReq.id)) ids.push(activeReq.id);

      const oldIdx = ids.indexOf(activeReq.id);
      const newIdx = overItem.type === "folder"
        ? ids.length - 1
        : Math.max(0, ids.indexOf(overItem.id));

      if (oldIdx !== newIdx) {
        ids.splice(oldIdx, 1);
        ids.splice(newIdx, 0, activeReq.id);
      }
      await api.reorderRequests(ids);

    } else if (activeItem.type === "folder") {
      const activeFolder = activeItem.folder!;

      // Determine new parent from drop target
      let newParentId: string | null;
      let newCollectionId: string;

      if (overItem.type === "folder") {
        // Dropping onto another folder: become sibling of it (same parent), not a child
        newParentId = overItem.parentFolderId;
        newCollectionId = overItem.collectionId;
      } else {
        // Drop onto a request: same folder level as that request's parent
        newParentId = overItem.parentFolderId;
        newCollectionId = overItem.collectionId;
      }

      const needsMove =
        activeFolder.parent_folder_id !== newParentId ||
        activeFolder.collection_id !== newCollectionId;

      if (needsMove) {
        await api.moveFolder(activeFolder.id, newCollectionId, newParentId);
      }

      // Reorder sibling folders
      const siblingFolders = folders
        .filter(f =>
          f.collection_id === newCollectionId &&
          f.parent_folder_id === newParentId &&
          f.id !== activeFolder.id
        )
        .sort((a, b) => a.sort_order - b.sort_order);

      const ids = siblingFolders.map(f => f.id);
      const overIdx = overItem.type === "folder" ? ids.indexOf(overItem.id) : ids.length;
      ids.splice(Math.max(0, overIdx), 0, activeFolder.id);
      await api.reorderFolders(ids);
    }

    onDataChange();
  };

  // ── Render helpers ─────────────────────────────────────────
  const renderFolderRow = (folder: FolderType, depth: number) => {
    const isCollapsed = collapsed.has(folder.id);
    const paddingLeft = 28 + depth * 20;
    const item = flatItems.find(i => i.id === folder.id);

    const row = (
      <div
        className="flex items-center gap-2 py-2.5 cursor-pointer hover:bg-[var(--surface-2)] transition-colors group"
        style={{ paddingLeft, paddingRight: 10, userSelect: "none" }}
        onClick={(e) => { e.stopPropagation(); toggle(folder.id); }}
        onContextMenu={(e) => handleContextMenu(e, "folder", folder.id)}
        onDoubleClick={(e) => { e.stopPropagation(); setRenaming({ type: "folder", id: folder.id, value: folder.name }); }}
      >
        <GripVertical size={13} className="flex-shrink-0 opacity-0 group-hover:opacity-30" style={{ color: "var(--text-muted)", cursor: "grab" }} />
        <span className="flex-shrink-0" style={{ color: "var(--text-muted)" }}>
          {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
        </span>
        <span style={{ color: "var(--text-muted)" }}>
          {isCollapsed ? <Folder size={15} /> : <FolderOpen size={15} />}
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
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="truncate flex-1 text-sm" style={{ color: "var(--text-secondary)" }}>
            {folder.name}
          </span>
        )}
      </div>
    );

    return item ? <SortableRow key={folder.id} item={item}>{row}</SortableRow> : row;
  };

  const renderRequestRow = (req: Request, depth: number) => {
    const isSelected = req.id === selectedRequestId;
    const meta = requestMeta.get(req.id);
    const paddingLeft = 28 + depth * 20;
    const item = flatItems.find(i => i.id === req.id);

    const row = (
      <div
        className="flex items-center gap-2 py-2.5 cursor-pointer transition-colors group"
        style={{
          paddingLeft, paddingRight: 10,
          background: isSelected ? "var(--surface-2)" : "transparent",
          borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
          userSelect: "none",
        }}
        onClick={(e) => { e.stopPropagation(); onSelectRequest(req); }}
        onContextMenu={(e) => handleContextMenu(e, "request", req.id)}
        onDoubleClick={(e) => { e.stopPropagation(); setRenaming({ type: "request", id: req.id, value: req.name }); }}
        onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
        onMouseLeave={(e) => { if (!isSelected) e.currentTarget.style.background = "transparent"; }}
      >
        <GripVertical size={13} className="flex-shrink-0 opacity-0 group-hover:opacity-30" style={{ color: "var(--text-muted)", cursor: "grab" }} />
        {meta ? (
          <span
            className="font-mono text-xs font-bold flex-shrink-0"
            style={{ color: methodColor(meta.method), width: 36, textAlign: "right" }}
          >
            {meta.method.length > 3 ? meta.method.slice(0, 3) : meta.method}
          </span>
        ) : (
          <span style={{ width: 36 }} />
        )}
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
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span className="truncate flex-1 text-sm">{req.name}</span>
        )}
      </div>
    );

    return item ? <SortableRow key={req.id} item={item}>{row}</SortableRow> : row;
  };

  const renderFolder = (folder: FolderType, depth: number) => {
    const isCollapsed = collapsed.has(folder.id);
    const subfolders = folders
      .filter(f => f.parent_folder_id === folder.id)
      .sort((a, b) => a.sort_order - b.sort_order);
    const folderRequests = requests
      .filter(r => r.folder_id === folder.id)
      .sort((a, b) => a.sort_order - b.sort_order);

    return (
      <div key={folder.id}>
        {renderFolderRow(folder, depth)}
        {!isCollapsed && (
          <div>
            {subfolders.map(f => renderFolder(f, depth + 1))}
            {folderRequests.map(r => renderRequestRow(r, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  // Find the active drag item for DragOverlay
  const activeDragItem = activeDragId ? flatItems.find(i => i.id === activeDragId) : null;

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

      {/* Tree with single DndContext + single SortableContext */}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
      >
        <SortableContext items={sortableIds} strategy={verticalListSortingStrategy}>
          <div className="flex-1 overflow-y-auto py-1.5" onClick={closeContextMenu}>
            {collections.map(col => {
              const isCollapsed = collapsed.has(col.id);
              const isSelected = col.id === selectedCollectionId;
              const colFolders = folders
                .filter(f => f.collection_id === col.id && !f.parent_folder_id)
                .sort((a, b) => a.sort_order - b.sort_order);
              const orphanRequests = requests
                .filter(r => r.collection_id === col.id && !r.folder_id)
                .sort((a, b) => a.sort_order - b.sort_order);

              return (
                <div key={col.id}>
                  {/* Collection header — not draggable */}
                  <div
                    className="flex items-center gap-2 py-3 cursor-pointer transition-colors"
                    style={{
                      paddingLeft: 10, paddingRight: 10,
                      background: isSelected ? "var(--surface-2)" : "transparent",
                      borderBottom: "1px solid var(--border-subtle)",
                    }}
                    onClick={() => toggle(col.id)}
                    onContextMenu={(e) => handleContextMenu(e, "collection", col.id)}
                    onDoubleClick={() => onSelectCollection(col.id)}
                    onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                    onMouseLeave={(e) => { e.currentTarget.style.background = isSelected ? "var(--surface-2)" : "transparent"; }}
                  >
                    <span className="flex-shrink-0" style={{ color: "var(--text-muted)" }}>
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
                      {orphanRequests.map(r => renderRequestRow(r, 0))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </SortableContext>

        {/* Drag preview overlay */}
        <DragOverlay>
          {activeDragItem && (
            <div
              className="flex items-center gap-2 rounded text-sm px-3 py-2 shadow-lg"
              style={{
                background: "var(--surface-3)",
                border: "1px solid var(--border)",
                opacity: 0.9,
                minWidth: 160,
              }}
            >
              {activeDragItem.type === "folder" ? (
                <>
                  <Folder size={14} style={{ color: "var(--text-muted)" }} />
                  <span>{activeDragItem.folder?.name}</span>
                </>
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
                onClick={() => handleNewRequest(contextMenu.id)}>New Request</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => handleNewFolder(contextMenu.id)}>New Folder</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { onSelectCollection(contextMenu.id); closeContextMenu(); }}>Settings</button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "collection", id: contextMenu.id, value: collections.find(c => c.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>Rename</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("collection", contextMenu.id)}>Delete</button>
            </>
          )}
          {contextMenu.type === "folder" && (
            <>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewRequest(f.collection_id, f.id); }}>New Request</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { const f = folders.find(ff => ff.id === contextMenu.id); if (f) handleNewFolder(f.collection_id, f.id); }}>New Subfolder</button>
              <div className="my-0.5" style={{ borderTop: "1px solid var(--border)" }} />
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "folder", id: contextMenu.id, value: folders.find(f => f.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>Rename</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("folder", contextMenu.id)}>Delete</button>
            </>
          )}
          {contextMenu.type === "request" && (
            <>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors"
                onClick={() => { setRenaming({ type: "request", id: contextMenu.id, value: requests.find(r => r.id === contextMenu.id)?.name ?? "" }); closeContextMenu(); }}>Rename</button>
              <button className="w-full text-left px-4 py-2.5 hover:bg-[var(--surface-3)] transition-colors" style={{ color: "var(--danger)" }}
                onClick={() => handleDelete("request", contextMenu.id)}>Delete</button>
            </>
          )}
        </div>
      )}
    </div>
  );
}

