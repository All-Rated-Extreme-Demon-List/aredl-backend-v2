use {
    crate::arepl::{
        levels::test_utils::add_test_level_to_pack,
        records::test_utils::{create_test_record, set_test_record_achieved_at},
    },
    chrono::{DateTime, Utc},
};
#[cfg(test)]
use {
    crate::{
        arepl::levels::test_utils::create_test_level,
        arepl::packs::test_utils::{create_test_pack, create_test_pack_tier},
        auth::{create_test_token, Permission},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn create_pack() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let tier_id = create_test_pack_tier(&db).await;
    let pack_data = json!({
        "name": "Test Pack",
        "tier": tier_id.to_string()
    });
    let req = test::TestRequest::post()
        .uri("/arepl/packs")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&pack_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        pack_data["name"].as_str().unwrap(),
        body["name"].as_str().unwrap(),
        "Names do not match!"
    );
}

#[actix_web::test]
async fn update_pack() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&db).await;
    let update_data = json!({
        "name": "Updated Pack Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/arepl/packs/{pack_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        update_data["name"].as_str().unwrap(),
        body["name"].as_str().unwrap(),
        "Names do not match!"
    );
}

#[actix_web::test]
async fn remove_pack() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&db).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/arepl/packs/{pack_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn add_level_to_pack() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&db).await;
    let level = create_test_level(&db).await;

    let level_data = json!([level]);

    let req = test::TestRequest::patch()
        .uri(&format!("/arepl/packs/{pack_id}/levels"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let added_level = body[0].as_object().unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(added_level, level.to_string());
}

#[actix_web::test]
async fn set_pack_levels() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&db).await;
    let level = create_test_level(&db).await;

    let level_data = json!([level]);

    let req = test::TestRequest::post()
        .uri(&format!("/arepl/packs/{pack_id}/levels"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let added_level = body[0].as_object().unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(added_level, level.to_string());
}

#[actix_web::test]
async fn remove_level_from_pack() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::PackModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&db).await;
    let level = create_test_level(&db).await;

    let level_data = json!([level]);

    let req = test::TestRequest::delete()
        .uri(&format!("/arepl/packs/{pack_id}/levels"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn get_pack_victors_returns_pack_victors_in_completion_order() {
    let (app, db, _, _) = init_test_app().await;

    let pack_id = create_test_pack(&db).await;
    let other_pack_id = create_test_pack(&db).await;
    let level_id = create_test_level(&db).await;
    let other_level_id = create_test_level(&db).await;
    let (first_user, _) = create_test_user(&db, None).await;
    let (second_user, _) = create_test_user(&db, None).await;
    let (other_user, _) = create_test_user(&db, None).await;

    let first_completed_at: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let second_completed_at: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();
    let other_completed_at: DateTime<Utc> = "2022-01-01T00:00:00Z".parse().unwrap();

    add_test_level_to_pack(&db, level_id, pack_id);
    add_test_level_to_pack(&db, other_level_id, other_pack_id);

    let first_record = create_test_record(&db, first_user, level_id).await;
    let second_record = create_test_record(&db, second_user, level_id).await;
    let other_record = create_test_record(&db, other_user, other_level_id).await;

    set_test_record_achieved_at(&db, second_record, second_completed_at).await;
    set_test_record_achieved_at(&db, first_record, first_completed_at).await;
    set_test_record_achieved_at(&db, other_record, other_completed_at).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/packs/{pack_id}/victors"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let victors = body.as_array().unwrap();

    assert_eq!(victors.len(), 2);
    assert_eq!(
        victors[0]["user"]["id"].as_str(),
        Some(first_user.to_string().as_str())
    );
    assert_eq!(
        victors[0]["completed_at"].as_str(),
        Some("2020-01-01T00:00:00")
    );
    assert_eq!(
        victors[1]["user"]["id"].as_str(),
        Some(second_user.to_string().as_str())
    );
    assert_eq!(
        victors[1]["completed_at"].as_str(),
        Some("2021-01-01T00:00:00")
    );
}
