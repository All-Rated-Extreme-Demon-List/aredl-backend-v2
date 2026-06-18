#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, oauth::OAuthProvider, test_utils::seed_connected_account},
        test_utils::init_test_app,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
};

#[actix_web::test]
async fn connected_accounts_returns_current_users_accounts() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (other_user_id, _) = create_test_user(&db, None).await;

    seed_connected_account(&db, user_id, OAuthProvider::Patreon, "patreon_123", None);
    seed_connected_account(
        &db,
        user_id,
        OAuthProvider::Discord,
        "discord_123",
        Some("Discord User"),
    );
    seed_connected_account(
        &db,
        other_user_id,
        OAuthProvider::Patreon,
        "patreon_other",
        None,
    );

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::get()
        .uri(&format!("/auth/connected-accounts/user/{user_id}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    let accounts = body.as_array().unwrap();

    assert_eq!(accounts.len(), 2);
    assert!(accounts.iter().any(|account| {
        account["user_id"] == user_id.to_string()
            && account["provider"] == "Patreon"
            && account["provider_user_id"] == "patreon_123"
            && account["provider_user_name"].is_null()
    }));
    assert!(accounts.iter().any(|account| {
        account["user_id"] == user_id.to_string()
            && account["provider"] == "Discord"
            && account["provider_user_id"] == "discord_123"
            && account["provider_user_name"] == "Discord User"
    }));
    assert!(!accounts
        .iter()
        .any(|account| account["provider_user_id"] == "patreon_other"));
}

#[actix_web::test]
async fn connected_accounts_hides_other_users_accounts_without_permission() {
    let (app, db, auth, _) = init_test_app().await;
    let (viewer_id, _) = create_test_user(&db, None).await;
    let (target_id, _) = create_test_user(&db, None).await;

    seed_connected_account(
        &db,
        target_id,
        OAuthProvider::Patreon,
        "patreon_target",
        None,
    );

    let token = create_test_token(viewer_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::get()
        .uri(&format!("/auth/connected-accounts/user/{target_id}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert!(body.as_array().unwrap().is_empty());
}
