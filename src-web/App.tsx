import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import type {
  Collection, Folder, Request, RequestVersion, RequestExecution,
  Environment, EnvVariable, RequestData, ResponseData, HttpMethod,
  KeyValuePair, AuthConfig,
} from "./lib/types";
import { defaultRequestData, resolveVariableRefs, findUnresolvedVars } from "./lib/types";
import { resolveDynamicVars, getDynamicVarPreviews } from "./lib/dynamicVars";
import * as api from "./lib/api";
import { Search, Settings, Maximize2, Minimize2 } from "lucide-react";
import { Sidebar } from "./components/layout/Sidebar";
import { Inspector } from "./components/layout/Inspector";
import { UrlBar } from "./components/editor/UrlBar";
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
  // "auto" = derive from context; "request"/"response" = user-maximized; "split" = force split
  const [splitOverride, setSplitOverride] = useState<"auto" | "request" | "response" | "split">("auto");

  // ── Search ───────────────────────────────────────────────
  const [searchOpen, setSearchOpen] = useState(false);

  // ── Error/status ─────────────────────────────────────────
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // ── Variable map for display/highlighting ────────────────
  const [collectionDisplayVars, setCollectionDisplayVars] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!currentRequest) { setCollectionDisplayVars({}); return; }
    api.getActiveCollectionVariables(currentRequest.collection_id)
      .then(vars => {
        const obj: Record<string, string> = {};
        for (const [k, v] of vars) obj[k] = v;
        setCollectionDisplayVars(obj);
      })
      .catch(() => setCollectionDisplayVars({}));
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentRequest?.collection_id, environments.find(e => e.is_active)?.id, envVariables]);

  const displayVariables = useMemo(() => {
    const vars: Record<string, string> = { ...collectionDisplayVars };
    for (const v of envVariables) vars[v.key] = v.value;
    const colName = collections.find(c => c.id === currentRequest?.collection_id)?.name;
    if (colName) vars["collectionName"] = colName;
    // Add stable preview values for dynamic variables (tooltip display only)
    const previews = getDynamicVarPreviews();
    for (const [k, v] of Object.entries(previews)) {
      if (!(k in vars)) vars[k] = v;
    }
    return resolveVariableRefs(vars);
  }, [envVariables, collectionDisplayVars, collections, currentRequest?.collection_id]);

  // ── Refs for drag tracking ───────────────────────────────
  const dragging = useRef<"sidebar" | "inspector" | "split" | null>(null);
  const splitContainerRef = useRef<HTMLDivElement>(null);

  // ── Effective split pane mode ────────────────────────────
  const noBody = editorData.body_type === "None";
  const noResponse = !currentResponse && !isLoading;
  const effectivePane: "request" | "response" | "split" = (() => {
    if (splitOverride === "request") return "request";
    if (splitOverride === "response") return "response";
    if (splitOverride === "split") return "split";
    // auto
    if (noResponse) return "request";
    if (noBody) return "response";
    return "split";
  })();

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

      // Load active env variables from new split model
      const activeEnv = envs.find(e => e.is_active);
      if (activeEnv) {
        const rows = await api.loadEnvVarRows(activeEnv.id);
        // Map VarRow → EnvVariable shape for downstream consumers
        setEnvVariables(rows.map(r => ({
          id: r.value_id ?? r.def_id,
          environment_id: activeEnv.id,
          key: r.key,
          value: r.value,
          is_secret: r.is_secret,
        })));
      } else {
        setEnvVariables([]);
      }
    } catch (e) {
      setErrorMessage(String(e));
    }
  }, []);

  useEffect(() => { refreshAll(); }, [refreshAll]);

  // ── Restore last-open request after first data load ──────
  const restoredRef = useRef(false);
  useEffect(() => {
    if (restoredRef.current || requests.length === 0) return;
    restoredRef.current = true;
    const savedId = localStorage.getItem("lr.selectedRequestId");
    if (savedId) {
      const req = requests.find(r => r.id === savedId);
      if (req) selectRequest(req);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [requests]);

  // ── Request selection ────────────────────────────────────
  const selectRequest = useCallback(async (req: Request) => {
    // Auto-save current if dirty
    if (dirty && currentRequest) {
      await saveCurrentVersion();
    }

    setCurrentRequest(req);
    setCenterView({ type: "request", requestId: req.id });
    setSplitOverride("auto"); // reset pane layout for each new request
    try { localStorage.setItem("lr.selectedRequestId", req.id); } catch { /* ignore */ }

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
  // All version logic (update-in-place vs create-new) lives in the backend.
  const saveCurrentVersion = useCallback(async () => {
    if (!currentRequest) return;

    const version = await api.saveVersion(currentRequest.id, editorData);

    // Update local state
    const changed = version.id !== selectedVersionId;
    setSelectedVersionId(version.id);
    setDirty(false);
    setRequestMeta(prev => new Map(prev).set(currentRequest.id, { method: editorData.method, url: editorData.url }));

    if (changed) {
      setCurrentRequest(prev => prev ? { ...prev, current_version_id: version.id } : prev);
      setRequests(prev => prev.map(r => r.id === currentRequest.id ? { ...r, current_version_id: version.id } : r));
    }

    const vers = await api.listVersions(currentRequest.id);
    setVersions(vers);
  }, [currentRequest, selectedVersionId, editorData]);

  // ── Navigate to request (from search) ────────────────────
  const navigateToRequest = useCallback(async (
    requestId: string,
    versionId?: string | null,
    executionId?: string | null,
    collectionId?: string | null,
  ) => {
    if (collectionId && !requestId) {
      setCenterView({ type: "collection", collectionId });
      return;
    }
    const req = requests.find(r => r.id === requestId);
    if (!req) return;

    if (dirty && currentRequest) await saveCurrentVersion();

    setCurrentRequest(req);
    setCenterView({ type: "request", requestId: req.id });
    try { localStorage.setItem("lr.selectedRequestId", req.id); } catch { /* ignore */ }

    try {
      const [vers, execs] = await Promise.all([
        api.listVersions(req.id),
        api.listExecutions(req.id),
      ]);
      setVersions(vers);
      setExecutions(execs);

      const targetVersionId = versionId ?? req.current_version_id;
      if (targetVersionId) {
        const v = await api.getVersion(targetVersionId);
        setEditorData(v.data);
        setSelectedVersionId(v.id);
        setRequestMeta(prev => new Map(prev).set(req.id, { method: v.data.method, url: v.data.url }));
      } else {
        setEditorData(defaultRequestData());
        setSelectedVersionId(null);
      }
      setDirty(false);
      setSelectedExecutionId(executionId ?? null);
      // If navigating to a specific execution, load its response and request snapshot
      if (executionId) {
        const exec = execs.find(e => e.id === executionId);
        if (exec) {
          setCurrentResponse(exec.response);
          setCurrentLatency(exec.latency_ms);
          if (exec.request_data) {
            setEditorData(exec.request_data);
          }
        }
      } else {
        setCurrentResponse(null);
      }
    } catch (e) {
      setErrorMessage(String(e));
    }
  }, [dirty, currentRequest, requests, saveCurrentVersion]);

  // ── Build effective request data (with collection auth + headers) ──
  const buildEffectiveData = useCallback((baseData: RequestData): RequestData => {
    const col = collections.find(c => c.id === currentRequest?.collection_id);
    const requestHeaderKeys = new Set(
      baseData.headers.filter(h => h.enabled && h.key).map(h => h.key.toLowerCase())
    );
    const extraHeaders: KeyValuePair[] = [];
    if (col?.headers_config) {
      try {
        const defaults = JSON.parse(col.headers_config) as KeyValuePair[];
        for (const h of defaults.filter(h => h.enabled && h.key)) {
          if (!requestHeaderKeys.has(h.key.toLowerCase())) {
            extraHeaders.push(h);
            requestHeaderKeys.add(h.key.toLowerCase());
          }
        }
      } catch { /* ignore */ }
    }
    if (col?.auth_config) {
      try {
        const auth = JSON.parse(col.auth_config) as AuthConfig;
        let authHeader: KeyValuePair | null = null;
        if (auth.auth_type === "bearer" && auth.bearer_token) {
          authHeader = { key: "Authorization", value: `Bearer ${auth.bearer_token}`, enabled: true };
        } else if (auth.auth_type === "basic") {
          const encoded = btoa(`${auth.basic_username ?? ""}:${auth.basic_password ?? ""}`);
          authHeader = { key: "Authorization", value: `Basic ${encoded}`, enabled: true };
        } else if (auth.auth_type === "api_key" && auth.api_key_value) {
          authHeader = { key: auth.api_key_header ?? "X-API-Key", value: auth.api_key_value, enabled: true };
        }
        if (authHeader && !requestHeaderKeys.has(authHeader.key.toLowerCase())) {
          extraHeaders.push(authHeader);
        }
      } catch { /* ignore */ }
    }
    return extraHeaders.length > 0
      ? { ...baseData, headers: [...extraHeaders, ...baseData.headers] }
      : baseData;
  }, [collections, currentRequest]);

  // ── Execute request ──────────────────────────────────────
  const sendRequest = useCallback(async () => {
    if (!currentRequest) return;

    // Save first
    await saveCurrentVersion();

    setIsLoading(true);
    setErrorMessage(null);
    setSplitOverride("auto"); // reset so response auto-shows after send
    try {
      // Build variables
      const variables: Record<string, string> = {};
      for (const v of envVariables) {
        variables[v.key] = v.value;
      }
      const colVars = await api.getActiveCollectionVariables(currentRequest.collection_id);
      for (const [k, v] of colVars) {
        variables[k] = v;
      }
      const col = collections.find(c => c.id === currentRequest.collection_id);
      if (col) variables["collectionName"] = col.name;

      // Inject fresh dynamic variable values ($randomInt, $randomEmail, etc.)
      const resolvedVariables = resolveVariableRefs(resolveDynamicVars(variables));

      const basePath = col?.base_path ?? "";

      const effectiveData = buildEffectiveData(editorData);

      // Block send if any {{variable}} references are unresolved
      const unresolved = findUnresolvedVars(effectiveData, basePath, resolvedVariables);
      if (unresolved.length > 0) {
        setErrorMessage(`Unresolved variables: ${unresolved.map(v => `{{${v}}}`).join(", ")}`);
        return;
      }

      // Get client certs from settings
      let clientCerts: import("./lib/types").ClientCertEntry[] = [];
      try {
        const certsJson = await api.getAppSetting("client_certs");
        if (certsJson) clientCerts = JSON.parse(certsJson);
      } catch { /* ignore */ }

      const [response, latency] = await api.executeRequest(
        effectiveData, resolvedVariables, basePath, clientCerts,
      );
      setCurrentResponse(response);
      setCurrentLatency(latency);

      // Save execution with request data snapshot
      const activeEnv = environments.find(e => e.is_active);
      const execution: RequestExecution = {
        id: crypto.randomUUID(),
        version_id: selectedVersionId ?? "",
        request_id: currentRequest.id,
        environment_id: activeEnv?.id ?? "",
        response,
        latency_ms: latency,
        executed_at: new Date().toISOString(),
        request_data: editorData,
      };
      await api.insertExecution(execution);
      const execs = await api.listExecutions(currentRequest.id);
      setExecutions(execs);
      setSelectedExecutionId(execution.id);
    } catch (e) {
      if (String(e).includes("Request cancelled")) {
        setErrorMessage(null);
      } else {
        setErrorMessage(String(e));
      }
    } finally {
      setIsLoading(false);
    }
  }, [currentRequest, editorData, envVariables, collections, environments, selectedVersionId, saveCurrentVersion, buildEffectiveData]);

  // ── Editor data change ───────────────────────────────────
  const onEditorChange = useCallback((data: RequestData) => {
    setEditorData(data);
    setDirty(true);
  }, []);

  // ── Copy as cURL ─────────────────────────────────────────
  const copyCurl = useCallback(async () => {
    if (!currentRequest) return;
    try {
      const col = collections.find(c => c.id === currentRequest.collection_id);
      const variables: Record<string, string> = {};
      for (const v of envVariables) variables[v.key] = v.value;
      const colVars = await api.getActiveCollectionVariables(currentRequest.collection_id);
      for (const [k, v] of colVars) variables[k] = v;
      if (col) variables["collectionName"] = col.name;
      const resolvedVariables = resolveVariableRefs(resolveDynamicVars(variables));
      const basePath = col?.base_path ?? "";
      const effectiveData = buildEffectiveData(editorData);
      const curlStr = await api.toCurl(effectiveData, resolvedVariables, basePath);
      await api.copyToClipboard(curlStr);
      setErrorMessage(null);
    } catch (e) {
      setErrorMessage(`Copy cURL failed: ${e}`);
    }
  }, [currentRequest, collections, envVariables, editorData, buildEffectiveData]);

  // ── Import from cURL ─────────────────────────────────────
  const importCurl = useCallback(async (curlStr: string) => {
    try {
      const parsed = await api.parseCurl(curlStr);
      setEditorData(parsed);
      setDirty(true);
    } catch (e) {
      setErrorMessage(`Import cURL failed: ${e}`);
    }
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
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        if (centerView.type === "request" && !isLoading) sendRequest();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [centerView, isLoading, sendRequest]);

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
        const container = splitContainerRef.current;
        if (!container) return;
        const rect = container.getBoundingClientRect();
        const ratio = (e.clientY - rect.top) / rect.height;
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
    <div className="flex h-screen w-screen overflow-hidden bg-[#121212] text-gray-300 font-sans">

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <div style={{ width: sidebarWidth, minWidth: sidebarWidth }} className="shrink-0 overflow-hidden border-r border-gray-800">
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
          className="w-[3px] cursor-col-resize shrink-0 bg-gray-800 hover:bg-blue-500/60 transition-colors"
          onMouseDown={() => startDrag("sidebar")}
        />

        {/* RIGHT AREA: top bar + url bar + content/inspector */}
        <div className="flex-1 flex flex-col overflow-hidden" style={{ minWidth: 0 }}>

          {/* Full-width top bar */}
          <div className="h-12 border-b border-gray-800 flex items-center px-4 gap-4 bg-[#161616] shrink-0">
            {/* Search trigger */}
            <button
              onClick={() => setSearchOpen(true)}
              className="flex-1 max-w-md mx-4 flex items-center justify-between bg-[#0d0d0d] border border-gray-800 hover:border-gray-600 rounded-md px-3 py-1.5 text-sm text-gray-400 transition-colors group"
            >
              <div className="flex items-center gap-2 overflow-hidden">
                <Search size={14} className="text-gray-500 group-hover:text-gray-400 shrink-0" />
                <span className="truncate">Search requests, history, and executions...</span>
              </div>
              <div className="flex items-center gap-1 shrink-0 ml-2">
                <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-500 px-1.5 py-0.5 rounded border border-gray-700 shadow-sm">⌘</span>
                <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-500 px-1.5 py-0.5 rounded border border-gray-700 shadow-sm">K</span>
              </div>
            </button>

            {/* Status messages */}
            {isLoading && (
              <div className="text-xs animate-pulse shrink-0 text-blue-400">Sending…</div>
            )}
            {errorMessage && (
              <div className="text-xs truncate max-w-xs shrink-0 text-red-400" title={errorMessage}>{errorMessage}</div>
            )}

            {/* Env chips + settings */}
            <div className="flex items-center gap-2 ml-auto">
              {environments.length > 0 && (
                <div className="flex items-center gap-1.5 shrink-0">
                  {environments.map(env => (
                    <button
                      key={env.id}
                      onClick={async () => {
                        if (!env.is_active) await api.setActiveEnvironment(env.id);
                        await refreshAll();
                      }}
                      className={`px-2.5 py-1 rounded-full text-xs font-medium transition-all shrink-0 border ${
                        env.is_active
                          ? "bg-blue-500 text-white border-blue-500"
                          : "bg-transparent text-gray-500 border-gray-700 hover:border-gray-500 hover:text-gray-300"
                      }`}
                    >
                      {env.name}
                    </button>
                  ))}
                </div>
              )}
              <button
                onClick={() => setCenterView({ type: "settings" })}
                className="p-1.5 rounded-md text-gray-500 hover:text-gray-300 hover:bg-[#1a1a1a] transition-colors shrink-0"
              >
                <Settings size={16} />
              </button>
            </div>
          </div>

          {/* Full-width URL bar (request view only) */}
          {showInspector && (
            <UrlBar
              data={editorData}
              onChange={onEditorChange}
              onSend={sendRequest}
              onCancel={() => api.cancelRequest()}
              onCopyCurl={copyCurl}
              onImportCurl={importCurl}
              isLoading={isLoading}
              basePath={collections.find(c => c.id === currentRequest?.collection_id)?.base_path ?? ""}
              variables={displayVariables}
            />
          )}

          {/* Body row: center content + inspector side by side */}
          <div className="flex-1 flex overflow-hidden">

            {/* CENTER CONTENT */}
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
                <div ref={splitContainerRef} className="flex-1 flex flex-col overflow-hidden">
                  {/* Request body editor */}
                  {effectivePane !== "response" && (
                    <div
                      style={effectivePane === "split" ? { height: `${splitRatio * 100}%` } : undefined}
                      className={`${effectivePane === "split" ? "shrink-0" : "flex-1"} overflow-hidden border-b border-gray-800`}
                    >
                      <RequestEditor
                        data={editorData}
                        onChange={onEditorChange}
                        isLoading={isLoading}
                        basePath={collections.find(c => c.id === currentRequest?.collection_id)?.base_path ?? ""}
                        requestName={currentRequest?.name ?? ""}
                        variables={displayVariables}
                        isMaximized={effectivePane === "request"}
                        // Only show maximize when there's a response to switch to
                        onMaximize={noResponse ? undefined : () => setSplitOverride(
                          splitOverride === "request" ? "auto" : "request"
                        )}
                      />
                    </div>
                  )}

                  {/* Split drag handle (only in split mode) */}
                  {effectivePane === "split" && (
                    <div
                      className="h-[3px] cursor-row-resize shrink-0 bg-gray-800 hover:bg-blue-500/60 transition-colors"
                      onMouseDown={() => startDrag("split")}
                    />
                  )}

                  {/* Response view */}
                  {effectivePane !== "request" && (
                    <div className="flex-1 overflow-hidden">
                      <ResponseView
                        response={currentResponse}
                        latency={currentLatency}
                        isLoading={isLoading}
                        isMaximized={effectivePane === "response"}
                        onMaximize={() => {
                          if (splitOverride === "response") {
                            setSplitOverride("auto");
                          } else if (effectivePane === "response") {
                            // Auto-collapsed because body=none — force split so user can access request editor
                            setSplitOverride("split");
                          } else {
                            setSplitOverride("response");
                          }
                        }}
                      />
                    </div>
                  )}
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
                className="w-[3px] cursor-col-resize shrink-0 bg-gray-800 hover:bg-blue-500/60 transition-colors"
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
                      if (exec.request_data) {
                        setEditorData(exec.request_data);
                        setDirty(false);
                      }
                    }
                  }}
                  environments={environments}
                  variables={displayVariables}
                />
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Global search modal */}
      {searchOpen && (
        <GlobalSearch
          onClose={() => setSearchOpen(false)}
          onNavigate={(requestId, versionId, executionId, collectionId) => {
            setSearchOpen(false);
            navigateToRequest(requestId, versionId, executionId, collectionId);
          }}
        />
      )}
    </div>
  );
}
