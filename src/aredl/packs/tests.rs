#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    db::DbConnection,
    schema::aredl::{packs, pack_tiers},
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::RunQueryDsl;
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use diesel::ExpressionMethods;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_pack_tier(conn: &mut DbConnection) -> Uuid {
    let tier_id = Uuid::new_v4();
    diesel::insert_into(pack_tiers::table)
        .values((
            pack_tiers::id.eq(tier_id),
            pack_tiers::name.eq("Test Tier"),
            pack_tiers::color.eq("#abcdef"),
            pack_tiers::placement.eq(1)
        ))
        .execute(conn)
        .expect("Failed to create test pack tier");
    tier_id

}

#[cfg(test)]
pub async fn create_test_pack(conn: &mut DbConnection) -> Uuid {
    let tier_id = create_test_pack_tier(conn).await;
    let pack_id = Uuid::new_v4();
    diesel::insert_into(packs::table)
        .values((
            packs::id.eq(pack_id),
            packs::name.eq("Test Pack"),
            packs::tier.eq(tier_id)
        ))
        .execute(conn)
        .expect("Failed to create test pack");

    pack_id
}

#[actix_web::test]
async fn create_pack() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let tier_id = create_test_pack_tier(&mut conn).await;
    let pack_data = json!({
        "name": "Test Pack",
        "tier": tier_id.to_string()
    });
    let req = test::TestRequest::post()
        .uri("/aredl/packs")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&pack_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(pack_data["name"].as_str().unwrap(), body["name"].as_str().unwrap(), "Names do not match!")
}

#[actix_web::test]
async fn update_pack() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&mut conn).await;
    let update_data = json!({
        "name": "Updated Pack Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/packs/{}", pack_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(update_data["name"].as_str().unwrap(), body["name"].as_str().unwrap(), "Names do not match!")
}

#[actix_web::test]
async fn remove_pack() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&mut conn).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/packs/{}", pack_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn add_level_to_pack() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&mut conn).await;
    let level = create_test_level(&mut conn).await;
    
    let level_data = json!([level]);

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/packs/{}/levels", pack_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    let added_level = body[0].as_object().unwrap()["id"].as_str().unwrap().to_string();
    assert_eq!(added_level, level.to_string())
}

#[actix_web::test]
async fn set_pack_levels() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&mut conn).await;
    let level = create_test_level(&mut conn).await;
    
    let level_data = json!([level]);

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/packs/{}/levels", pack_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    let added_level = body[0].as_object().unwrap()["id"].as_str().unwrap().to_string();
    assert_eq!(added_level, level.to_string())
}

#[actix_web::test]
async fn remove_level_from_pack() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::PackModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let pack_id = create_test_pack(&mut conn).await;
    let level = create_test_level(&mut conn).await;
    
    let level_data = json!([level]);

    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/packs/{}/levels", pack_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    assert_eq!(body.as_array().unwrap().len(), 0)
}
