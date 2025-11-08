use crate::aredl::packs::test_utils::create_test_pack_tier;
#[cfg(test)]
use crate::auth::{create_test_token, Permission};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_pack_tier() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackTierModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let tier_data = json!({
        "name": "Test Tier",
        "color": "#abcdef",
        "placement": 1
    });

    let req = test::TestRequest::post()
        .uri("/aredl/pack-tiers")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&tier_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["name"].as_str().unwrap(),
        "Test Tier",
        "Names do not match!"
    )
}

#[actix_web::test]
async fn get_pack_tiers() {
    let (app, _, _, _) = init_test_app().await;
    let req = test::TestRequest::get()
        .uri("/aredl/pack-tiers")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn update_pack_tier() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackTierModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let tier_id = create_test_pack_tier(&mut conn).await;
    let update_data = json!({
        "name": "Updated Tier Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/pack-tiers/{}", tier_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["name"].as_str().unwrap(),
        "Updated Tier Name",
        "Names do not match!"
    )
}

#[actix_web::test]
async fn delete_pack_tier() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackTierModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let tier_id = create_test_pack_tier(&mut conn).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/pack-tiers/{}", tier_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}
