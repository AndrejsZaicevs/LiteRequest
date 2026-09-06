import type { EnvVariable, Collection, Request, Folder, VarRow } from "./types";
import { resolveVariableRefs } from "./types";
import { resolveDynamicVars } from "./dynamicVars";
import * as api from "./api";

/**
 * Merge variable scopes into a single map.
 * Priority (lowest → highest): environment < collection < built-ins.
 * This is the single source of truth for scope priority — both the
 * execution path and the display/tooltip path must go through it.
 */
export function mergeVariables(
  envVariables: Pick<EnvVariable, "key" | "value">[],
  collectionVars: Record<string, string>,
  builtins: { collectionName?: string; requestName?: string; parentName?: string },
): Record<string, string> {
  const variables: Record<string, string> = {};
  for (const v of envVariables) {
    variables[v.key] = v.value;
  }
  Object.assign(variables, collectionVars);
  if (builtins.collectionName) variables["collectionName"] = builtins.collectionName;
  if (builtins.requestName) variables["requestName"] = builtins.requestName;
  if (builtins.parentName) variables["parentName"] = builtins.parentName;
  return variables;
}

/**
 * Name of the request's parent in the sidebar tree: its folder if it is
 * inside one, otherwise the collection it belongs to.
 */
export function resolveParentName(
  request: Request,
  folders: Folder[],
  collection: Collection | undefined,
): string | undefined {
  if (request.folder_id) {
    const folder = folders.find(f => f.id === request.folder_id);
    if (folder) return folder.name;
  }
  return collection?.name;
}

export const SECRET_MASK = "***";

/**
 * Replace the values of the given secret keys with a mask.
 * Must run BEFORE {{ref}} resolution so a secret referenced from another
 * variable (e.g. token = "Bearer {{API_KEY}}") is masked there too.
 */
export function maskSecretVariables(
  vars: Record<string, string>,
  secretKeys: Set<string>,
  mask: string = SECRET_MASK,
): Record<string, string> {
  if (secretKeys.size === 0) return vars;
  const out = { ...vars };
  for (const key of secretKeys) {
    if (key in out) out[key] = mask;
  }
  return out;
}

/**
 * Which variable names are marked as hidden/secret, respecting scope
 * priority: a collection value overrides an env value, so a non-secret
 * collection value un-hides a key that is secret at the env level, and
 * vice versa. Rows with no value in the active env are ignored.
 */
export function collectSecretKeys(
  envVariables: Pick<EnvVariable, "key" | "is_secret">[],
  collectionRows: Pick<VarRow, "key" | "is_secret" | "value_id">[] = [],
): Set<string> {
  const secret = new Set<string>();
  for (const v of envVariables) {
    if (v.is_secret) secret.add(v.key);
  }
  for (const row of collectionRows) {
    if (row.value_id == null) continue;
    if (row.is_secret) secret.add(row.key);
    else secret.delete(row.key);
  }
  return secret;
}

export interface BuildResolvedOptions {
  /** Keys whose values should be replaced with `mask` (used for cURL export). */
  secretKeys?: Set<string>;
  mask?: string;
}

/**
 * Build a fully-resolved variable map for request execution or cURL export.
 * Merges env variables, collection variables, built-in names, and dynamic vars.
 */
export async function buildResolvedVariables(
  envVariables: EnvVariable[],
  collection: Collection | undefined,
  request: Request,
  folders: Folder[] = [],
  options: BuildResolvedOptions = {},
): Promise<Record<string, string>> {
  const colVars = await api.getActiveCollectionVariables(request.collection_id);
  let variables = mergeVariables(envVariables, Object.fromEntries(colVars), {
    collectionName: collection?.name,
    requestName: request.name,
    parentName: resolveParentName(request, folders, collection),
  });
  if (options.secretKeys) {
    variables = maskSecretVariables(variables, options.secretKeys, options.mask);
  }
  return resolveVariableRefs(resolveDynamicVars(variables));
}
