#[cfg(test)]
use {
    crate::{
        arepl::{
            levels::test_utils::create_test_level,
            statistics::submissions::daily::ResolvedLeaderboardRow,
            submissions::{
                test_utils::{create_test_submission, insert_history_entry},
                SubmissionStatus,
            },
        },
        auth::permission::get_permission_privilege_level,
        auth::{create_test_token, Permission},
        roles::test_utils::{add_user_to_role, create_test_role},
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::{http::header, test::{self, read_body_json}},
    diesel::{sql_query, RunQueryDsl},
    serde_json::Value,
    uuid::Uuid,
};

#[actix_web::test]
async fn submission_stats_filter_moderator() {
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
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
        let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Status: {}", resp.status());

    let body: Value = read_body_json(resp).await;
    let entries = body["data"].as_array().expect("`data` should be array");
    assert_eq!(entries.len(), 1, "Entries array length should be 1");
    let entry = &entries[0];
    assert_eq!(entry["accepted"].as_i64().unwrap(), 1);
    assert_eq!(entry["denied"].as_i64().unwrap(), 1);
}

#[actix_web::test]
async fn submission_stats_hides_base_reviewer_filter_for_non_auditor() {
    let (app, db, auth, _db) = init_test_app().await;

    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (requester_non_auditor, _) =
        create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (requester_auditor, _) =
        create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;

    let reviewers_audit_level =
        get_permission_privilege_level(&mut db.connection().unwrap(), Permission::ReviewersAudit)
            .unwrap();
    let reviewers_audit_role = create_test_role(&db, reviewers_audit_level).await;
    add_user_to_role(&db, reviewers_audit_role, requester_auditor).await;

    let non_auditor_token = create_test_token(requester_non_auditor, &auth.jwt_encoding_key)
        .expect("Failed to generate token");
    let auditor_token = create_test_token(requester_auditor, &auth.jwt_encoding_key)
        .expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let sub = create_test_submission(level_id, Uuid::new_v4(), &db).await;
    insert_history_entry(sub, Some(base_reviewer), SubmissionStatus::Accepted, &db).await;

    sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let uri = format!(
        "/arepl/statistics/submissions/daily?reviewer_id={}&page=1&per_page=10",
        base_reviewer
    );

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", non_auditor_token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Status: {}", resp.status());
    let body: Value = read_body_json(resp).await;
    let entries = body["data"].as_array().expect("`data` should be array");
    assert_eq!(entries.len(), 0);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", auditor_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Status: {}", resp.status());
    let body: Value = read_body_json(resp).await;
    let entries = body["data"].as_array().expect("`data` should be array");
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]["moderator"]["id"].as_str().unwrap(),
        base_reviewer.to_string()
    );
}

#[actix_web::test]
async fn submission_leaderboard_include_base_reviewers_requires_audit() {
    let (app, db, auth, _db) = init_test_app().await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (requester_non_auditor, _) =
        create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (requester_auditor, _) =
        create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;

    let reviewers_audit_level =
        get_permission_privilege_level(&mut db.connection().unwrap(), Permission::ReviewersAudit)
            .unwrap();
    let reviewers_audit_role = create_test_role(&db, reviewers_audit_level).await;
    add_user_to_role(&db, reviewers_audit_role, requester_auditor).await;

    let non_auditor_token = create_test_token(requester_non_auditor, &auth.jwt_encoding_key)
        .expect("Failed to generate token");
    let auditor_token = create_test_token(requester_auditor, &auth.jwt_encoding_key)
        .expect("Failed to generate token");

    let lvl = create_test_level(&db).await;

    let base_sub = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(
        base_sub,
        Some(base_reviewer),
        SubmissionStatus::Accepted,
        &db,
    )
    .await;

    let full_sub = create_test_submission(lvl, Uuid::new_v4(), &db).await;
    insert_history_entry(full_sub, Some(full_reviewer), SubmissionStatus::Denied, &db).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.submission_stats")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let uri = "/arepl/statistics/submissions/daily/leaderboard?include_base_reviewers=true";

    let req = test::TestRequest::get()
        .uri(uri)
        .insert_header((
            header::AUTHORIZATION,
            format!("Bearer {}", non_auditor_token),
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = read_body_json(resp).await;
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].moderator.id, full_reviewer);

    let req = test::TestRequest::get()
        .uri(uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", auditor_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = read_body_json(resp).await;
    assert_eq!(arr.len(), 2);
    assert!(arr.iter().any(|row| row.moderator.id == base_reviewer));
    assert!(arr.iter().any(|row| row.moderator.id == full_reviewer));
}

#[actix_web::test]
async fn submission_stats_endpoints_require_full_review_permission() {
    let (app, db, auth, _db) = init_test_app().await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let token = create_test_token(base_reviewer, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::get()
        .uri("/arepl/statistics/submissions/daily?page=1&per_page=10")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (submission_review_full) to access this endpoint"),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/arepl/statistics/submissions/daily/leaderboard")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (submission_review_full) to access this endpoint"),
    )
    .await;
}

#[actix_web::test]
async fn submission_leaderboard_counts_and_ordering() {
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (mod2, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
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

        let req = test::TestRequest::get()
        .uri("/arepl/statistics/submissions/daily/leaderboard")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = read_body_json(resp).await;

    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].moderator.id, mod1);
    assert_eq!(arr[0].total, 3);
    assert_eq!(arr[1].moderator.id, mod2);
    assert_eq!(arr[1].total, 2);
}

#[actix_web::test]
async fn submission_leaderboard_only_active_filters_out() {
    let (app, db, auth, _db) = init_test_app().await;
    let (mod_active, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
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
        let req = test::TestRequest::get()
        .uri(uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLeaderboardRow> = read_body_json(resp).await;

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].moderator.id, mod_active);
    assert_eq!(arr[0].total, 1);
}

#[actix_web::test]
async fn submission_leaderboard_since_filters_out_future_date() {
    let (app, db, auth, _db) = init_test_app().await;
    let (mod1, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
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

        let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let arr: Vec<ResolvedLeaderboardRow> = read_body_json(resp).await;
    assert_eq!(
        arr.len(),
        0,
        "No mods should show up for a future 'since' date"
    );
}
