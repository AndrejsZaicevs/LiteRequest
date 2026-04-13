import { useState, useEffect, useCallback, useRef } from "react";
import type {
  Collection, Folder, Request, RequestVersion, RequestExecution,
  Environment, EnvVariable, RequestData, ResponseData, HttpMethod,
} from "./lib/types";
import { defaultRequestData } from "./lib/types";
import * as api from "./lib/api";
import { TopBar } from "./components/layout/TopBar";
import { Sidebar } from "./components/layout/Sidebar";
import { Inspector } from "./components/layout/Inspector";
import { RequestEditor } from "./components/editor/RequestEditor";
import { ResponseView } from "./components/response/ResponseView";
import { CollectionConfig } from "./components/settings/CollectionConfig";
import { AppSettings } from "./components/settings/AppSettings";
import { GlobalSearch } from "./components/search/GlobalSearch";

export type CenterView =
  | { type: "welcome" }
  | { type: "request"; requestId: string }
  | { type: "collection"; collectionId: string }
  | { type: "settings" };

export default function App() {
  // ── Data caches ──────────────────────────────────────────
  const [collections, setCollections] = useState<Collection[]>([]);
  const [folders, setFolders] = useState<Folder[]>([]);
  const [requests, setRequests] = useState<Request[]>([]);
  const [versions, setVersions] = useState<RequestVersion[]>([]);
  const [executions, setExecutions] = useState<RequestExecution[]>([]);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [envVariables, setEnvVariables] = useState<EnvVariable[]>([]);

  // ── Selection state ──────────────────────────────────────
  const [centerView, setCenterView] = useState<CenterView>({ type: "welcome" });
  const [currentRequest, setCurrentRequest] = useState<Request | null>(null);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null);
  const [selectedExecutionId, setSelectedExecutionId] = useState<string | null>(null);

  // Cache of request method/url from their current version (for sidebar display)
  const [requestMeta, setRequestMeta] = useState<Map<string, { method: HttpMethod; url: string }>>(new Map());

  // ── Editor state ─────────────────────────────────────────
  const [editorData, setEditorData] = useState<RequestData>(defaultRequestData());
  const [dirty, setDirty] = useState(false);

  // ── Response state ───────────────────────────────────────
  const [currentResponse, setCurrentResponse] = useState<ResponseData | null>(null);
  const [currentLatency, setCurrentLatency] = useState<number>(0);
  const [isLoading, setIsLoading] = useState(false);

  // ── Panel sizing ─────────────────────────────────────────
  const [sidebarWidth, setSidebarWidth] = useState(240);
  const [inspectorWidth, setInspectorWidth] = useState(280);
  const [splitRatio, setSplitRatio] = useState(0.5);

  // ── Search ───────────────────────────────────────────────
  const [searchOpen, setSearchOpen] = useState(false);

  // ── Error/status ─────────────────────────────────────────
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // ── Refs for drag tracking ───────────────────────────────
  const dragging = useRef<"sidebar" | "inspector" | "split" | null>(null);

  // ── Data loading ─────────────────────────────────────────
  const refreshAll = useCallback(async () => {
    try {
      const [cols, envs] = await Promise.all([
        api.listCollections(),
        api.listEnvironments(),
      ]);
      setCollections(cols);
      setEnvironments(envs);

      // Load all folders and requests for all collections
      const allFolders: Folder[] = [];
      const allRequests: Request[] = [];
      for (const c of cols) {
        const [f, r] = await Promise.all([
          api.listFolders(c.id),
          api.listRequestsByCollection(c.id),
        ]);
        allFolders.push(...f);
        allRequests.push(...r);
      }
      setFolders(allFolders);
      setRequests(allRequests);

      // Load current version data for all requests so sidebar shows methods
      const metaMap = new Map<string, { method: HttpMethod; url: string }>();
      await Promise.all(
        allRequests
          .filter(r => r.current_version_id)
          .map(async (req) => {
            try {
              const v = await api.getVersion(req.current_version_id!);
              metaMap.set(req.id, { method: v.data.method, url: v.data.url });
            } catch { /* ignore missing versions */ }
          })
      );
      setRequestMeta(metaMap);

      // Load active env variables
      const activeEnv = envs.find(e => e.is_active);
      if (activeEnv) {
        const vars = await api.listEnvVariables(activeEnv.id);
        setEnvVariables(vars);
      } else {
        setEnvVariables([]);
      }
    } catch (e) {
      setErrorMessage(String(e));
    }
  }, []);

  useEffect(() => { refreshAll(); }, [refreshAll]);

  // ── Request selection ────────────────────────────────────
  const selectRequest = useCallback(async (req: Request) => {
    // Auto-save current if dirty
    if (dirty && currentRequest) {
      await saveCurrentVersion();
    }

    setCurrentRequest(req);
    setCenterView({ type: "request", requestId: req.id });

    try {
      const [vers, execs] = await Promise.all([
        api.listVersions(req.id),
        api.listExecutions(req.id),
      ]);
      setVersions(vers);
      setExecutions(execs);

      // Load current version data
      if (req.current_version_id) {
        const v = await api.getVersion(req.current_version_id);
        setEditorData(v.data);
        setSelectedVersionId(v.id);
        // Cache method/url for sidebar display
        setRequestMeta(prev => new Map(prev).set(req.id, { method: v.data.method, url: v.data.url }));
      } else {
        setEditorData(defaultRequestData());
        setSelectedVersionId(null);
      }
      setDirty(false);
      setSelectedExecutionId(null);
      setCurrentResponse(null);
    } catch (e) {
      setErrorMessage(String(e));
    }
  }, [dirty, currentRequest]);

  // ── Save version ─────────────────────────────────────────
  const saveCurrentVersion = useCallback(async () => {
    if (!currentRequest) return;
    const now = new Date().toISOString();
    const versionId = crypto.randomUUID();

    // Check if latest version has no executions — update in place
    if (selectedVersionId) {
      const hasExec = await api.versionHasExecutions(selectedVersionId);
      if (!hasExec) {
        await api.updateVersionData(selectedVersionId, editorData, now);
        setDirty(false);
        // Update sidebar meta for this request
        setRequestMeta(prev => new Map(prev).set(currentRequest.id, { method: editorData.method, url: editorData.url }));
        const vers = await api.listVersions(currentRequest.id);
        setVersions(vers);
        return;
      }
    }

    // Check for dedup — don't create if identical to current
    if (selectedVersionId) {
      try {
        const current = await api.getVersion(selectedVersionId);
        if (JSON.stringify(current.data) === JSON.stringify(editorData)) {
          setDirty(false);
          return;
        }
      } catch { /* ignore */ }
    }

    const version: RequestVersion = {
      id: versionId,
      request_id: currentRequest.id,
      data: editorData,
      created_at: now,
    };
    await api.insertVersion(version);
    // Link this version as the request's current version
    await api.updateRequestVersion(currentRequest.id, versionId);
    setSelectedVersionId(versionId);
    setDirty(false);
    // Update sidebar meta
    setRequestMeta(prev => new Map(prev).set(currentRequest.id, { method: editorData.method, url: editorData.url }));
    // Update local currentRequest so switching away and back works
    setCurrentRequest(prev => prev ? { ...prev, current_version_id: versionId } : prev);
    setRequests(prev => prev.map(r => r.id === currentRequest.id ? { ...r, current_version_id: versionId } : r));

    const vers = await api.listVersions(currentRequest.id);
    setVersions(vers);
  }, [currentRequest, selectedVersionId, editorData]);

  // ── Execute request ──────────────────────────────────────
  const sendRequest = useCallback(async () => {
    if (!currentRequest) return;

    // Save first
    await saveCurrentVersion();

    setIsLoading(true);
    setErrorMessage(null);
    try {
      // Build variables
      const variables: Record<string, string> = {};
      // Global env variables
      for (const v of envVariables) {
        variables[v.key] = v.value;
      }
      // Collection variables
      const colVars = await api.getActiveCollectionVariables(currentRequest.collection_id);
      for (const [k, v] of colVars) {
        variables[k] = v;
      }
      // Collection name as variable
      const col = collections.find(c => c.id === currentRequest.collection_id);
      if (col) variables["collectionName"] = col.name;

      const basePath = col?.base_path ?? "";

      // Get client certs from settings
      let clientCerts: import("./lib/types").ClientCertEntry[] = [];
      try {
        const certsJson = await api.getAppSetting("client_certs");
        if (certsJson) clientCerts = JSON.parse(certsJson);
      } catch { /* ignore */ }

      const [response, latency] = await api.executeRequest(
        editorData, variables, basePath, clientCerts,
      );
      setCurrentResponse(response);
      setCurrentLatency(latency);

      // Save execution
      const activeEnv = environments.find(e => e.is_active);
      const execution: RequestExecution = {
        id: crypto.randomUUID(),
        version_id: selectedVersionId ?? "",
        request_id: currentRequest.id,
        environment_id: activeEnv?.id ?? "",
        response,
        latency_ms: latency,
        executed_at: new Date().toISOString(),
      };
      await api.insertExecution(execution);
      const execs = await api.listExecutions(currentRequest.id);
      setExecutions(execs);
      setSelectedExecutionId(execution.id);
    } catch (e) {
      setErrorMessage(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [currentRequest, editorData, envVariables, collections, environments, selectedVersionId, saveCurrentVersion]);

  // ── Editor data change ───────────────────────────────────
  const onEditorChange = useCallback((data: RequestData) => {
    setEditorData(data);
    setDirty(true);
  }, []);

  // ── Auto-save on modification (debounced) ────────────────
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    if (!dirty || !currentRequest) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      saveCurrentVersion();
    }, 500);
    return () => { if (saveTimer.current) clearTimeout(saveTimer.current); };
  }, [dirty, editorData, currentRequest, saveCurrentVersion]);

  // ── Keyboard shortcuts ───────────────────────────────────
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen(prev => !prev);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  // ── Panel drag handlers ──────────────────────────────────
  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!dragging.current) return;
      e.preventDefault();
      if (dragging.current === "sidebar") {
        setSidebarWidth(Math.max(180, Math.min(400, e.clientX)));
      } else if (dragging.current === "inspector") {
        setInspectorWidth(Math.max(200, Math.min(500, window.innerWidth - e.clientX)));
      } else if (dragging.current === "split") {
        const mainLeft = sidebarWidth;
        const mainWidth = window.innerWidth - sidebarWidth - inspectorWidth;
        const ratio = (e.clientX - mainLeft) / mainWidth;
        setSplitRatio(Math.max(0.2, Math.min(0.8, ratio)));
      }
    };
    const onMouseUp = () => {
      dragging.current = null;
      document.body.classList.remove("resizing");
    };
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
  }, [sidebarWidth, inspectorWidth]);

  const startDrag = (panel: "sidebar" | "inspector" | "split") => {
    dragging.current = panel;
    document.body.classList.add("resizing");
  };

  // ── Render ───────────────────────────────────────────────
  const showInspector = centerView.type === "request";
  const mainWidth = window.innerWidth - sidebarWidth - (showInspector ? inspectorWidth : 0);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#121212]">
      <TopBar
        environments={environments}
        onEnvChange={async (id) => {
          if (id) {
            await api.setActiveEnvironment(id);
          }
          await refreshAll();
        }}
        onSearch={() => setSearchOpen(true)}
        onSettings={() => setCenterView({ type: "settings" })}
        errorMessage={errorMessage}
        statusMessage={statusMessage}
        isLoading={isLoading}
      />

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <div style={{ width: sidebarWidth, minWidth: sidebarWidth }} className="flex-shrink-0 overflow-hidden">
          <Sidebar
            collections={collections}
            folders={folders}
            requests={requests}
            selectedRequestId={centerView.type === "request" ? centerView.requestId : null}
            selectedCollectionId={centerView.type === "collection" ? centerView.collectionId : null}
            requestMeta={requestMeta}
            onSelectRequest={(req) => selectRequest(req)}
            onSelectCollection={(id) => setCenterView({ type: "collection", collectionId: id })}
            onDataChange={refreshAll}
          />
        </div>

        {/* Sidebar resize handle */}
        <div
          className="w-px cursor-col-resize hover:bg-blue-500/50 transition-colors shrink-0 bg-gray-800"
          onMouseDown={() => startDrag("sidebar")}
        />

        {/* Main content */}
        <div className="flex-1 flex flex-col overflow-hidden bg-[#121212]" style={{ minWidth: 0 }}>
          {centerView.type === "welcome" && (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-center">
                <div className="text-3xl font-bold text-gray-200 mb-3">Welcome</div>
                <div className="text-sm text-gray-500">Select a request or create a new collection to get started</div>
                <div className="mt-6 flex items-center justify-center gap-2 text-xs text-gray-600">
                  <span className="bg-gray-800 px-2 py-1 rounded border border-gray-700">⌘K</span>
                  <span>to search</span>
                </div>
              </div>
            </div>
          )}

          {centerView.type === "request" && (
            <div className="flex-1 flex flex-col overflow-hidden">
              {/* Request editor (top portion) */}
              <div style={{ height: `${splitRatio * 100}%` }} className="shrink-0 overflow-hidden border-b border-gray-800">
                <RequestEditor
                  data={editorData}
                  onChange={onEditorChange}
                  onSend={sendRequest}
                  isLoading={isLoading}
                  basePath={collections.find(c => c.id === currentRequest?.collection_id)?.base_path ?? ""}
                  requestName={currentRequest?.name ?? ""}
                />
              </div>

              {/* Split drag handle */}
              <div
                className="h-px cursor-row-resize hover:bg-blue-500/50 transition-colors shrink-0 bg-gray-800"
                onMouseDown={() => startDrag("split")}
              />

              {/* Response view (bottom portion) */}
              <div className="flex-1 overflow-hidden">
                <ResponseView
                  response={currentResponse}
                  latency={currentLatency}
                  isLoading={isLoading}
                />
              </div>
            </div>
          )}

          {centerView.type === "collection" && (
            <CollectionConfig
              collectionId={centerView.collectionId}
              collections={collections}
              environments={environments}
              onUpdate={refreshAll}
            />
          )}

          {centerView.type === "settings" && (
            <AppSettings
              environments={environments}
              onUpdate={refreshAll}
            />
          )}
        </div>

        {/* Inspector resize handle */}
        {showInspector && (
          <div
            className="w-px cursor-col-resize hover:bg-blue-500/50 transition-colors shrink-0 bg-gray-800"
            onMouseDown={() => startDrag("inspector")}
          />
        )}

        {/* Inspector panel */}
        {showInspector && (
          <div style={{ width: inspectorWidth, minWidth: inspectorWidth }} className="flex-shrink-0 overflow-hidden">
            <Inspector
              data={editorData}
              onChange={onEditorChange}
              versions={versions}
              executions={executions}
              selectedVersionId={selectedVersionId}
              selectedExecutionId={selectedExecutionId}
              onSelectVersion={async (vid) => {
                setSelectedVersionId(vid);
                const v = await api.getVersion(vid);
                setEditorData(v.data);
                setDirty(false);
              }}
              onSelectExecution={(eid) => {
                setSelectedExecutionId(eid);
                const exec = executions.find(e => e.id === eid);
                if (exec) {
                  setCurrentResponse(exec.response);
                  setCurrentLatency(exec.latency_ms);
                }
              }}
              environments={environments}
            />
          </div>
        )}
      </div>

      {/* Global search modal */}
      {searchOpen && (
        <GlobalSearch
          collections={collections}
          folders={folders}
          requests={requests}
          requestMeta={requestMeta}
          onClose={() => setSearchOpen(false)}
          onSelectRequest={(req) => {
            setSearchOpen(false);
            selectRequest(req);
          }}
          onSelectCollection={(id) => {
            setSearchOpen(false);
            setCenterView({ type: "collection", collectionId: id });
          }}
        />
      )}
    </div>
  );
}
