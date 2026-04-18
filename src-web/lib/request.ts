import type { RequestData, KeyValuePair, AuthConfig, Collection } from "./types";

/**
 * Merge collection-level auth and default headers into request data.
 * Collection headers are prepended (lower priority than request-level headers).
 * Auth headers are only added if the request doesn't already set them.
 */
export function buildEffectiveData(
  baseData: RequestData,
  collection: Collection | undefined,
): RequestData {
  const requestHeaderKeys = new Set(
    baseData.headers.filter(h => h.enabled && h.key).map(h => h.key.toLowerCase())
  );
  const extraHeaders: KeyValuePair[] = [];

  if (collection?.headers_config) {
    try {
      const defaults = JSON.parse(collection.headers_config) as KeyValuePair[];
      for (const h of defaults.filter(h => h.enabled && h.key)) {
        if (!requestHeaderKeys.has(h.key.toLowerCase())) {
          extraHeaders.push(h);
          requestHeaderKeys.add(h.key.toLowerCase());
        }
      }
    } catch { /* ignore malformed config */ }
  }

  if (collection?.auth_config) {
    try {
      const auth = JSON.parse(collection.auth_config) as AuthConfig;
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
    } catch { /* ignore malformed config */ }
  }

  return extraHeaders.length > 0
    ? { ...baseData, headers: [...extraHeaders, ...baseData.headers] }
    : baseData;
}
