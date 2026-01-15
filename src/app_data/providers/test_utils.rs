#[cfg(test)]
use httpmock::{prelude::*, Mock};

#[cfg(test)]
pub fn set_google_env(server_base: &str) {
    std::env::set_var(
        "GOOGLE_OAUTH_CLIENT",
        format!(
            r#"{{
                    "web": {{
                        "client_id": "test_client_id",
                        "client_secret": "test_client_secret",
                        "token_uri": "{}/token"
                    }}
                }}"#,
            server_base
        ),
    );

    std::env::set_var(
        "GOOGLE_OAUTH_REFRESH",
        r#"{ "refresh_token": "test_refresh" }"#,
    );
}

#[cfg(test)]
pub fn clear_google_env() {
    std::env::remove_var("GOOGLE_OAUTH_CLIENT");
    std::env::remove_var("GOOGLE_OAUTH_REFRESH");
}

#[cfg(test)]
pub async fn mock_google_token_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
) -> Mock<'a> {
    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("grant_type=refresh_token")
                .body_includes("refresh_token=test_refresh")
                .body_includes("client_id=test_client_id")
                .body_includes("client_secret=test_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{ "access_token": "{}", "expires_in": {} }}"#,
                    access_token, expires_in
                ));
        })
        .await
}
