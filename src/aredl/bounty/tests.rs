#[cfg(test)]
use {
    crate::{
        app_data::providers::{
            context::{GoogleAuthState, ProviderContext},
            list::youtube::YouTubeProvider,
            model::{Provider, ProviderRegistry},
        },
        aredl::{
            bounty::test_utils::{
                count_test_bounty_completions, create_test_bounty, create_test_bounty_completion,
                fetch_test_bounty, fetch_test_record, find_test_bounty,
                set_test_record_achieved_at,
            },
            levels::test_utils::create_test_level,
            records::test_utils::create_test_record,
        },
        auth::{create_test_token, Permission},
        providers::{
            test_utils::{
                clear_google_env, mock_google_token_endpoint, mock_youtube_videos_endpoint,
                set_google_env,
            },
            VideoProvidersAppState,
        },
        schema::aredl::{bounties, bounty_completed, records},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    chrono::{DateTime, Duration as ChronoDuration, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::prelude::*,
    serde_json::json,
    serial_test::serial,
    std::sync::Arc,
    tokio::time::{sleep, Duration},
    uuid::Uuid,
};

#[actix_web::test]
async fn bounty_board_permissions_and_validation() {
    let (app, db, auth, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (regular_id, _) = create_test_user(&db, None).await;
    let (manager_id, _) = create_test_user(&db, Some(Permission::BountyManage)).await;
    let regular_token = create_test_token(regular_id, &auth.jwt_encoding_key).unwrap();
    let manager_token = create_test_token(manager_id, &auth.jwt_encoding_key).unwrap();

    let valid_body = json!({
        "level_id": level_id,
        "bounty_type": "Bounty",
        "bounty_difficulty": "Medium",
        "start_date": "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        "end_date": "2026-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        "target_submissions": 2,
        "is_target_public": false
    });

    let forbidden_create = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", regular_token)))
            .set_json(&valid_body)
            .to_request(),
    )
    .await;
    assert_error_response(
        forbidden_create,
        403,
        Some("You do not have the required permission (bounty_manage) to access this endpoint"),
    )
    .await;

    let invalid_target = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .set_json(json!({
                "level_id": level_id,
                "bounty_type": "Bounty",
                "bounty_difficulty": "Medium",
                "start_date": "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                "end_date": null,
                "target_submissions": 0,
                "is_target_public": false
            }))
            .to_request(),
    )
    .await;
    assert_error_response(
        invalid_target,
        400,
        Some("Target submissions must be a positive integer."),
    )
    .await;

    let invalid_dates = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .set_json(json!({
                "level_id": level_id,
                "bounty_type": "Bounty",
                "bounty_difficulty": "Medium",
                "start_date": "2026-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                "end_date": "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                "target_submissions": null,
                "is_target_public": false
            }))
            .to_request(),
    )
    .await;
    assert_error_response(
        invalid_dates,
        400,
        Some("End date must be after start date."),
    )
    .await;

    let create_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .set_json(&valid_body)
            .to_request(),
    )
    .await;
    assert!(
        create_resp.status().is_success(),
        "create bounty status is {}",
        create_resp.status()
    );
    let created: serde_json::Value = read_body_json(create_resp).await;
    let bounty_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();

    let invalid_update = test::call_service(
        &app,
        test::TestRequest::patch()
            .uri(&format!("/aredl/bounty-board/{}", bounty_id))
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .set_json(
                json!({ "start_date": "2026-03-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap() }),
            )
            .to_request(),
    )
    .await;
    assert_error_response(
        invalid_update,
        400,
        Some("End date must be after start date."),
    )
    .await;

    let forbidden_delete = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/aredl/bounty-board/{}", bounty_id))
            .insert_header(("Authorization", format!("Bearer {}", regular_token)))
            .to_request(),
    )
    .await;
    assert_error_response(
        forbidden_delete,
        403,
        Some("You do not have the required permission (bounty_manage) to access this endpoint"),
    )
    .await;

    let delete_resp = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/aredl/bounty-board/{}", bounty_id))
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .to_request(),
    )
    .await;
    assert!(
        delete_resp.status().is_success(),
        "delete bounty status is {}",
        delete_resp.status()
    );
}

#[actix_web::test]
async fn bounty_board_visibility_and_completed_by_user() {
    let (app, db, auth, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    let (manager_id, _) = create_test_user(&db, Some(Permission::BountyManage)).await;
    let user_token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let manager_token = create_test_token(manager_id, &auth.jwt_encoding_key).unwrap();
    let start = Utc::now() - ChronoDuration::days(1);
    let end = Some(Utc::now() + ChronoDuration::days(1));

    let hidden_bounty = create_test_bounty(&db, level_id, start, end, Some(3), false).await;
    let public_bounty = create_test_bounty(&db, level_id, start, end, Some(5), true).await;
    create_test_bounty_completion(&db, hidden_bounty.id, user_id).await;

    let anon_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/aredl/bounty-board")
            .to_request(),
    )
    .await;
    assert!(anon_resp.status().is_success());
    let anon_body: serde_json::Value = read_body_json(anon_resp).await;
    let anon_hidden = find_test_bounty(&anon_body, hidden_bounty.id);
    let anon_public = find_test_bounty(&anon_body, public_bounty.id);
    assert!(anon_hidden.get("completed_by_user").is_none());
    assert!(anon_hidden["target_submissions"].is_null());
    assert_eq!(anon_public["target_submissions"], 5);

    let user_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", user_token)))
            .to_request(),
    )
    .await;
    assert!(user_resp.status().is_success());
    let user_body: serde_json::Value = read_body_json(user_resp).await;
    let user_hidden = find_test_bounty(&user_body, hidden_bounty.id);
    let user_public = find_test_bounty(&user_body, public_bounty.id);
    assert_eq!(user_hidden["completed_by_user"], true);
    assert!(user_hidden["target_submissions"].is_null());
    assert_eq!(user_public["completed_by_user"], false);
    assert_eq!(user_public["target_submissions"], 5);

    let manager_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/aredl/bounty-board")
            .insert_header(("Authorization", format!("Bearer {}", manager_token)))
            .to_request(),
    )
    .await;
    assert!(manager_resp.status().is_success());
    let manager_body: serde_json::Value = read_body_json(manager_resp).await;
    let manager_hidden = find_test_bounty(&manager_body, hidden_bounty.id);
    assert_eq!(manager_hidden["target_submissions"], 3);
}

#[actix_web::test]
async fn bounty_completions_endpoint_and_delete_cascade() {
    let (app, db, _auth, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (user_id, username) = create_test_user(&db, None).await;
    let bounty = create_test_bounty(
        &db,
        level_id,
        Utc::now() - ChronoDuration::days(1),
        Some(Utc::now() + ChronoDuration::days(1)),
        None,
        true,
    )
    .await;

    let empty_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/aredl/bounty-board/{}/completions", bounty.id))
            .to_request(),
    )
    .await;
    assert!(empty_resp.status().is_success());
    let empty_body: serde_json::Value = read_body_json(empty_resp).await;
    assert_eq!(empty_body.as_array().unwrap().len(), 0);

    create_test_bounty_completion(&db, bounty.id, user_id).await;

    let completed_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/aredl/bounty-board/{}/completions", bounty.id))
            .to_request(),
    )
    .await;
    assert!(completed_resp.status().is_success());
    let completed_body: serde_json::Value = read_body_json(completed_resp).await;
    assert_eq!(completed_body.as_array().unwrap().len(), 1);
    assert_eq!(completed_body[0]["user"]["id"], user_id.to_string());
    assert_eq!(completed_body[0]["user"]["username"], username);
    assert!(completed_body[0]["completed_at"].is_string());

    diesel::delete(bounties::table.filter(bounties::id.eq(bounty.id)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to delete test bounty");
    let remaining = bounty_completed::table
        .filter(bounty_completed::bounty_id.eq(bounty.id))
        .count()
        .get_result::<i64>(&mut db.connection().unwrap())
        .expect("Failed to count cascaded bounty completions");
    assert_eq!(remaining, 0);
}

#[actix_web::test]
async fn bounty_completion_windows_and_overlaps() {
    let (_app, db, _auth, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    let achieved_at = "2026-01-10T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let record_id = create_test_record(&db, user_id, level_id).await;
    set_test_record_achieved_at(&db, record_id, achieved_at);

    let closed_before = create_test_bounty(
        &db,
        level_id,
        "2025-12-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        Some("2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()),
        None,
        true,
    )
    .await;
    let starts_after = create_test_bounty(
        &db,
        level_id,
        "2026-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        None,
        None,
        true,
    )
    .await;
    let active = create_test_bounty(
        &db,
        level_id,
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        Some("2026-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()),
        None,
        true,
    )
    .await;
    let open_ended = create_test_bounty(
        &db,
        level_id,
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        None,
        None,
        true,
    )
    .await;
    let overlapping = create_test_bounty(
        &db,
        level_id,
        "2026-01-05T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        Some("2026-01-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap()),
        None,
        true,
    )
    .await;

    fetch_test_record(&db, record_id)
        .complete_bounty_if_exists(&mut db.connection().unwrap())
        .expect("Failed to complete bounties");

    assert_eq!(count_test_bounty_completions(&db, active.id), 1);
    assert_eq!(count_test_bounty_completions(&db, open_ended.id), 1);
    assert_eq!(count_test_bounty_completions(&db, overlapping.id), 1);
    assert_eq!(count_test_bounty_completions(&db, closed_before.id), 0);
    assert_eq!(count_test_bounty_completions(&db, starts_after.id), 0);
}

#[actix_web::test]
async fn bounty_target_count_closes_and_stays_strict() {
    let (_app, db, _auth, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let (user_1, _) = create_test_user(&db, None).await;
    let (user_2, _) = create_test_user(&db, None).await;
    let (user_3, _) = create_test_user(&db, None).await;
    let achieved_at = "2026-01-10T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let original_end = "2026-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let bounty = create_test_bounty(
        &db,
        level_id,
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        Some(original_end),
        Some(2),
        true,
    )
    .await;

    let record_1 = create_test_record(&db, user_1, level_id).await;
    set_test_record_achieved_at(&db, record_1, achieved_at);
    let record_1 = fetch_test_record(&db, record_1);
    record_1
        .complete_bounty_if_exists(&mut db.connection().unwrap())
        .expect("Failed to complete first bounty");
    record_1
        .complete_bounty_if_exists(&mut db.connection().unwrap())
        .expect("Failed to reprocess first bounty");
    assert_eq!(count_test_bounty_completions(&db, bounty.id), 1);
    assert_eq!(
        fetch_test_bounty(&db, bounty.id).end_date,
        Some(original_end)
    );

    let before_close = Utc::now();
    let record_2 = create_test_record(&db, user_2, level_id).await;
    set_test_record_achieved_at(&db, record_2, achieved_at);
    fetch_test_record(&db, record_2)
        .complete_bounty_if_exists(&mut db.connection().unwrap())
        .expect("Failed to complete second bounty");
    let closed_bounty = fetch_test_bounty(&db, bounty.id);
    assert_eq!(count_test_bounty_completions(&db, bounty.id), 2);
    assert!(
        closed_bounty.end_date.unwrap() >= before_close,
        "auto-close should set end_date to current time"
    );
    assert_ne!(closed_bounty.end_date, Some(original_end));

    let record_3 = create_test_record(&db, user_3, level_id).await;
    set_test_record_achieved_at(&db, record_3, achieved_at);
    fetch_test_record(&db, record_3)
        .complete_bounty_if_exists(&mut db.connection().unwrap())
        .expect("Failed to process third bounty");
    assert_eq!(count_test_bounty_completions(&db, bounty.id), 2);
}

#[actix_web::test]
#[serial]
async fn bounty_completion_uses_fetched_video_timestamp() {
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
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let moderator_token = create_test_token(moderator_id, &auth.jwt_encoding_key).unwrap();
    let (inside_user, _) = create_test_user(&db, None).await;
    let (outside_user, _) = create_test_user(&db, None).await;
    let inside_token = create_test_token(inside_user, &auth.jwt_encoding_key).unwrap();
    let outside_token = create_test_token(outside_user, &auth.jwt_encoding_key).unwrap();
    let inside_level = create_test_level(&db).await;
    let outside_level = create_test_level(&db).await;
    let inside_bounty = create_test_bounty(
        &db,
        inside_level,
        "2009-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        Some("2010-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()),
        None,
        true,
    )
    .await;
    let outside_bounty = create_test_bounty(
        &db,
        outside_level,
        "2020-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        None,
        None,
        true,
    )
    .await;

    let create_submission = |level_id: Uuid, token: &str| {
        test::TestRequest::post()
            .uri("/aredl/submissions")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(json!({
                "level_id": level_id,
                "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
                "raw_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
                "mobile": false
            }))
            .to_request()
    };

    let inside_create =
        test::call_service(&app, create_submission(inside_level, &inside_token)).await;
    assert!(inside_create.status().is_success());
    let inside_submission: serde_json::Value = read_body_json(inside_create).await;
    let outside_create =
        test::call_service(&app, create_submission(outside_level, &outside_token)).await;
    assert!(outside_create.status().is_success());
    let outside_submission: serde_json::Value = read_body_json(outside_create).await;

    for submission in [&inside_submission, &outside_submission] {
        let submission_id = submission["id"].as_str().unwrap();
        let accept_resp = test::call_service(
            &app,
            test::TestRequest::patch()
                .uri(&format!("/aredl/submissions/{}", submission_id))
                .insert_header(("Authorization", format!("Bearer {}", moderator_token)))
                .set_json(json!({ "status": "Accepted", "reviewer_notes": "ok" }))
                .to_request(),
        )
        .await;
        assert!(
            accept_resp.status().is_success(),
            "accept status is {}",
            accept_resp.status()
        );
    }

    let expected_achieved_at = "2009-10-25T06:57:33Z".parse::<DateTime<Utc>>().unwrap();
    for _ in 0..40 {
        let updated = records::table
            .filter(records::achieved_at.eq(expected_achieved_at))
            .count()
            .get_result::<i64>(&mut db.connection().unwrap())
            .expect("Failed to count updated records");
        if updated >= 2 {
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }

    assert_eq!(yt_mock.calls_async().await, 2);
    assert_eq!(count_test_bounty_completions(&db, inside_bounty.id), 1);
    assert_eq!(count_test_bounty_completions(&db, outside_bounty.id), 0);

    std::env::remove_var("YOUTUBE_API_BASE_URL");
    clear_google_env();
}
