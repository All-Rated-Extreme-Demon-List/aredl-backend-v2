use {
    crate::{
        aredl::levels::test_utils::create_test_level_with_record,
        auth::create_test_token,
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn get_user_badges() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let grant_req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(json!({
            "badge_code": "global.level_completion.5",
            "description": "Test level",
        }))
        .to_request();

    let grant_resp = test::call_service(&app, grant_req).await;
    assert!(grant_resp.status().is_success());

    let get_req = test::TestRequest::get()
        .uri(&format!("/users/{user_id}/badges"))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert!(get_resp.status().is_success());

    let badges: serde_json::Value = read_body_json(get_resp).await;
    assert!(badges.as_array().unwrap().iter().any(|badge| {
        badge["badge_code"] == "global.level_completion.5" && badge["description"] == "Test level"
    }));
}

#[actix_web::test]
async fn grant_user_badge() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(json!({
            "badge_code": "global.level_completion.5",
            "description": "Manual grant",
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let badges: serde_json::Value = read_body_json(resp).await;
    assert!(badges.as_array().unwrap().iter().any(|badge| {
        badge["badge_code"] == "global.level_completion.5" && badge["description"] == "Manual grant"
    }));
}

#[actix_web::test]
async fn remove_user_badge() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let grant_req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(json!({
            "badge_code": "global.level_completion.5",
            "description": "Manual grant",
        }))
        .to_request();

    let grant_resp = test::call_service(&app, grant_req).await;
    assert!(grant_resp.status().is_success());

    let remove_req = test::TestRequest::delete()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(json!(["global.level_completion.5"]))
        .to_request();

    let remove_resp = test::call_service(&app, remove_req).await;
    assert!(remove_resp.status().is_success());

    let badges: serde_json::Value = read_body_json(remove_resp).await;
    assert!(!badges
        .as_array()
        .unwrap()
        .iter()
        .any(|badge| badge["badge_code"] == "global.level_completion.5"));
}

#[actix_web::test]
async fn sync_user_badges() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    for _ in 0..5 {
        create_test_level_with_record(&db, user_id).await;
    }

    let req = test::TestRequest::post()
        .uri(&format!("/users/{user_id}/badges/sync"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let badges: serde_json::Value = read_body_json(resp).await;
    assert!(badges
        .as_array()
        .unwrap()
        .iter()
        .any(|badge| badge["badge_code"] == "global.level_completion.5"));
}

#[actix_web::test]
async fn grant_user_badge_rejects_invalid_code() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(json!({
            "badge_code": "global.invalid_badge",
            "description": "Nope",
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, 400, Some("Unknown badge code: global.invalid_badge")).await;
}
