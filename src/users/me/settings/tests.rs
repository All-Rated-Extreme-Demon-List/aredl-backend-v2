#[cfg(test)]
use {
    crate::{
        auth::create_test_token,
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    serde_json::{json, Value},
};

#[actix_web::test]
async fn set_user_settingss() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (other_user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    // default None
    let req = test::TestRequest::get()
        .uri("/users/@me/settings")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let settings: Value = read_body_json(resp).await;
    assert_eq!(settings["user_id"], user_id.to_string());
    assert_eq!(settings["clan_notifications"], "None");

    for setting in ["All", "None", "NewOnly"] {
        // patch and check that it's been set
        let req = test::TestRequest::patch()
            .uri("/users/@me/settings")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(json!({"clan_notifications": setting, "user_id": other_user_id}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let settings: Value = read_body_json(resp).await;
        assert_eq!(settings["user_id"], user_id.to_string());
        assert_eq!(settings["clan_notifications"], setting);

        let req = test::TestRequest::get()
            .uri("/users/@me/settings")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let settings: Value = read_body_json(resp).await;
        assert_eq!(settings["clan_notifications"], setting);
    }
}

#[actix_web::test]
async fn settings_require_authentication() {
    let (app, _, _, _) = init_test_app().await;
    for req in [
        test::TestRequest::get()
            .uri("/users/@me/settings")
            .to_request(),
        test::TestRequest::patch()
            .uri("/users/@me/settings")
            .set_json(json!({"clan_notifications": "All"}))
            .to_request(),
    ] {
        let resp = test::call_service(&app, req).await;
        assert_error_response!(resp, StatusCode::UNAUTHORIZED, None);
    }
}
