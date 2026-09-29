use super::utils::{add_error_example, add_error_response, operations};
use crate::get_optional_secret;
use serde_json::json;
use std::env::var;
use utoipa::openapi::{
    extensions::Extensions,
    path::ParameterIn,
    schema::Components,
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi, Required, Server,
};
use utoipa::Modify;

/// register available security schemes
pub(super) struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::default);
        components.add_security_scheme(
            "bearer_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Access token or API key, sent as Authorization: Bearer <token>.",
                    ))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "refresh_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// register server URL
pub(super) struct ServerAddon;

impl Modify for ServerAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        let mut server: Server = Server::default();
        server.url = var("DOCS_API_SERVER").unwrap_or_else(|_| {
            let port = get_optional_secret("PORT").unwrap_or_else(|| "8080".to_owned());
            format!("http://127.0.0.1:{port}")
        });
        server.description = Some("API Server".to_owned());
        openapi.servers = Some(vec![server]);
    }
}

/// register unique operation IDs to avoid conflicts of same-named handlers
pub(super) struct OperationIdAddon;

impl Modify for OperationIdAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        for (path, item) in &mut openapi.paths.paths {
            let path_id = path
                .trim_start_matches('/')
                .replace(['/', '-'], "_")
                .replace('{', "by_")
                .replace('}', "")
                .replace('@', "current_");
            for (method, operation) in operations(item) {
                operation.operation_id = Some(format!("{method}_{path_id}"));
            }
        }
    }
}

/// register x-badges based on a route's summary prefix
pub(super) struct StaffBadgeAddon;

impl Modify for StaffBadgeAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        for item in openapi.paths.paths.values_mut() {
            for (_, operation) in operations(item) {
                let mut badges = Vec::new();
                if let Some(summary) = operation.summary.as_mut() {
                    for (marker, label, color) in [
                        ("[Staff]", "Staff Only", "red"),
                        ("[AuthPublic]", "Authed User / Public", "green"),
                        ("[Auth]", "Authed User", "orange"),
                    ] {
                        if summary.contains(marker) {
                            *summary = summary.replace(marker, "").trim().to_owned();
                            badges.push(json!({"label": label, "color": color}));
                        }
                    }
                }
                if badges.is_empty() {
                    badges.push(json!({"label": "Public", "color": "green"}));
                }
                operation
                    .extensions
                    .get_or_insert_with(Extensions::default)
                    .entry("x-badges".to_owned())
                    .and_modify(|existing| {
                        if let Some(array) = existing.as_array_mut() {
                            array.extend(badges.iter().cloned());
                        }
                    })
                    .or_insert_with(|| json!(badges));
            }
        }
    }
}

/// register error responses for invalid extractor inputs (query, path, body)
pub(super) struct ExtractorResponsesAddon;

impl Modify for ExtractorResponsesAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        for item in openapi.paths.paths.values_mut() {
            let path_parameters = item.parameters.clone();
            for (_, operation) in operations(item) {
                let parameters = operation
                    .parameters
                    .iter()
                    .flatten()
                    .chain(path_parameters.iter().flatten());
                let has_query = parameters
                    .clone()
                    .any(|parameter| matches!(parameter.parameter_in, ParameterIn::Query));
                if has_query {
                    if let Some(content) =
                        add_error_response(operation, "400", "Invalid query parameters")
                    {
                        add_error_example(
                            content,
                            "invalid_query",
                            "Query deserialize error: invalid digit found in string",
                        );
                    }
                }
                if operation.request_body.as_ref().is_some_and(|body| {
                    body.required == Some(Required::True)
                        && body.content.contains_key("application/json")
                }) {
                    add_error_response(operation, "400", "Invalid JSON body");
                }
            }
        }
    }
}
