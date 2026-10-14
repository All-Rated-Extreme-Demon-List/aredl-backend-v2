#[cfg(test)]
use {
    crate::users::test_utils::create_test_placeholder_user,
    crate::{
        audit::AuditAction,
        auth::{
            create_test_token,
            oauth::OAuthProvider,
            test_utils::{create_test_auth_token, seed_connected_account, seed_oauth_request},
            Permission,
        },
        list::{
            levels::test_utils::create_test_level,
            records::test_utils::{create_test_record, get_test_record},
            submissions::{
                test_utils::{
                    create_test_submission, get_test_submission, latest_test_submission_history,
                    set_test_submission_locked, set_test_submission_raw_url_status_and_reviewer,
                },
                SubmissionStatus,
            },
        },
        roles::test_utils::{
            add_permission_to_role, add_user_to_role, create_test_hidden_role, create_test_role,
        },
        test_utils::{assert_error_response, init_test_app},
        users::{
            test_utils::{
                create_test_user, create_test_user_with_permissions, get_test_user,
                get_test_user_data_state, latest_audit_entry_for_user,
                set_test_user_account_status, set_test_user_discord_id,
                set_test_user_discord_profile,
            },
            AccountStatus, User, UserUpsert,
        },
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    chrono::Utc,
    serde_json::json,
    uuid::Uuid,
};

#[actix_web::test]
async fn delete_user() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, username) = create_test_user(&db, None).await;
    set_test_user_discord_profile(&db, user_id);
    seed_connected_account(
        &db,
        user_id,
        OAuthProvider::Discord,
        "123456789012345678",
        Some(&username),
    );
    seed_oauth_request(
        &db,
        OAuthProvider::Discord,
        "delete-user-oauth",
        None,
        None,
        Some(user_id),
    );
    let before = get_test_user(&db, user_id);
    let (staff_id, _) = create_test_user(&db, Some(Permission::UserDelete)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let mut submission_ids = Vec::new();
    for status in [
        SubmissionStatus::Pending,
        SubmissionStatus::Claimed,
        SubmissionStatus::UnderConsideration,
        SubmissionStatus::UnderReview,
        SubmissionStatus::Accepted,
        SubmissionStatus::Denied,
    ] {
        let level_id = create_test_level(&db).await;
        let submission_id = create_test_submission(level_id, user_id, &db).await;
        set_test_submission_raw_url_status_and_reviewer(
            &db,
            submission_id,
            None,
            status,
            Some(staff_id),
        );
        set_test_submission_locked(&db, submission_id, true);
        submission_ids.push(submission_id);
    }
    let (other_user_id, _) = create_test_user(&db, None).await;
    let other_level_id = create_test_level(&db).await;
    let other_record_id = create_test_record(&db, other_user_id, other_level_id).await;
    let other_submission_id = get_test_record(&db, other_record_id).submission_id;

    let req = test::TestRequest::delete()
        .uri("/users/123456789012345678")
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::NO_CONTENT
    );

    let deleted = get_test_user(&db, user_id);
    assert_eq!(deleted.id, user_id);
    assert_eq!(deleted.global_name, "Deleted User");
    assert_ne!(deleted.username, username);
    assert_eq!(deleted.username.len(), 9);
    assert!(deleted.username.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(deleted.account_status, AccountStatus::Deleted);
    assert_eq!(deleted.discord_id, None);
    assert_eq!(deleted.discord_avatar, None);
    assert_eq!(deleted.discord_avatar_decoration, None);
    assert_eq!(deleted.description, None);
    assert_eq!(deleted.country, None);
    assert_eq!(deleted.background_level, None);
    assert_eq!(deleted.featured_badge_code, None);
    assert!(deleted.access_valid_after > before.access_valid_after);
    let data = get_test_user_data_state(&db, user_id);
    assert_eq!(data.record_count, 0);
    assert_eq!(data.connected_account_count, 0);
    assert_eq!(data.oauth_request_count, 0);
    assert_eq!(data.last_discord_avatar_update, None);
    for submission_id in submission_ids {
        assert_eq!(
            get_test_submission(&db, submission_id).status,
            SubmissionStatus::Denied
        );
        assert_eq!(
            latest_test_submission_history(&db, submission_id).status,
            SubmissionStatus::Denied
        );
    }
    assert_eq!(
        get_test_record(&db, other_record_id).submitted_by,
        other_user_id
    );
    assert_eq!(
        get_test_submission(&db, other_submission_id).status,
        SubmissionStatus::Accepted
    );
    let log = latest_audit_entry_for_user(&db, user_id, AuditAction::Update);
    assert_eq!(log.actor_id, Some(staff_id));
    assert_eq!(
        log.diff.expect("Expected deletion diff")["account_status"],
        5
    );

    let req = test::TestRequest::delete()
        .uri(&format!("/users/{user_id}"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(get_test_user(&db, user_id).username, deleted.username);
}

#[actix_web::test]
async fn delete_me_invalidates_access_refresh_and_api_key_tokens() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let record_id = create_test_record(&db, user_id, level_id).await;
    let submission_id = get_test_record(&db, record_id).submission_id;
    let refresh_token = create_test_auth_token(user_id, &auth.jwt_encoding_key, false, "refresh");
    let api_key = create_test_auth_token(user_id, &auth.jwt_encoding_key, true, "access");
    let req = test::TestRequest::delete()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(get_test_user(&db, user_id).global_name, "Deleted User");
    assert_eq!(
        get_test_submission(&db, submission_id).status,
        SubmissionStatus::Denied
    );
    let data = get_test_user_data_state(&db, user_id);
    assert_eq!(data.record_count, 0);
    assert_eq!(data.connected_account_count, 0);
    assert_eq!(data.oauth_request_count, 0);
    assert_eq!(data.last_discord_avatar_update, None);
    assert_eq!(
        latest_audit_entry_for_user(&db, user_id, AuditAction::Update).actor_id,
        Some(user_id)
    );

    for (uri, invalid_token) in [
        ("/users/@me", token),
        ("/users/@me", api_key),
        ("/auth/refresh", refresh_token),
    ] {
        let req = test::TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", format!("Bearer {invalid_token}")))
            .to_request();
        assert_error_response!(
            test::call_service(&app, req).await,
            StatusCode::UNAUTHORIZED,
            Some("Token has been invalidated")
        );
    }
}

#[actix_web::test]
async fn delete_user_requires_delete_permission_and_higher_privilege() {
    let (app, db, auth, _) = init_test_app().await;
    let (target_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;
    let token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to create token");
    let req = test::TestRequest::delete()
        .uri(&format!("/users/{target_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::FORBIDDEN,
        Some("You do not have the required permission (user_delete) to access this endpoint")
    );
    assert_eq!(
        get_test_user(&db, target_id).account_status,
        AccountStatus::Normal
    );

    let (deleter_id, _) = create_test_user(&db, Some(Permission::UserDelete)).await;
    let token =
        create_test_token(deleter_id, &auth.jwt_encoding_key).expect("Failed to create token");
    let req = test::TestRequest::delete()
        .uri(&format!("/users/{staff_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::FORBIDDEN,
        Some("You do not have sufficient privilege to affect this user.")
    );
    assert_eq!(
        get_test_user(&db, staff_id).account_status,
        AccountStatus::Normal
    );
}

#[actix_web::test]
async fn deleted_status_cannot_be_manually_set_or_later_edited() {
    let (app, db, auth, _) = init_test_app().await;
    let (target_id, _) = create_test_user(&db, None).await;
    let target_token =
        create_test_token(target_id, &auth.jwt_encoding_key).expect("Failed to create token");
    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {target_token}")))
        .set_json(json!({"account_status": 5}))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::BAD_REQUEST,
        Some("You can only set your account status to normal, semi-unranked, or unranked.")
    );
    let (staff_id, _) = create_test_user_with_permissions(
        &db,
        &[
            Permission::UserBan,
            Permission::UserRedact,
            Permission::UserDelete,
            Permission::UserModify,
        ],
    )
    .await;
    let token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to create token");
    let req = test::TestRequest::patch()
        .uri(&format!("/users/{target_id}/ban"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"account_status": 5}))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::BAD_REQUEST,
        Some("Account status cannot be manually set to deleted, use the delete endpoint instead.")
    );
    let req = test::TestRequest::delete()
        .uri(&format!("/users/{target_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, req).await.status(),
        StatusCode::NO_CONTENT
    );
    let req = test::TestRequest::patch()
        .uri(&format!("/users/{target_id}/ban"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"account_status": 0}))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::NOT_FOUND,
        Some("Not found")
    );
    assert_eq!(
        get_test_user(&db, target_id).account_status,
        AccountStatus::Deleted
    );
    let req = test::TestRequest::patch()
        .uri(&format!("/users/{target_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"global_name": "Restored user"}))
        .to_request();
    assert_error_response!(
        test::call_service(&app, req).await,
        StatusCode::NOT_FOUND,
        Some("Not found")
    );
}

#[actix_web::test]
async fn create_placeholder_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_user_id, _) = create_test_user(&db, Some(Permission::PlaceholderCreate)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let placeholder_payload = json!({
        "username": "test_placeholder"
    });

    let req = test::TestRequest::post()
        .uri("/users/placeholders")
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&placeholder_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_http::StatusCode::CREATED);

    let created_user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(created_user["global_name"], "test_placeholder");

    let created_user_id: Uuid = created_user["id"]
        .as_str()
        .expect("Expected user id in response")
        .parse()
        .expect("Expected valid user id");
    let log = latest_audit_entry_for_user(&db, created_user_id, AuditAction::Create);
    assert_eq!(log.actor_id, Some(staff_user_id));
    if let Some(diff) = log.diff {
        assert_eq!(diff["global_name"], "test_placeholder");
    } else {
        panic!("Expected diff in audit log for user creation");
    }
}

#[actix_web::test]
async fn update_user_info() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let update_payload = json!({
        "global_name": "Updated Name",
        "description": "Updated description"
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let updated_user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(updated_user["global_name"], "Updated Name");
    assert_eq!(updated_user["description"], "Updated description");

    let log = latest_audit_entry_for_user(&db, user_id, AuditAction::Update);
    assert_eq!(log.actor_id, Some(staff_user_id));
    if let Some(diff) = log.diff {
        assert_eq!(diff["global_name"], "Updated Name");
        assert_eq!(diff["description"], "Updated description");
    } else {
        panic!("Expected diff in audit log for user update");
    }
}

#[actix_web::test]
async fn update_user_info_rejects_editing_user_with_same_or_higher_privilege() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::UserModify)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;

    let update_payload = json!({
        "global_name": "Updated Name",
        "description": "Updated description"
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{staff_user_id}"))
        .insert_header(("Authorization", format!("Bearer {user_token}")))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn ban_user() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, username) = create_test_user(&db, None).await;
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;
    let (manager_user_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let manager_token = create_test_token(manager_user_id, &auth.jwt_encoding_key)
        .expect("Failed to generate token");

    let ban_payload = json!({ "account_status": 3 });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/ban"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&ban_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let banned_user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(banned_user["account_status"], 3);

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{username}%"))
        .insert_header(("Authorization", format!("Bearer {manager_token}")))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap()[0]["account_status"], 3);
}

#[actix_web::test]
async fn redact_user_requires_redact_permission() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let redact_payload = json!({ "account_status": 4 });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/ban"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&redact_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);

    let account_status = get_test_user(&db, user_id).account_status;
    assert_ne!(account_status, AccountStatus::Redacted);
}

#[actix_web::test]
async fn redact_user_succeeds_with_redact_permission() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_user_id, _) =
        create_test_user_with_permissions(&db, &[Permission::UserBan, Permission::UserRedact])
            .await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let redact_payload = json!({ "account_status": 4 });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/ban"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&redact_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let redacted_user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(redacted_user["account_status"], 4);
}

#[actix_web::test]
async fn find_user() {
    let (app, db, _, _) = init_test_app().await;
    let (user_id, username) = create_test_user(&db, None).await;

    let req = test::TestRequest::get()
        .uri(&format!("/users/{user_id}"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(user["username"], username);
}

#[actix_web::test]
async fn find_user_by_discord_id() {
    let (app, db, _, _) = init_test_app().await;
    let (user_id, username) = create_test_user(&db, None).await;
    let discord_id = "1234567890";

    set_test_user_discord_id(&db, user_id, discord_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/users/{discord_id}"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let user: serde_json::Value = read_body_json(resp).await;
    assert_eq!(user["username"], username);
}

#[actix_web::test]
async fn find_user_hides_hidden_roles_and_hidden_scopes_except_for_role_assign() {
    let (app, db, auth, _) = init_test_app().await;
    let (target_user, _) = create_test_user(&db, None).await;
    let (normal_requester, _) = create_test_user(&db, None).await;
    let (manager_requester, _) = create_test_user(&db, Some(Permission::RoleAssign)).await;

    let visible_role_id = create_test_role(&db, 1).await;
    let hidden_role_id = create_test_hidden_role(&db, 0).await;
    add_permission_to_role(&db, hidden_role_id, Permission::SubmissionReview).await;

    add_user_to_role(&db, visible_role_id, target_user).await;
    add_user_to_role(&db, hidden_role_id, target_user).await;

    let normal_token = create_test_token(normal_requester, &auth.jwt_encoding_key).unwrap();
    let manager_token = create_test_token(manager_requester, &auth.jwt_encoding_key).unwrap();

    let assert_hidden_data_is_not_exposed = |body: &serde_json::Value| {
        let roles = body["roles"]
            .as_array()
            .expect("roles should be an array in user response");
        let scopes = body["scopes"]
            .as_array()
            .expect("scopes should be an array in user response");

        assert!(
            !roles
                .iter()
                .any(|role| role["id"] == hidden_role_id.to_string()),
            "Hidden role should not be present in user response"
        );
        assert!(
            roles
                .iter()
                .any(|role| role["id"] == visible_role_id.to_string()),
            "Visible role should be present in user response"
        );
        assert!(
            !scopes.iter().any(|scope| scope == "submission_review"),
            "Hidden-role-derived scope should not be present in user response"
        );
    };

    let assert_hidden_data_is_exposed = |body: &serde_json::Value| {
        let roles = body["roles"]
            .as_array()
            .expect("roles should be an array in user response");
        let scopes = body["scopes"]
            .as_array()
            .expect("scopes should be an array in user response");

        assert!(
            roles
                .iter()
                .any(|role| role["id"] == hidden_role_id.to_string()),
            "Hidden role should be present for role_assign users"
        );
        assert!(
            scopes.iter().any(|scope| scope == "submission_review"),
            "Hidden-role-derived scope should be present for role_assign users"
        );
    };

    let anon_req = test::TestRequest::get()
        .uri(&format!("/users/{target_user}"))
        .to_request();
    let anon_resp = test::call_service(&app, anon_req).await;
    assert!(anon_resp.status().is_success());
    let anon_body: serde_json::Value = read_body_json(anon_resp).await;
    assert_hidden_data_is_not_exposed(&anon_body);

    let normal_req = test::TestRequest::get()
        .uri(&format!("/users/{target_user}"))
        .insert_header(("Authorization", format!("Bearer {normal_token}")))
        .to_request();
    let normal_resp = test::call_service(&app, normal_req).await;
    assert!(normal_resp.status().is_success());
    let normal_body: serde_json::Value = read_body_json(normal_resp).await;
    assert_hidden_data_is_not_exposed(&normal_body);

    let manager_req = test::TestRequest::get()
        .uri(&format!("/users/{target_user}"))
        .insert_header(("Authorization", format!("Bearer {manager_token}")))
        .to_request();
    let manager_resp = test::call_service(&app, manager_req).await;
    assert!(manager_resp.status().is_success());
    let manager_body: serde_json::Value = read_body_json(manager_resp).await;
    assert_hidden_data_is_exposed(&manager_body);
}

#[actix_web::test]
async fn list_users() {
    let (app, db, _, _) = init_test_app().await;
    let (_, username) = create_test_user(&db, None).await;
    create_test_user(&db, None).await;

    let req = test::TestRequest::get().uri("/users").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let users: serde_json::Value = read_body_json(resp).await;
    assert!(users["data"].as_array().unwrap().len() >= 2);

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{username}%"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let users: serde_json::Value = read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        users["data"].as_array().unwrap()[0]["global_name"],
        username
    );
}

#[actix_web::test]
async fn user_display_name_limit_applies_to_self_update() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserModify)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let update_payload = json!({
        "global_name": "This is a 35 characters or longer username that should return an error",
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {user_token}")))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::BAD_REQUEST,
        Some("The display name can at most be 35 characters long."),
    );
}

#[actix_web::test]
async fn list_users_with_filters() {
    let (app, db, _, _) = init_test_app().await;
    let (_, name) = create_test_user(&db, None).await;
    let (placeholder_id, _) = create_test_placeholder_user(&db).await;

    let req = test::TestRequest::get()
        .uri("/users?placeholder=true")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;
    assert!(users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["id"] == placeholder_id.to_string()));

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{name}%&per_page=1&page=1"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap().len(), 1);
}

#[actix_web::test]
async fn list_users_with_discord_id_filter() {
    let (app, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let discord_id = "1234567890";

    set_test_user_discord_id(&db, user_id, discord_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%25{discord_id}%25"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;
    assert!(users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["id"] == user_id.to_string()));
}

#[actix_web::test]
async fn list_users_with_account_status_filter() {
    let (app, db, auth, _) = init_test_app().await;
    let (banned_user_id, _) = create_test_user(&db, None).await;
    let (other_user_id, _) = create_test_user(&db, None).await;
    let (staff_user_id, _) = create_test_user(&db, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    set_test_user_account_status(&db, banned_user_id, AccountStatus::ListBanned).await;
    set_test_user_account_status(&db, other_user_id, AccountStatus::Normal).await;

    let req = test::TestRequest::get()
        .uri("/users?account_status=3")
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;

    assert!(users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["id"] == banned_user_id.to_string()));
    assert!(!users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["id"] == other_user_id.to_string()));
}

#[actix_web::test]
async fn list_users_hides_banned_users_for_unauthenticated_requests() {
    let (app, db, _, _) = init_test_app().await;
    let (banned_user_id, _) = create_test_user(&db, None).await;
    let (other_user_id, _) = create_test_user(&db, None).await;

    set_test_user_account_status(&db, banned_user_id, AccountStatus::ListBanned).await;
    set_test_user_account_status(&db, other_user_id, AccountStatus::Normal).await;

    let req = test::TestRequest::get()
        .uri("/users?account_status=3")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = read_body_json(resp).await;

    assert!(!users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["id"] == banned_user_id.to_string()));
    assert!(!users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|user| user["id"] == other_user_id.to_string()));
}

#[actix_web::test]
async fn upsert_creates_and_updates_user() {
    let (_, db, _, _) = init_test_app().await;

    let user_upsert = UserUpsert {
        username: "new_user".to_owned(),
        global_name: "New User".to_owned(),
        discord_id: Some("123".to_owned()),
        placeholder: false,
        country: Some(1),
        discord_avatar: Some("avatar".to_owned()),
        discord_avatar_decoration: Some("decoration".to_owned()),
        last_discord_avatar_update: Some(Utc::now().naive_utc()),
    };

    let created = User::upsert(&mut db.connection().unwrap(), user_upsert).expect("insert");
    assert_eq!(created.username, "new_user");
    assert_eq!(created.discord_id.as_deref(), Some("123"));
    assert_eq!(
        created.discord_avatar_decoration.as_deref(),
        Some("decoration")
    );

    let fetched = get_test_user(&db, created.id);
    assert_eq!(fetched.username, "new_user");

    let update_upsert = UserUpsert {
        username: "updated".to_owned(),
        global_name: "Updated".to_owned(),
        discord_id: Some("123".to_owned()),
        placeholder: false,
        country: Some(2),
        discord_avatar: Some("newavatar".to_owned()),
        discord_avatar_decoration: Some("newdecoration".to_owned()),
        last_discord_avatar_update: Some(Utc::now().naive_utc()),
    };

    let updated = User::upsert(&mut db.connection().unwrap(), update_upsert).expect("update");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.username, "updated");
    assert_eq!(updated.country, Some(1));
    assert_eq!(updated.global_name, "New User");
    assert_eq!(
        updated.discord_avatar_decoration.as_deref(),
        Some("newdecoration")
    );

    let remove_decoration = UserUpsert {
        username: "updated".to_owned(),
        global_name: "Updated".to_owned(),
        discord_id: Some("123".to_owned()),
        placeholder: false,
        country: Some(2),
        discord_avatar: Some("newavatar".to_owned()),
        discord_avatar_decoration: None,
        last_discord_avatar_update: Some(Utc::now().naive_utc()),
    };
    let updated =
        User::upsert(&mut db.connection().unwrap(), remove_decoration).expect("remove decoration");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.discord_avatar_decoration, None);
    assert_eq!(
        get_test_user(&db, created.id).discord_avatar_decoration,
        None
    );
}

#[actix_web::test]
async fn placeholder_random_username() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_user_id, _) = create_test_user(&db, Some(Permission::PlaceholderCreate)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let placeholder_payload = json!({
        "username": "test_placeholder"
    });

    let req = test::TestRequest::post()
        .uri("/users/placeholders")
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(&placeholder_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_http::StatusCode::CREATED);

    let created_user: serde_json::Value = read_body_json(resp).await;
    assert_ne!(created_user["username"], "test_placeholder");
}
