use crate::arepl::levels::test_utils::{create_test_level, create_test_level_with_record};
use crate::arepl::records::test_utils::create_test_record;
#[cfg(test)]
use crate::auth::{create_test_token, Permission};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;

#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level = create_test_level(&mut conn).await;

    let record_data = json!({
        "submitted_by": user_id.to_string(),
        "mobile": false,
        "level_id": level.to_string(),
        "video_url": "https://video.com",
        "completion_time": 1235854,
        "is_verification": false,
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/arepl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&record_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["submitted_by"].as_str().unwrap(),
        user_id.to_string().as_str(),
        "Names do not match!"
    )
}

#[actix_web::test]
async fn get_record_list() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_level_with_record(&mut conn, user_id).await;

    let req = test::TestRequest::get()
        .uri("/arepl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    let length = body["data"].as_array().unwrap().len();

    assert_ne!(length, 0, "No records were returned!");
}

#[actix_web::test]
async fn update_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, record_id) = create_test_level_with_record(&mut conn, user_id).await;
    let update_data = json!({
        "video_url": "https://updated.com"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/arepl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["video_url"].as_str().unwrap(),
        update_data["video_url"].as_str().unwrap(),
        "Videos do not match!"
    )
}

#[actix_web::test]
async fn get_own_records() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (user_id_2, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    create_test_record(&mut conn, user_id, level_id).await;
    create_test_record(&mut conn, user_id_2, level_id).await;

    let req = test::TestRequest::get()
        .uri("/arepl/records/@me")
        // should return user 1's records only
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_ne!(
        body["data"].as_array().unwrap().len(),
        0,
        "Did not return any data!"
    );
    assert_ne!(
        body["data"].as_array().unwrap().len(),
        2,
        "Returned both users' records!"
    );
    let submitter_id = body["data"].as_array().unwrap()[0].as_object().unwrap()["submitted_by"]
        .as_object()
        .unwrap()["id"]
        .as_str()
        .unwrap();

    assert_eq!(
        submitter_id,
        user_id.to_string().as_str(),
        "Submitters do not match!"
    )
}

#[actix_web::test]
async fn delete_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (_, record_id) = create_test_level_with_record(&mut conn, user_id).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/arepl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn get_one_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (_, record_id) = create_test_level_with_record(&mut conn, user_id).await;
    let req = test::TestRequest::get()
        .uri(&format!("/arepl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["id"].as_str().unwrap().to_string(),
        record_id.to_string(),
        "Record IDs do not match!"
    )
}
