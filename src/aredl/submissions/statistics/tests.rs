use crate::aredl::levels::test_utils::create_test_level;
use crate::aredl::submissions::statistics::ResolvedLeaderboardRow;
use crate::aredl::submissions::test_utils::{create_test_submission, insert_history_entry};
use crate::aredl::submissions::SubmissionStatus;
use crate::auth::{create_test_token, Permission};
use crate::test_utils::init_test_app;
use crate::users::test_utils::create_test_user;
use actix_web::http::header;
use actix_web::test::{self, TestRequest};
use diesel::{sql_query, RunQueryDsl};
use serde_json::Value;
use uuid::Uuid;

#[actix_web::test]
async fn submission_stats_filter_moderator() {
    let (app, mut conn, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let level_id = create_test_level(&mut conn).await;
    let mod_id = mod1;

    let sub = create_test_submission(level_id, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(sub, Some(mod_id), SubmissionStatus::Accepted, &mut conn).await;
    insert_history_entry(sub, Some(mod_id), SubmissionStatus::Denied, &mut conn).await;
    sql_query("REFRESH MATERIALIZED VIEW aredl.submission_stats")
        .execute(&mut conn)
        .unwrap();

    let uri = format!(
        "/aredl/submissions/statistics?moderator_id={}&page=1&per_page=10",
        mod_id
    );
    let req = TestRequest::get()
        .uri(&uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Status: {}", resp.status());

    let body: Value = test::read_body_json(resp).await;
    let entries = body["data"].as_array().expect("`data` should be array");
    assert_eq!(entries.len(), 1, "Entries array length should be 1");
    let entry = &entries[0];
    assert_eq!(entry["accepted"].as_i64().unwrap(), 1);
    assert_eq!(entry["denied"].as_i64().unwrap(), 1);
}

#[actix_web::test]
async fn submission_leaderboard_counts_and_ordering() {
    let (app, mut conn, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let (mod2, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&mut conn).await;

    let sub1 = create_test_submission(lvl, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::aredl::submissions::SubmissionStatus::Accepted,
        &mut conn,
    )
    .await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::aredl::submissions::SubmissionStatus::Accepted,
        &mut conn,
    )
    .await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::aredl::submissions::SubmissionStatus::Denied,
        &mut conn,
    )
    .await;

    let sub2 = create_test_submission(lvl, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(
        sub2,
        Some(mod2),
        crate::aredl::submissions::SubmissionStatus::Accepted,
        &mut conn,
    )
    .await;
    insert_history_entry(
        sub2,
        Some(mod2),
        crate::aredl::submissions::SubmissionStatus::UnderConsideration,
        &mut conn,
    )
    .await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.submission_stats")
        .execute(&mut conn)
        .unwrap();

    let req = TestRequest::get()
        .uri("/aredl/submissions/statistics/leaderboard")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = test::read_body_json(resp).await;

    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].moderator.id, mod1);
    assert_eq!(arr[0].total, 3);
    assert_eq!(arr[1].moderator.id, mod2);
    assert_eq!(arr[1].total, 2);
}

#[actix_web::test]
async fn submission_leaderboard_only_active_filters_out() {
    let (app, mut conn, auth, _db) = init_test_app().await;
    let (mod_active, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let (mod_inactive, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(mod_active, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&mut conn).await;

    let s1 = create_test_submission(lvl, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(s1, Some(mod_active), SubmissionStatus::Accepted, &mut conn).await;
    let s2 = create_test_submission(lvl, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(s2, Some(mod_inactive), SubmissionStatus::Denied, &mut conn).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.submission_stats")
        .execute(&mut conn)
        .unwrap();

    let uri = "/aredl/submissions/statistics/leaderboard?only_active=true";
    let req = TestRequest::get()
        .uri(uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = test::read_body_json(resp).await;

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].moderator.id, mod_active);
    assert_eq!(arr[0].total, 1);
}

#[actix_web::test]
async fn submission_leaderboard_since_filters_out_future_date() {
    let (app, mut conn, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&mut conn).await;
    let sub = create_test_submission(lvl, Uuid::new_v4(), &mut conn).await;
    insert_history_entry(sub, Some(mod1), SubmissionStatus::Accepted, &mut conn).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.submission_stats")
        .execute(&mut conn)
        .unwrap();

    let tomorrow = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
    let uri = format!(
        "/aredl/submissions/statistics/leaderboard?since={}",
        tomorrow
    );

    let req = TestRequest::get()
        .uri(&uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let arr: Vec<ResolvedLeaderboardRow> = test::read_body_json(resp).await;
    assert_eq!(
        arr.len(),
        0,
        "No mods should show up for a future 'since' date"
    );
}
