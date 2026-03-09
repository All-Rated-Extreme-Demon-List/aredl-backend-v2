#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::notes::test_utils::create_test_note, levels::test_utils::create_test_level,
        },
        auth::{create_test_token, Permission},
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn create_note() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let note_data = json!({
        "note": "test note",
        "note_type": "Other",
        "timestamp": null,
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/notes/{}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&note_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(
        level_id.to_string(),
        body["level_id"],
        "Level IDs do not match!"
    );
    assert_eq!(body["note"], note_data["note"]);
    assert_eq!(body["note_type"], note_data["note_type"]);
    assert_eq!(body["added_by"], user_id.to_string());
}

#[actix_web::test]
async fn update_note() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let note_id = create_test_note(&db, level_id, user_id).await;

    let note_data = json!({
        "note": "updated note",
        "note_type": "NerfDate"
    });
    let req = test::TestRequest::patch()
        .uri(format!("/aredl/levels/notes/{}", note_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&note_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["note"], note_data["note"]);
    assert_eq!(body["note_type"], note_data["note_type"]);
}

#[actix_web::test]
async fn delete_note() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let note_id = create_test_note(&db, level_id, user_id).await;

    let req = test::TestRequest::delete()
        .uri(format!("/aredl/levels/notes/{}", note_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn list_notes() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    create_test_note(&db, level_id, user_id).await;
    create_test_note(&db, level_id, user_id).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/levels/notes?level_id={}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();

    assert_eq!(data.len(), 2);
    assert!(data
        .iter()
        .all(|x| x["added_by"]["id"] == user_id.to_string()));
}

#[actix_web::test]
async fn notes_auth() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let note_data = json!({
        "note": "test description",
        "note_type": "Other",
        "timestamp": null
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/notes/{}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&note_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (level_modify) to access this endpoint"),
    )
    .await;
}

#[actix_web::test]
async fn reviewer_notes_are_private() {
    let (app, db, auth, _) = init_test_app().await;

    let level_id = create_test_level(&db).await;

    // Create a reviewer note as a reviewer
    let (reviewer_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let reviewer_token =
        create_test_token(reviewer_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let reviewer_note_data = json!({
        "note": "secret reviewer note",
        "note_type": "ReviewerNotes",
        "timestamp": null
    });
    let create_req = test::TestRequest::post()
        .uri(format!("/aredl/levels/notes/{}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", reviewer_token)))
        .set_json(&reviewer_note_data)
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert!(
        create_resp.status().is_success(),
        "status is {}",
        create_resp.status()
    );

    // non-reviewer should not see ReviewerNotes
    let (normal_user_id, _) = create_test_user(&db, None).await;
    let normal_token = create_test_token(normal_user_id, &auth.jwt_encoding_key)
        .expect("Failed to generate token");

    let list_req = test::TestRequest::get()
        .uri(format!("/aredl/levels/notes?level_id={}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", normal_token)))
        .to_request();
    let list_resp = test::call_service(&app, list_req).await;
    assert!(
        list_resp.status().is_success(),
        "status is {}",
        list_resp.status()
    );

    let list_body: serde_json::Value = read_body_json(list_resp).await;
    let data = list_body["data"].as_array().unwrap();
    assert!(data.iter().all(|x| x["note_type"] != "ReviewerNotes"));

    // reviewer should see ReviewerNotes
    let reviewer_list_req = test::TestRequest::get()
        .uri(format!("/aredl/levels/notes?level_id={}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", reviewer_token)))
        .to_request();
    let reviewer_list_resp = test::call_service(&app, reviewer_list_req).await;
    assert!(
        reviewer_list_resp.status().is_success(),
        "status is {}",
        reviewer_list_resp.status()
    );
    let reviewer_list_body: serde_json::Value = read_body_json(reviewer_list_resp).await;
    let reviewer_data = reviewer_list_body["data"].as_array().unwrap();
    assert!(reviewer_data
        .iter()
        .any(|x| x["note_type"] == "ReviewerNotes"));
}
