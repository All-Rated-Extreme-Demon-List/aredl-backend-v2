#[cfg(test)]
use crate::{
    aredl::{
        packs::test_utils::create_test_pack,
        levels::test_utils::{create_test_level, create_test_level_with_record}
    },
    auth::{create_test_token, Permission},
    schema::aredl::{levels_created, pack_levels},
    
};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_level() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_data = json!({
        "name": "Test Level",
        "position": 1,
        "level_id": 123456,
        "publisher_id": user_id.to_string(),
        "legacy": false,
        "two_player": false
    });
    let req = test::TestRequest::post()
        .uri("/aredl/levels")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        level_data["level_id"].as_i64().unwrap(),
        body["level_id"].as_i64().unwrap(),
        "Level IDs do not match!"
    )
}

#[actix_web::test]
async fn list_levels() {
    let (app, mut conn, _, _) = init_test_app().await;
    create_test_level(&mut conn).await;
    create_test_level(&mut conn).await;
    create_test_level(&mut conn).await;
    let req = test::TestRequest::get().uri("/aredl/levels").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body.as_array().unwrap().len(),
        3,
        "Response doesn't have 3 levels!"
    );
    assert_eq!(
        body[0].as_object().unwrap()["position"].as_i64().unwrap(),
        1,
        "First level returned is not the top 1!"
    )
}

#[actix_web::test]
async fn update_level() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    let update_data = json!({
        "name": "Updated Level Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/levels/{}", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"].to_string(), update_data["name"].to_string())
}

#[actix_web::test]
async fn find_level() {
    let (app, mut conn, _, _) = init_test_app().await;
    let level_id = create_test_level(&mut conn).await;
    let req = test::TestRequest::get()
        .uri(&format!("/aredl/levels/{}", level_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        level_id.to_string(),
        body["id"].as_str().unwrap().to_string(),
        "IDs do not match!"
    )
}

#[actix_web::test]
async fn list_creators() {
    let (app, mut conn, _, _) = init_test_app().await;
    let level_id = create_test_level(&mut conn).await;
    let (creator_id, _) = create_test_user(&mut conn, None).await;

    diesel::insert_into(levels_created::table)
        .values((
            levels_created::level_id.eq(level_id),
            levels_created::user_id.eq(creator_id),
        ))
        .execute(&mut conn)
        .expect("Failed to add creator to level!");

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/levels/{}/creators", level_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body.as_array().unwrap()[0].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string(),
        creator_id.to_string(),
        "Creators do not match!"
    )
}

#[actix_web::test]
async fn set_creators() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    let new_creator_id = user_id;
    let req = test::TestRequest::post()
        .uri(&format!("/aredl/levels/{}/creators", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![new_creator_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.is_array(), "Response is not an array");
    assert_eq!(body[0].as_str().unwrap(), new_creator_id.to_string());
}

#[actix_web::test]
async fn add_and_remove_creators() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    // Add creator
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/levels/{}/creators", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![user_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.is_array(), "Response is not an array");
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u.as_str().unwrap() == user_id.to_string()));
    // Remove creator
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/levels/{}/creators", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![user_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.is_array(), "Response is not an array");
    assert!(
        !body
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["id"].as_str().unwrap() == user_id.to_string()),
        "Creator was not removed"
    );
}

#[actix_web::test]
async fn get_level_history() {
    let (app, mut conn, _, _) = init_test_app().await;
    let level_id = create_test_level(&mut conn).await;
    // move this level by placing a new one at #1
    let other_level = create_test_level(&mut conn).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/levels/{}/history", level_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    let move_entry = &body.as_array().unwrap()[0];
    let place_entry = &body.as_array().unwrap()[1];
    assert_eq!(move_entry["event"].as_str().unwrap(), "OtherPlaced");
    assert_eq!(
        move_entry["cause"].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string(),
        other_level.to_string()
    );
    assert_eq!(move_entry["position_diff"].as_i64().unwrap(), 1);
    assert_eq!(place_entry["event"].as_str().unwrap(), "Placed");
}

#[actix_web::test]
async fn get_level_pack() {
    let (app, mut conn, _, _) = init_test_app().await;
    let level = create_test_level(&mut conn).await;
    let pack = create_test_pack(&mut conn).await;
    // insert the pack into the level
    diesel::insert_into(pack_levels::table)
        .values((
            pack_levels::pack_id.eq(pack),
            pack_levels::level_id.eq(level),
        ))
        .execute(&mut conn)
        .expect("Failed to add level to pack!");

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/levels/{}/packs", level))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    let body: serde_json::Value = read_body_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1, "This level is in more than 1 pack!");
    assert_eq!(
        arr[0].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string(),
        pack.to_string(),
        "Pack IDs do not match!"
    )
}

#[actix_web::test]
async fn get_level_records() {
    let (app, mut conn, _, _) = init_test_app().await;
    let (submitter, _) = create_test_user(&mut conn, None).await;
    let (level_id, record_id) = create_test_level_with_record(&mut conn, submitter).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/levels/{}/records", level_id))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    let body: serde_json::Value = read_body_json(res).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1, "This level has more than 1 record!");
    assert_eq!(
        arr[0].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string(),
        record_id.to_string(),
        "Record IDs do not match!"
    )
}
