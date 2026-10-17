#[cfg(test)]
use {
    crate::{
        auth::create_test_token,
        list::{
            levels::test_utils::{
                add_test_level_creators, create_test_level_with_record,
                create_test_level_with_record_for_list, get_test_level,
            },
            List,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn get_user_badges() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (level_id, record_id) = create_test_level_with_record(&db, user_id).await;
    let level_name = get_test_level(&db, level_id).await.name;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let grant_req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(json!({
            "badge_code": "classic.hardest_level.10",
            "record_id": record_id,
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
        badge["badge_code"] == "classic.hardest_level.10"
            && badge["record"]["id"] == record_id.to_string()
            && badge["record"]["level"] == level_name
            && badge["record"]["achieved_at"].is_string()
            && badge.get("description").is_none()
    }));
}

#[actix_web::test]
async fn grant_user_badge() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (level_id, _) = create_test_level_with_record(&db, user_id).await;
    let level_name = get_test_level(&db, level_id).await.name;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let staff_token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(json!({
            "badge_code": "global.level_completion.5",
            "level_ids": [level_id],
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let badges: serde_json::Value = read_body_json(resp).await;
    assert!(badges.as_array().unwrap().iter().any(|badge| {
        badge["badge_code"] == "global.level_completion.5"
            && badge["levels"]
                .as_array()
                .is_some_and(|levels| levels.iter().any(|level| level == &level_name))
            && badge.get("description").is_none()
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
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(json!({
            "badge_code": "global.level_completion.5",
        }))
        .to_request();

    let grant_resp = test::call_service(&app, grant_req).await;
    assert!(grant_resp.status().is_success());

    let remove_req = test::TestRequest::delete()
        .uri(&format!("/users/{user_id}/badges"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
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
        let (level_id, _) = create_test_level_with_record(&db, user_id).await;
        add_test_level_creators(&db, level_id, &[user_id]).await;
    }

    let req = test::TestRequest::post()
        .uri(&format!("/users/{user_id}/badges/sync"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let badges: serde_json::Value = read_body_json(resp).await;
    assert!(badges
        .as_array()
        .unwrap()
        .iter()
        .any(|badge| badge["badge_code"] == "global.level_completion.5"));

    let creator = badges
        .as_array()
        .unwrap()
        .iter()
        .find(|badge| badge["badge_code"] == "global.creator")
        .unwrap();
    assert_eq!(creator["levels"].as_array().unwrap().len(), 5);
    let unlocked_at = creator["unlocked_at"].clone();

    let (level_id, _) = create_test_level_with_record(&db, user_id).await;
    add_test_level_creators(&db, level_id, &[user_id]).await;
    let level_name = get_test_level(&db, level_id).await.name;
    let req = test::TestRequest::post()
        .uri(&format!("/users/{user_id}/badges/sync"))
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let badges: serde_json::Value = read_body_json(resp).await;
    let creator = badges
        .as_array()
        .unwrap()
        .iter()
        .find(|badge| badge["badge_code"] == "global.creator")
        .unwrap();
    assert_eq!(creator["unlocked_at"], unlocked_at);
    assert_eq!(creator["levels"].as_array().unwrap().len(), 6);
    assert!(creator["levels"]
        .as_array()
        .unwrap()
        .iter()
        .any(|level| level == &level_name));
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
        .insert_header(("Authorization", format!("Bearer {staff_token}")))
        .set_json(json!({
            "badge_code": "global.invalid_badge",
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::BAD_REQUEST,
        Some("Unknown badge code: global.invalid_badge"),
    );
}

#[actix_web::test]
async fn sync_user_badges_combines_classic_and_platformer_records() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(crate::auth::Permission::UserModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    for _ in 0..4 {
        create_test_level_with_record(&db, user_id).await;
    }
    let (platformer_level, platformer_record) =
        create_test_level_with_record_for_list(&db, List::Platformer, user_id).await;
    let platformer_level_name = get_test_level(&db, platformer_level).await.name;

    let req = test::TestRequest::post()
        .uri(&format!("/users/{user_id}/badges/sync"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let badges: serde_json::Value = read_body_json(resp).await;
    let badges = badges.as_array().unwrap();
    assert!(badges
        .iter()
        .any(|badge| badge["badge_code"] == "global.level_completion.5"));
    assert!(!badges
        .iter()
        .any(|badge| badge["badge_code"] == "global.level_completion.10"));
    assert!(badges
        .iter()
        .any(|badge| badge["badge_code"] == "classic.hardest_level.10"));
    assert!(badges
        .iter()
        .any(|badge| badge["badge_code"] == "platformer.hardest_level.10"
            && badge["record"]["id"] == platformer_record.to_string()
            && badge["record"]["level"] == platformer_level_name));
    assert!(badges
        .iter()
        .any(|badge| badge["badge_code"] == "platformer.fastest_time"));
}
