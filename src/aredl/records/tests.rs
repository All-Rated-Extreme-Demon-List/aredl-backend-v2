#[cfg(test)]
use {
    crate::{
        app_data::providers::{
            context::{GoogleAuthState, ProviderContext},
            list::youtube::YouTubeProvider,
            model::{Provider, ProviderRegistry},
        },
        aredl::{
            levels::test_utils::{create_test_level, create_test_level_with_record},
            records::test_utils::{
                create_test_record, create_two_test_records_with_different_timestamps,
            },
        },
        auth::{create_test_token, Permission},
        providers::{
            test_utils::{
                clear_google_env, mock_google_token_endpoint, mock_youtube_videos_endpoint,
                set_google_env,
            },
            VideoProvidersAppState,
        },
        schema::aredl::records,
        {test_utils::*, users::test_utils::create_test_user},
    },
    actix_web::test::{self, read_body_json},
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::prelude::*,
    serde_json::json,
    serial_test::serial,
    std::sync::Arc,
};

#[actix_web::test]
async fn create_record() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level = create_test_level(&db).await;

    let record_data = json!({
        "submitted_by": user_id.to_string(),
        "mobile": false,
        "level_id": level.to_string(),
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "is_verification": false,
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&record_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(
        body["submitted_by"].as_str().unwrap(),
        user_id.to_string().as_str(),
        "Names do not match!"
    )
}

#[actix_web::test]
async fn create_self_record_fails() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level = create_test_level(&db).await;

    let record_data = json!({
        "submitted_by": user_id.to_string(),
        "mobile": false,
        "level_id": level.to_string(),
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "is_verification": false,
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&record_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, 400, Some("You cannot create records for yourself")).await;
}

#[actix_web::test]
async fn get_record_list() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_level_with_record(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let length = body["data"].as_array().unwrap().len();

    assert_ne!(length, 0, "No records were returned!");
}

#[actix_web::test]
async fn update_record() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, record_id) = create_test_level_with_record(&db, user_id).await;
    let update_data = json!({
        "video_url": "https://updated.com"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body["video_url"].as_str().unwrap(),
        update_data["video_url"].as_str().unwrap(),
        "Videos do not match!"
    )
}

#[actix_web::test]
async fn update_self_record_fails() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, record_id) = create_test_level_with_record(&db, user_id).await;
    let update_data = json!({
        "video_url": "https://updated.com"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, 400, Some("You cannot update records for yourself")).await;
}

#[actix_web::test]
async fn get_own_records() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let (user_id_2, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    create_test_record(&db, user_id, level_id).await;
    create_test_record(&db, user_id_2, level_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
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
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (_, record_id) = create_test_level_with_record(&db, user_id).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn get_one_record() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (_, record_id) = create_test_level_with_record(&db, user_id).await;
    let req = test::TestRequest::get()
        .uri(&format!("/aredl/records/{}", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body["id"].as_str().unwrap().to_string(),
        record_id.to_string(),
        "Record IDs do not match!"
    )
}

#[actix_web::test]
async fn get_records_mobile_filter() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let (_, mobile_record) = create_test_level_with_record(&db, user_id).await;

    create_test_level_with_record(&db, user_id).await;
    diesel::update(records::table.filter(records::id.eq(mobile_record)))
        .set(records::mobile.eq(true))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/aredl/records?mobile_filter=true")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], mobile_record.to_string());
}

#[actix_web::test]
async fn get_records_level_filter() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let (level_one, record_one) = create_test_level_with_record(&db, user_id).await;
    create_test_level_with_record(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/records?level_filter={level_one}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_submitter_filter() {
    let (app, db, auth, _) = init_test_app().await;
    let (submitter_one, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let (submitter_two, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(submitter_one, &auth.jwt_encoding_key).unwrap();

    let (level_id, record_one) = create_test_level_with_record(&db, submitter_one).await;
    create_test_record(&db, submitter_two, level_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/records?submitter_filter={submitter_one}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], record_one.to_string());
}

#[actix_web::test]
async fn get_records_sort_oldest_created_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=OldestCreatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], older.to_string());
    assert_eq!(got[1], newer.to_string());
}

#[actix_web::test]
async fn get_records_sort_newest_created_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=NewestCreatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], newer.to_string());
    assert_eq!(got[1], older.to_string());
}

#[actix_web::test]
async fn get_records_sort_oldest_achieved_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=OldestAchievedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], older.to_string());
    assert_eq!(got[1], newer.to_string());
}

#[actix_web::test]
async fn get_records_sort_newest_achieved_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=NewestAchievedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], newer.to_string());
    assert_eq!(got[1], older.to_string());
}

#[actix_web::test]
async fn get_records_sort_oldest_updated_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=OldestUpdatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], older.to_string());
    assert_eq!(got[1], newer.to_string());
}

#[actix_web::test]
async fn get_records_sort_newest_updated_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_records_with_different_timestamps(&db, user_id).await;

    let req = test::TestRequest::get()
        .uri("/aredl/records?per_page=10&sort=NewestUpdatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();
    assert!(got.len() >= 2);
    assert_eq!(got[0], newer.to_string());
    assert_eq!(got[1], older.to_string());
}

#[actix_web::test]
#[serial]
async fn update_timestamp_endpoint_fetches_youtube_published_at() {
    clear_google_env();
    let server = MockServer::start_async().await;
    set_google_env(&server.base_url());
    mock_google_token_endpoint(&server, 3600, "test_access").await;

    let yt_mock =
        mock_youtube_videos_endpoint(&server, "xvFZjo5PgG0", "2009-10-25T06:57:33Z").await;

    let google_auth = GoogleAuthState::new()
        .await
        .expect("Failed to create GoogleAuthState");

    std::env::set_var("YOUTUBE_API_BASE_URL", server.base_url());

    let providers_app_state = Arc::new(VideoProvidersAppState::new(
        ProviderRegistry::new(vec![Arc::new(YouTubeProvider) as Arc<dyn Provider>]),
        ProviderContext {
            http: reqwest::Client::new(),
            google_auth: Some(Arc::new(google_auth)),
            twitch_auth: None,
        },
    ));

    let (app, db, auth, _) = init_test_app_with_providers(providers_app_state).await;

    let (submitter_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::RecordModify)).await;
    let token = create_test_token(moderator_id, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;

    let record_data = json!({
        "submitted_by": submitter_id.to_string(),
        "mobile": false,
        "level_id": level.to_string(),
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "is_verification": false,
        "raw_url": "https://raw.com"
    });

    let create_req = test::TestRequest::post()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&record_data)
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(
        create_resp.status().is_success(),
        "create status is {}",
        create_resp.status()
    );
    let created_body: serde_json::Value = read_body_json(create_resp).await;
    let record_id = created_body["id"]
        .as_str()
        .expect("created record must have id");

    let update_req = test::TestRequest::patch()
        .uri(&format!("/aredl/records/{}/update-timestamp", record_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let update_resp = test::call_service(&app, update_req).await;
    assert!(
        update_resp.status().is_success(),
        "update-timestamp status is {}",
        update_resp.status()
    );

    let updated_body: serde_json::Value = read_body_json(update_resp).await;

    assert_eq!(yt_mock.calls_async().await, 1);

    let got = updated_body["achieved_at"]
        .as_str()
        .expect("achieved_at must be a string");
    let got_dt: DateTime<Utc> = got.parse().expect("achieved_at must be RFC3339");
    let expected: DateTime<Utc> = "2009-10-25T06:57:33Z".parse().unwrap();
    assert_eq!(got_dt, expected);

    std::env::remove_var("YOUTUBE_API_BASE_URL");
    clear_google_env();
}
