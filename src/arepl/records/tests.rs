#[cfg(test)]
use {
    crate::{
        arepl::{
            levels::test_utils::{create_test_level, create_test_level_with_record},
            records::test_utils::create_test_record,
        },
        auth::{create_test_token, Permission},
        schema::arepl::records,
        {test_utils::*, users::test_utils::create_test_user},
    },
    actix_web::test,
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    serde_json::json,
};

#[actix_web::test]
async fn create_record() {
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (user_id_2, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    create_test_record(&mut conn, user_id, level_id).await;
    create_test_record(&mut conn, user_id_2, level_id).await;

    let req = test::TestRequest::get()
        .uri("/arepl/records/@me")
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
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
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

#[actix_web::test]
async fn get_records_mobile_filter() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    create_test_level_with_record(&mut conn, user_id).await;
    let (_, mobile_record) = create_test_level_with_record(&mut conn, user_id).await;
    diesel::update(records::table.filter(records::id.eq(mobile_record)))
        .set(records::mobile.eq(true))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/arepl/records?mobile_filter=true")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], mobile_record.to_string());
}

#[actix_web::test]
async fn get_records_level_filter() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (level_one, record_one) = create_test_level_with_record(&mut conn, user_id).await;
    create_test_level_with_record(&mut conn, user_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/records?level_filter={level_one}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_submitter_filter() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (submitter_one, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (submitter_two, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(submitter_one, &auth.jwt_encoding_key).unwrap();

    let (level_id, record_one) = create_test_level_with_record(&mut conn, submitter_one).await;
    create_test_record(&mut conn, submitter_two, level_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/records?submitter_filter={submitter_one}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_reviewer_filter() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (reviewer_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (_, reviewed_record) = create_test_level_with_record(&mut conn, user_id).await;
    create_test_level_with_record(&mut conn, user_id).await;
    diesel::update(records::table.filter(records::id.eq(reviewed_record)))
        .set(records::reviewer_id.eq(Some(reviewer_id)))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/records?reviewer_filter={reviewer_id}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], reviewed_record.to_string());
}

#[actix_web::test]
async fn get_records_mobile_filter_full() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let (_, mobile_record) = create_test_level_with_record(&mut conn, user_id).await;

    create_test_level_with_record(&mut conn, user_id).await;
    diesel::update(records::table.filter(records::id.eq(mobile_record)))
        .set(records::mobile.eq(true))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/arepl/records/full?mobile_filter=true")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], mobile_record.to_string());
}

#[actix_web::test]
async fn get_records_level_filter_full() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let (level_one, record_one) = create_test_level_with_record(&mut conn, user_id).await;
    create_test_level_with_record(&mut conn, user_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/records/full?level_filter={level_one}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_submitter_filter_full() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (submitter_one, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (submitter_two, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(submitter_one, &auth.jwt_encoding_key).unwrap();

    let (level_id, record_one) = create_test_level_with_record(&mut conn, submitter_one).await;
    create_test_record(&mut conn, submitter_two, level_id).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/arepl/records/full?submitter_filter={submitter_one}"
        ))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_reviewer_filter_full() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let (reviewer_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (_, reviewed_record) = create_test_level_with_record(&mut conn, user_id).await;
    create_test_level_with_record(&mut conn, user_id).await;
    diesel::update(records::table.filter(records::id.eq(reviewed_record)))
        .set(records::reviewer_id.eq(Some(reviewer_id)))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/arepl/records/full?reviewer_filter={reviewer_id}"
        ))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], reviewed_record.to_string());
}
