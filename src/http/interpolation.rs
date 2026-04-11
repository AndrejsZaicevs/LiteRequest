use std::collections::HashMap;

/// Replace all {{variable}} patterns with their values
pub fn interpolate(input: &str, variables: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key); // {{key}}
        result = result.replace(&pattern, value);
    }
    result
}

/// Resolve a full URL from base_path + request url
pub fn resolve_url(base_path: &str, request_url: &str, variables: &HashMap<String, String>) -> String {
    let base = interpolate(base_path, variables);
    let request = request_url;

    if request.starts_with("http://") || request.starts_with("https://") {
        // Absolute URL — use as-is
        request.to_string()
    } else {
        // Relative — append to base
        let base = base.trim_end_matches('/');
        let path = request.trim_start_matches('/');
        if base.is_empty() {
            path.to_string()
        } else {
            format!("{base}/{path}")
        }
    }
}

/// Replace `:paramName` segments in a URL with their values
pub fn resolve_path_params(url: &str, params: &[(String, String)]) -> String {
    let mut result = url.to_string();
    for (key, value) in params {
        result = result.replace(&format!(":{}", key), value);
    }
    result
}

/// Extract `:paramName` segments from a URL path
pub fn extract_path_params(url: &str) -> Vec<String> {
    // Strip query string and fragment before scanning
    let path_part = url.split('?').next().unwrap_or(url);
    let path_part = path_part.split('#').next().unwrap_or(path_part);
    path_part
        .split('/')
        .filter(|seg| seg.starts_with(':') && seg.len() > 1)
        .map(|seg| seg[1..].to_string())
        .collect()
}

/// Extract variable names from {{...}} patterns (for IntelliSense)
pub fn extract_variable_refs(input: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut i = 0;
    let bytes = input.as_bytes();
    while i + 3 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = input[i + 2..].find("}}") {
                let var_name = &input[i + 2..i + 2 + end];
                vars.push(var_name.trim().to_string());
                i = i + 2 + end + 2;
                continue;
            }
        }
        i += 1;
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate() {
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("version".to_string(), "v2".to_string());

        assert_eq!(
            interpolate("https://{{host}}/{{version}}/users", &vars),
            "https://api.example.com/v2/users"
        );
    }

    #[test]
    fn test_resolve_url_absolute() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_url("https://base.com", "https://other.com/path", &vars),
            "https://other.com/path"
        );
    }

    #[test]
    fn test_resolve_url_relative() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_url("https://base.com/v1", "/users", &vars),
            "https://base.com/v1/users"
        );
    }

    #[test]
    fn test_extract_variable_refs() {
        let refs = extract_variable_refs("{{host}}/{{version}}/users");
        assert_eq!(refs, vec!["host", "version"]);
    }

    #[test]
    fn test_resolve_path_params() {
        let params = vec![
            ("userId".to_string(), "42".to_string()),
            ("orderId".to_string(), "99".to_string()),
        ];
        assert_eq!(
            resolve_path_params("/users/:userId/orders/:orderId", &params),
            "/users/42/orders/99"
        );
    }

    #[test]
    fn test_resolve_path_params_no_match() {
        let params = vec![("id".to_string(), "1".to_string())];
        assert_eq!(
            resolve_path_params("/users/:userId", &params),
            "/users/:userId"
        );
    }

    #[test]
    fn test_extract_path_params_basic() {
        let params = extract_path_params("/users/:userId/orders/:orderId");
        assert_eq!(params, vec!["userId", "orderId"]);
    }

    #[test]
    fn test_extract_path_params_with_query() {
        let params = extract_path_params("/users/:id?foo=bar");
        assert_eq!(params, vec!["id"]);
    }

    #[test]
    fn test_extract_path_params_none() {
        let params = extract_path_params("/users/all");
        assert!(params.is_empty());
    }

    #[test]
    fn test_collection_vars_override_globals() {
        let mut globals = HashMap::new();
        globals.insert("host".to_string(), "global.example.com".to_string());
        globals.insert("version".to_string(), "v1".to_string());

        // Simulate collection variable overriding "host"
        globals.insert("host".to_string(), "billing.example.com".to_string());

        assert_eq!(
            interpolate("https://{{host}}/{{version}}/charges", &globals),
            "https://billing.example.com/v1/charges"
        );
    }

    #[test]
    fn test_base_path_with_instance_variable() {
        let mut vars = HashMap::new();
        vars.insert("host".to_string(), "api.example.com".to_string());
        vars.insert("instance_id".to_string(), "inst-123".to_string());

        assert_eq!(
            resolve_url("https://{{host}}/{{instance_id}}/v1", "/charges", &vars),
            "https://api.example.com/inst-123/v1/charges"
        );
    }
}
