use super::utils::{add_error_example, add_error_response, operations};
use serde_json::{json, Value};
use utoipa::openapi::OpenApi;
use utoipa::Modify;

// register error responses to routes that load/require authentication
pub(super) struct AuthResponsesAddon;

impl Modify for AuthResponsesAddon {
    fn modify(&self, openapi: &mut OpenApi) {
        for item in openapi.paths.paths.values_mut() {
            for (_, operation) in operations(item) {
                let Some(security) = &operation.security else {
                    continue;
                };
                let security: Vec<_> = security
                    .iter()
                    .map(|requirement| json!(requirement))
                    .collect();
                let authenticated = security.iter().any(|requirement| {
                    requirement.get("bearer_token").is_some()
                        || requirement.get("refresh_token").is_some()
                });
                let permission_required = authenticated
                    && security.iter().all(|requirement| {
                        requirement
                            .get("bearer_token")
                            .and_then(Value::as_array)
                            .is_some_and(|scopes| !scopes.is_empty())
                    });
                if authenticated {
                    if let Some(content) = add_error_response(
                        operation,
                        "401",
                        "Missing required authentication, or invalid, expired or invalidated token",
                    ) {
                        for (name, message) in [
                            ("invalid_token", "Invalid token"),
                            ("expired_token", "Expired token"),
                            ("invalid_token_type", "Invalid token type"),
                            ("invalidated_token", "Token has been invalidated"),
                        ] {
                            add_error_example(content, name, message);
                        }
                    }
                }
                if permission_required {
                    let permissions = security
                        .iter()
                        .map(|requirement| {
                            requirement["bearer_token"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .collect::<Vec<_>>()
                        .join(" or ");
                    let description = format!("Missing required permission: {permissions}");
                    if let Some(content) = add_error_response(operation, "403", &description) {
                        for permission in security
                            .iter()
                            .filter_map(|requirement| requirement["bearer_token"].as_array())
                            .flatten()
                            .filter_map(Value::as_str)
                        {
                            add_error_example(content, &format!("missing_{permission}"),
                                &format!("You do not have the required permission ({permission}) to access this endpoint"));
                        }
                    }
                }
            }
        }
    }
}
