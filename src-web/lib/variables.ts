import type { EnvVariable, Collection, Request } from "./types";
import { resolveVariableRefs } from "./types";
import { resolveDynamicVars } from "./dynamicVars";
import * as api from "./api";

/**
 * Build a fully-resolved variable map for request execution or cURL export.
 * Merges env variables, collection variables, built-in names, and dynamic vars.
 */
export async function buildResolvedVariables(
  envVariables: EnvVariable[],
  collection: Collection | undefined,
  request: Request,
): Promise<Record<string, string>> {
  const variables: Record<string, string> = {};

  for (const v of envVariables) {
    variables[v.key] = v.value;
  }

  const colVars = await api.getActiveCollectionVariables(request.collection_id);
  for (const [k, v] of colVars) {
    variables[k] = v;
  }

  if (collection) variables["collectionName"] = collection.name;
  variables["requestName"] = request.name;

  return resolveVariableRefs(resolveDynamicVars(variables));
}
