use crate::arepl::levels::test_utils::create_test_level;
use crate::arepl::statistics::submissions::daily::ResolvedLeaderboardRow;
use crate::arepl::submissions::test_utils::{create_test_submission, insert_history_entry};
use crate::arepl::submissions::SubmissionStatus;
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
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let level_id = create_test_level(&db).await;
    let mod_id = mod1;

    let sub = create_test_submission(level_id, Uuid::new_v4(), &db).await;
    insert_history_entry(sub, Some(mod_id), SubmissionStatus::Accepted, &db).await;
    insert_history_entry(sub, Some(mod_id), SubmissionStatus::Denied, &db).await;
    sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let uri = format!(
        "/arepl/statistics/submissions/daily?reviewer_id={}&page=1&per_page=10",
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
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let (mod2, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&db).await;

    let sub1 = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::arepl::submissions::SubmissionStatus::Accepted,
        &db,
    )
    .await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::arepl::submissions::SubmissionStatus::Accepted,
        &db,
    )
    .await;
    insert_history_entry(
        sub1,
        Some(mod1),
        crate::arepl::submissions::SubmissionStatus::Denied,
        &db,
    )
    .await;

    let sub2 = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(
        sub2,
        Some(mod2),
        crate::arepl::submissions::SubmissionStatus::Accepted,
        &db,
    )
    .await;
    insert_history_entry(
        sub2,
        Some(mod2),
        crate::arepl::submissions::SubmissionStatus::UnderConsideration,
        &db,
    )
    .await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = TestRequest::get()
        .uri("/arepl/statistics/submissions/daily/leaderboard")
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
    let (app, db, auth, _db) = init_test_app().await;
    let (mod_active, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let (mod_inactive, _) = create_test_user(&db, None).await;
    let token = create_test_token(mod_active, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&db).await;

    let s1 = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(s1, Some(mod_active), SubmissionStatus::Accepted, &db).await;
    let s2 = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(s2, Some(mod_inactive), SubmissionStatus::Denied, &db).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let uri = "/arepl/statistics/submissions/daily/leaderboard?only_active=true";
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
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod1, &auth.jwt_encoding_key).unwrap();

    let lvl = create_test_level(&db).await;
    let sub = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(sub, Some(mod1), SubmissionStatus::Accepted, &db).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let tomorrow = chrono::Utc::now().date_naive() + chrono::Duration::days(1);
    let uri = format!(
        "/arepl/statistics/submissions/daily/leaderboard?since={}",
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
