#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    schema::aredl::shifts,
    db::DbConnection,
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{RunQueryDsl, ExpressionMethods};
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;
#[cfg(test)]
use chrono::Utc;

#[cfg(test)]
async fn create_test_shift(conn: &mut DbConnection, user_id: Uuid, should_start_immediately: bool) -> Uuid {
    let start_time = match should_start_immediately {
        true => Utc::now(),
        false => Utc::now() + chrono::Duration::hours(1),
    };
    
    diesel::insert_into(shifts::table)
        .values((
            shifts::user_id.eq(user_id),
            shifts::target_count.eq(20),
            shifts::start_at.eq(start_time),
            shifts::end_at.eq(start_time + chrono::Duration::hours(4))
        ))
        .returning(shifts::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test shift")
}

#[actix_web::test]
async fn get_shifts_list() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_shift(&mut conn, user_id, false).await;
    let req = test::TestRequest::get()
        .uri("/aredl/shifts")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_ne!(body["data"].as_array().unwrap().len(), 0)
}

#[actix_web::test]
async fn get_my_shifts() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_shift(&mut conn, user_id, false).await;
    let req = test::TestRequest::get()
        .uri("/aredl/shifts/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(
        body["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(
            |x| x["user"].as_object().unwrap()
                ["id"].as_str().unwrap().to_string() ==
                user_id.to_string()
        )
    )
}

#[actix_web::test]
async fn patch_shift() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let shift_id = create_test_shift(&mut conn, user_id, false).await;
    let patch_data = json!({
        "status": "Completed"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/shifts/{}", shift_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&patch_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(patch_data["status"].as_str().unwrap(), body["status"].as_str().unwrap(), "Statuses do not match!")
}

#[actix_web::test]
async fn delete_shift() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let shift_id = create_test_shift(&mut conn, user_id, false).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/shifts/{}", shift_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

