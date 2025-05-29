#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    schema::aredl::{shifts, recurrent_shifts},
    db::DbConnection,
    aredl::shifts::Weekday,
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

async fn create_test_recurring_shift(conn: &mut DbConnection, user_id: Uuid) -> Uuid {
    diesel::insert_into(recurrent_shifts::table)
        .values((
            recurrent_shifts::user_id.eq(user_id),
            recurrent_shifts::start_hour.eq(12),
            recurrent_shifts::target_count.eq(20),
            recurrent_shifts::duration.eq(1),
            recurrent_shifts::weekday.eq(Weekday::Friday)
        ))
        .returning(recurrent_shifts::id)
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

#[actix_web::test]
async fn create_recurring_shift() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let insert_data = json!({
        "user_id": user_id,
        "weekday": "Friday",
        "start_hour": 12,
        "duration": 1,
        "target_count": 20
    });
    let req = test::TestRequest::post()
        .uri("/aredl/shifts/recurring")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&insert_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["user_id"].as_str().unwrap(), user_id.to_string());
}

#[actix_web::test]
async fn list_recurring_shifts() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_recurring_shift(&mut conn, user_id).await;
    let req = test::TestRequest::get()
        .uri("/aredl/shifts/recurring")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.as_array().unwrap().iter().any(|x| x["user"]["id"].as_str().unwrap() == user_id.to_string()));
}

#[actix_web::test]
async fn patch_recurring_shift() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let recurring_id = create_test_recurring_shift(&mut conn, user_id).await;
    let patch_data = json!({
        "target_count": 42
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/shifts/recurring/{}", recurring_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&patch_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["target_count"].as_i64().unwrap(), 42);
}

#[actix_web::test]
async fn delete_recurring_shift() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let recurring_id = create_test_recurring_shift(&mut conn, user_id).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/shifts/recurring/{}", recurring_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), recurring_id.to_string());
}
