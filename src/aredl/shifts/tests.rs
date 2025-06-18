#[cfg(test)]
use crate::aredl::shifts::test_utils::{create_test_recurring_shift, create_test_shift};
#[cfg(test)]
use crate::auth::{create_test_token, Permission};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn get_shifts_list() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_shift(&mut conn, user_id, false).await;
    let req = test::TestRequest::get()
        .uri("/aredl/shifts/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["user"].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
            == user_id.to_string()))
}

#[actix_web::test]
async fn patch_shift() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    assert_eq!(
        patch_data["status"].as_str().unwrap(),
        body["status"].as_str().unwrap(),
        "Statuses do not match!"
    )
}

#[actix_web::test]
async fn delete_shift() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_recurring_shift(&mut conn, user_id).await;
    let req = test::TestRequest::get()
        .uri("/aredl/shifts/recurring")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["user"]["id"].as_str().unwrap() == user_id.to_string()));
}

#[actix_web::test]
async fn patch_recurring_shift() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
