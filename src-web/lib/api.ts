import { invoke } from "@tauri-apps/api/core";
import type {
  Collection, Folder, Request, RequestVersion, RequestExecution,
  RequestData, ResponseData, Environment, EnvVariable, VarDef, VarRow,
  ClientCertEntry,
} from "./types";

// ── Collections ──────────────────────────────────────────────
export const listCollections = () => invoke<Collection[]>("list_collections");
export const insertCollection = (collection: Collection) => invoke<void>("insert_collection", { collection });
export const updateCollection = (collection: Collection) => invoke<void>("update_collection", { collection });
export const deleteCollection = (id: string) => invoke<void>("delete_collection", { id });
export const renameCollection = (id: string, name: string) => invoke<void>("rename_collection", { id, name });

// ── Folders ──────────────────────────────────────────────────
export const listFolders = (collectionId: string) => invoke<Folder[]>("list_folders", { collectionId });
export const insertFolder = (folder: Folder) => invoke<void>("insert_folder", { folder });
export const deleteFolder = (id: string) => invoke<void>("delete_folder", { id });
export const renameFolder = (id: string, name: string) => invoke<void>("rename_folder", { id, name });
export const moveFolder = (id: string, collectionId: string, parentFolderId?: string | null) =>
  invoke<void>("move_folder", { id, collectionId, parentFolderId });

// ── Requests ─────────────────────────────────────────────────
export const listRequestsByCollection = (collectionId: string) =>
  invoke<Request[]>("list_requests_by_collection", { collectionId });
export const listRequestsByFolder = (folderId: string) =>
  invoke<Request[]>("list_requests_by_folder", { folderId });
export const listOrphanRequests = (collectionId: string) =>
  invoke<Request[]>("list_orphan_requests", { collectionId });
export const insertRequest = (request: Request) => invoke<void>("insert_request", { request });
export const renameRequest = (id: string, name: string) => invoke<void>("rename_request", { id, name });
export const deleteRequest = (id: string) => invoke<void>("delete_request", { id });
export const moveRequest = (id: string, collectionId: string, folderId?: string | null) =>
  invoke<void>("move_request", { id, collectionId, folderId });
export const reorderRequests = (orderedIds: string[]) => invoke<void>("reorder_requests", { orderedIds });
export const updateRequestVersion = (requestId: string, versionId: string) =>
  invoke<void>("update_request_version", { requestId, versionId });

// ── Versions ─────────────────────────────────────────────────
export const insertVersion = (version: RequestVersion) => invoke<void>("insert_version", { version });
export const getVersion = (id: string) => invoke<RequestVersion>("get_version", { id });
export const listVersions = (requestId: string) => invoke<RequestVersion[]>("list_versions", { requestId });
export const updateVersionData = (versionId: string, data: RequestData, createdAt: string) =>
  invoke<void>("update_version_data", { versionId, data, createdAt });
export const deleteVersion = (versionId: string) => invoke<void>("delete_version", { versionId });
export const versionHasExecutions = (versionId: string) => invoke<boolean>("version_has_executions", { versionId });

// ── Executions ───────────────────────────────────────────────
export const insertExecution = (execution: RequestExecution) => invoke<void>("insert_execution", { execution });
export const listExecutions = (requestId: string) => invoke<RequestExecution[]>("list_executions", { requestId });

// ── Environments ─────────────────────────────────────────────
export const listEnvironments = () => invoke<Environment[]>("list_environments");
export const insertEnvironment = (environment: Environment) => invoke<void>("insert_environment", { environment });
export const setActiveEnvironment = (id: string) => invoke<void>("set_active_environment", { id });
export const deleteEnvironment = (id: string) => invoke<void>("delete_environment", { id });

// ── Env Variables ────────────────────────────────────────────
export const listEnvVariables = (environmentId: string) => invoke<EnvVariable[]>("list_env_variables", { environmentId });
export const insertEnvVariable = (variable: EnvVariable) => invoke<void>("insert_env_variable", { variable });
export const updateEnvVariable = (variable: EnvVariable) => invoke<void>("update_env_variable", { variable });
export const deleteEnvVariable = (id: string) => invoke<void>("delete_env_variable", { id });
export const getActiveVariables = () => invoke<EnvVariable[]>("get_active_variables");

// ── Collection Variables ─────────────────────────────────────
export const insertVarDef = (def: VarDef) => invoke<void>("insert_var_def", { def });
export const updateVarDefKey = (defId: string, key: string) => invoke<void>("update_var_def_key", { defId, key });
export const deleteVarDef = (defId: string) => invoke<void>("delete_var_def", { defId });
export const listVarDefs = (collectionId: string) => invoke<VarDef[]>("list_var_defs", { collectionId });
export const upsertVarValue = (valId: string, defId: string, environmentId: string, value: string, isSecret: boolean) =>
  invoke<void>("upsert_var_value", { valId, defId, environmentId, value, isSecret });
export const loadVarRows = (collectionId: string, environmentId: string) =>
  invoke<VarRow[]>("load_var_rows", { collectionId, environmentId });
export const getActiveCollectionVariables = (collectionId: string) =>
  invoke<[string, string][]>("get_active_collection_variables", { collectionId });

// ── App Settings ─────────────────────────────────────────────
export const getAppSetting = (key: string) => invoke<string | null>("get_app_setting", { key });
export const setAppSetting = (key: string, value: string) => invoke<void>("set_app_setting", { key, value });

// ── HTTP Execution ───────────────────────────────────────────
export const executeRequest = (
  data: RequestData,
  variables: Record<string, string>,
  basePath: string,
  clientCerts: ClientCertEntry[],
) => invoke<[ResponseData, number]>("execute_request", { data, variables, basePath, clientCerts });

// ── cURL ─────────────────────────────────────────────────────
export const toCurl = (data: RequestData, variables: Record<string, string>, basePath: string) =>
  invoke<string>("to_curl", { data, variables, basePath });
export const parseCurl = (input: string) => invoke<RequestData>("parse_curl", { input });

// ── Interpolation ────────────────────────────────────────────
export const interpolateStr = (input: string, variables: Record<string, string>) =>
  invoke<string>("interpolate", { input, variables });
export const resolveUrl = (basePath: string, requestUrl: string, variables: Record<string, string>) =>
  invoke<string>("resolve_url", { basePath, requestUrl, variables });
export const extractPathParams = (url: string) => invoke<string[]>("extract_path_params", { url });

// ── Maintenance ──────────────────────────────────────────────
export const pruneOldExecutions = (days: number) => invoke<number>("prune_old_executions", { days });
