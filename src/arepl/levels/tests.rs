use {
    crate::arepl::levels::test_utils::refresh_test_position_history,
    chrono::{DateTime, Utc},
    std::time::Duration,
    tokio::time::sleep,
};
#[cfg(test)]
use {
    crate::{
        arepl::{
            levels::test_utils::{create_test_level, create_test_level_with_record},
            packs::test_utils::create_test_pack,
        },
        auth::{create_test_token, Permission},
        schema::arepl::{levels_created, pack_levels, position_history},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    serde_json::json,
};

#[actix_web::test]
async fn create_level() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
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
        .uri("/arepl/levels")
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
    let (app, db, _, _) = init_test_app().await;
    create_test_level(&db).await;
    create_test_level(&db).await;
    create_test_level(&db).await;
    let req = test::TestRequest::get().uri("/arepl/levels").to_request();
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
    );
    assert!(
        body[0]
            .as_object()
            .unwrap()
            .get("completed_by_user")
            .is_none(),
        "Unauthenticated response should not include completed_by_user"
    );
}

#[actix_web::test]
async fn list_levels_with_completion_status_when_authenticated() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (completed_level_id, _) = create_test_level_with_record(&db, user_id).await;
    let incomplete_level_id = create_test_level(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/levels")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let levels = body.as_array().expect("Response should be an array");

    let completed_level = levels
        .iter()
        .find(|level| level["id"].as_str() == Some(completed_level_id.to_string().as_str()))
        .expect("Completed level should be present");
    assert_eq!(
        completed_level["completed_by_user"].as_bool(),
        Some(true),
        "Completed level should be marked as completed"
    );

    let incomplete_level = levels
        .iter()
        .find(|level| level["id"].as_str() == Some(incomplete_level_id.to_string().as_str()))
        .expect("Incomplete level should be present");
    assert_eq!(
        incomplete_level["completed_by_user"].as_bool(),
        Some(false),
        "Incomplete level should be marked as not completed"
    );
}

#[actix_web::test]
async fn list_levels_at_timestamp() {
    let (app, db, _, _) = init_test_app().await;

    let first_level = create_test_level(&db).await;
    refresh_test_position_history(&db).await;

    let at: DateTime<Utc> = position_history::table
        .filter(position_history::affected_level.eq(first_level))
        .order_by(position_history::i.desc())
        .select(position_history::created_at)
        .first(&mut db.connection().unwrap())
        .expect("Failed to fetch first level position history timestamp");

    sleep(Duration::from_millis(50)).await;
    let second_level = create_test_level(&db).await;
    refresh_test_position_history(&db).await;

    let second_at: DateTime<Utc> = position_history::table
        .filter(position_history::affected_level.eq(second_level))
        .order_by(position_history::i.desc())
        .select(position_history::created_at)
        .first(&mut db.connection().unwrap())
        .expect("Failed to fetch second level position history timestamp");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/arepl/levels?at={}",
            at.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let previous_list = body
        .as_array()
        .expect("Time machine response should be an array");

    let first_level = first_level.to_string();
    let second_level = second_level.to_string();

    assert!(
        previous_list
            .iter()
            .any(|entry| entry["id"].as_str() == Some(first_level.as_str())),
        "time machine response should contain the first level; first_at={at:?}; second_at={second_at:?}"
    );
    assert!(
        !previous_list
            .iter()
            .any(|entry| entry["id"].as_str() == Some(second_level.as_str())),
        "time machine response should not contain the second level; first_at={at:?}; second_at={second_at:?}"
    );
}

#[actix_web::test]
async fn update_level() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let update_data = json!({
        "name": "Updated Level Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/arepl/levels/{}", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["name"].to_string(), update_data["name"].to_string())
}

#[actix_web::test]
async fn find_level() {
    let (app, db, _, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let req = test::TestRequest::get()
        .uri(&format!("/arepl/levels/{}", level_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        level_id.to_string(),
        body["id"].as_str().unwrap().to_string(),
        "IDs do not match!"
    )
}

#[actix_web::test]
async fn list_creators() {
    let (app, db, _, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (creator_id, _) = create_test_user(&db, None).await;

    diesel::insert_into(levels_created::table)
        .values((
            levels_created::level_id.eq(level_id),
            levels_created::user_id.eq(creator_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to add creator to level!");

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/levels/{}/creators", level_id))
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
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let new_creator_id = user_id;
    let req = test::TestRequest::post()
        .uri(&format!("/arepl/levels/{}/creators", level_id))
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
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    // Add creator
    let req = test::TestRequest::patch()
        .uri(&format!("/arepl/levels/{}/creators", level_id))
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
        .uri(&format!("/arepl/levels/{}/creators", level_id))
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
    let (app, db, _, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    // move this level by placing a new one at #1
    let other_level = create_test_level(&db).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/levels/{}/history", level_id))
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
    let (app, db, _, _) = init_test_app().await;
    let level = create_test_level(&db).await;
    let pack = create_test_pack(&db).await;
    // insert the pack into the level
    diesel::insert_into(pack_levels::table)
        .values((
            pack_levels::pack_id.eq(pack),
            pack_levels::level_id.eq(level),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to add level to pack!");

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/levels/{}/packs", level))
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
    let (app, db, _, _) = init_test_app().await;
    let (submitter, _) = create_test_user(&db, None).await;
    let (level_id, record_id) = create_test_level_with_record(&db, submitter).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/levels/{}/records", level_id))
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
