#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        clans::test_utils::{create_test_clan, create_test_clan_member},
        schema::clan_members,
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    serde_json::json,
    uuid::Uuid,
};

#[actix_web::test]
async fn create_and_join() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "Test Clan", "tag": "TC"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let clan: serde_json::Value = read_body_json(resp).await;
    let clan_id = Uuid::parse_str(clan["id"].as_str().unwrap()).unwrap();
    let count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn list_clans() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "List Clan", "tag": "LC"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let _resp = test::call_service(&app, req).await;

    let req = test::TestRequest::get().uri("/clans").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[actix_web::test]
async fn create_empty_clan() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "Empty Clan", "tag": "EC"});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn update_clan() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "Updated"});
    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let clan: serde_json::Value = read_body_json(resp).await;
    assert_eq!(clan["global_name"], "Updated");
}

#[actix_web::test]
async fn delete_clan() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    create_test_clan_member(&db, clan_id, staff_id, 2).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn delete_clan_with_multiple_members_forbidden() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You cannot delete a clan unless you're the only member left in it."),
    )
    .await;
}

#[actix_web::test]
async fn create_clan_name_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let name = "a".repeat(101);
    let payload = json!({"global_name": name, "tag": "TL"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan name can at most be 100 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_empty_clan_name_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let name = "a".repeat(101);
    let payload = json!({"global_name": name, "tag": "TL"});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan name can at most be 100 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_clan_tag_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "TagLong", "tag": "TOOLONG"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan tag can at most be 5 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_empty_clan_tag_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "TagLong", "tag": "TOOLONG"});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan tag can at most be 5 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_clan_description_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let desc = "d".repeat(301);
    let payload = json!({"global_name": "DescLong", "tag": "DL", "description": desc});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan description can at most be 300 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_empty_clan_description_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let desc = "d".repeat(301);
    let payload = json!({"global_name": "DescLong", "tag": "DL", "description": desc});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan description can at most be 300 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn create_clan_already_in_clan() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, user_id, 0).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "NewClan", "tag": "NC"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, 400, Some("You are already in a clan.")).await;
}

#[actix_web::test]
async fn update_clan_name_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let name = "n".repeat(101);
    let payload = json!({"global_name": name});
    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan name can at most be 100 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn update_clan_tag_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"tag": "LONGER"});
    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan tag can at most be 5 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn update_clan_description_too_long() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let desc = "x".repeat(301);
    let payload = json!({"description": desc});
    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        400,
        Some("The clan description can at most be 300 characters long."),
    )
    .await;
}

#[actix_web::test]
async fn find_clan_with_filter() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "Alpha Clan", "tag": "ALP"});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let _ = test::call_service(&app, req).await;

    let payload = json!({"global_name": "Beta Clan", "tag": "BET"});
    let req = test::TestRequest::post()
        .uri("/clans/placeholder")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let _ = test::call_service(&app, req).await;

    let req = test::TestRequest::get()
        .uri("/clans?name_filter=%25Alpha%25&per_page=1&page=1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["global_name"], "Alpha Clan");
}
