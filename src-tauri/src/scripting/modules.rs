use rquickjs::loader::{Loader, Resolver};
use crate::db::Database;
use std::sync::{Arc, Mutex};

const LR_PREFIX: &str = "lr:collections/";

/// Resolves `lr:collections/<name>` module specifiers.
pub struct LrResolver;

impl Resolver for LrResolver {
    fn resolve(&mut self, _ctx: &rquickjs::Ctx<'_>, base: &str, name: &str) -> rquickjs::Result<String> {
        if name.starts_with(LR_PREFIX) {
            Ok(name.to_string())
        } else {
            Err(rquickjs::Error::new_resolving(base, name))
        }
    }
}

/// Loads `lr:collections/<name>` modules by generating JS that calls
/// the HTTP bridge functions for each request in the collection.
pub struct LrLoader {
    db: Arc<Mutex<Database>>,
}

impl LrLoader {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Generate a JS module source for a collection by name.
    fn generate_module_source(&self, collection_name: &str) -> Result<String, String> {
        let db = self.db.lock().map_err(|e| format!("DB lock: {e}"))?;

        let collections = db.list_collections().map_err(|e| format!("{e}"))?;
        let collection = collections.iter()
            .find(|c| c.name == collection_name)
            .ok_or_else(|| format!("Collection '{}' not found", collection_name))?;

        let requests = db.list_requests_by_collection(&collection.id)
            .map_err(|e| format!("{e}"))?;

        let mut exports = Vec::new();
        for req in &requests {
            let fn_name = sanitize_to_camel_case(&req.name);
            if fn_name.is_empty() {
                continue;
            }

            // Get current version data for default method/url/headers etc.
            let version_data = if let Some(ref vid) = req.current_version_id {
                db.get_version(vid).ok().map(|v| v.data)
            } else {
                None
            };

            let (method, url, default_headers, default_params) = match version_data {
                Some(ref d) => {
                    let headers_json = serde_json::to_string(
                        &d.headers.iter()
                            .filter(|h| h.enabled && !h.key.is_empty())
                            .map(|h| (&h.key, &h.value))
                            .collect::<Vec<_>>()
                    ).unwrap_or_else(|_| "[]".to_string());
                    let params_json = serde_json::to_string(
                        &d.query_params.iter()
                            .filter(|p| p.enabled && !p.key.is_empty())
                            .map(|p| (&p.key, &p.value))
                            .collect::<Vec<_>>()
                    ).unwrap_or_else(|_| "[]".to_string());
                    (d.method.as_str().to_string(), d.url.clone(), headers_json, params_json)
                }
                None => ("GET".to_string(), String::new(), "[]".to_string(), "[]".to_string()),
            };

            // Each exported function calls the __lr_execute bridge
            exports.push(format!(
                r#"export async function {fn_name}(options) {{
  const opts = options || {{}};
  return await __lr_execute({{
    requestId: "{req_id}",
    collectionId: "{coll_id}",
    method: "{method}",
    url: "{url}",
    defaultHeaders: {default_headers},
    defaultParams: {default_params},
    overrides: opts,
  }});
}}"#,
                fn_name = fn_name,
                req_id = req.id,
                coll_id = collection.id,
                method = method,
                url = url.replace('"', r#"\""#),
                default_headers = default_headers,
                default_params = default_params,
            ));
        }

        Ok(exports.join("\n\n"))
    }
}

impl Loader for LrLoader {
    fn load<'js>(&mut self, ctx: &rquickjs::Ctx<'js>, name: &str) -> rquickjs::Result<rquickjs::module::Module<'js>> {
        if let Some(collection_name) = name.strip_prefix(LR_PREFIX) {
            let source = self.generate_module_source(collection_name)
                .map_err(|e| rquickjs::Error::new_loading_message(name, &e))?;
            rquickjs::module::Module::declare(ctx.clone(), name, source)
                .map_err(|e| rquickjs::Error::new_loading_message(name, &format!("{e}")))
        } else {
            Err(rquickjs::Error::new_loading(name))
        }
    }
}

/// Convert a name like "Get Users" or "POST /api/users" to "getUsers" or "postApiUsers".
pub fn sanitize_to_camel_case(name: &str) -> String {
    let cleaned: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' { c } else { ' ' })
        .collect();

    let words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_uppercase().next().unwrap_or(first));
                result.extend(chars.map(|c| c.to_lowercase().next().unwrap_or(c)));
            }
        }
    }

    // Ensure it starts with a letter
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, '_');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_names() {
        assert_eq!(sanitize_to_camel_case("Get Users"), "getUsers");
        assert_eq!(sanitize_to_camel_case("POST /api/users"), "postApiUsers");
        assert_eq!(sanitize_to_camel_case("list-items"), "listItems");
        assert_eq!(sanitize_to_camel_case("my_request"), "myRequest");
        assert_eq!(sanitize_to_camel_case("123test"), "_123test");
    }
}
