#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::test_utils::create_test_level,
            statistics::submissions::ResolvedQueueLevelSubmissionsRow,
            submissions::test_utils::create_test_submission, submissions::SubmissionStatus,
        },
        auth::create_test_token,
        schema::aredl::submissions,
        test_utils::init_test_app,
        users::test_utils::create_test_user,
    },
    actix_web::{
        http::header,
        test::{self, TestRequest},
    },
    diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl},
};

#[actix_web::test]
async fn total_submissions_counts_ordering_and_percent_unique_pairs() {
    let (app, db, auth, _db) = init_test_app().await;

    let (auth_user, _) = create_test_user(&db, None).await;
    let token = create_test_token(auth_user, &auth.jwt_encoding_key).unwrap();

    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;
    let (u3, _) = create_test_user(&db, None).await;
    let (u4, _) = create_test_user(&db, None).await;

    let level1 = create_test_level(&db).await;
    let level2 = create_test_level(&db).await;

    create_test_submission(level1, u1, &db).await;
    create_test_submission(level1, u2, &db).await;
    create_test_submission(level1, u3, &db).await;
    create_test_submission(level2, u4, &db).await;

    sql_query("REFRESH MATERIALIZED VIEW aredl.submission_totals")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = TestRequest::get()
        .uri("/aredl/statistics/submissions")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedQueueLevelSubmissionsRow> = test::read_body_json(resp).await;

    assert_eq!(arr.len(), 3);

    assert!(arr[0].level.is_none());
    assert_eq!(arr[0].submissions, 4);
    assert!((arr[0].percent_of_queue - 100.0).abs() < 1e-6);

    assert_eq!(arr[1].level.as_ref().unwrap().id, level1);
    assert_eq!(arr[1].submissions, 3);
    assert!((arr[1].percent_of_queue - 75.0).abs() < 1e-6);

    assert_eq!(arr[2].level.as_ref().unwrap().id, level2);
    assert_eq!(arr[2].submissions, 1);
    assert!((arr[2].percent_of_queue - 25.0).abs() < 1e-6);
}

#[actix_web::test]
async fn total_submissions_ignores_non_pending_unique_pairs() {
    let (app, db, auth, _db) = init_test_app().await;

    let (auth_user, _) = create_test_user(&db, None).await;
    let token = create_test_token(auth_user, &auth.jwt_encoding_key).unwrap();

    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;

    let level1 = create_test_level(&db).await;

    create_test_submission(level1, u1, &db).await;
    let non_pending_id = create_test_submission(level1, u2, &db).await;

    diesel::update(submissions::table.filter(submissions::id.eq(non_pending_id)))
        .set(submissions::status.eq(SubmissionStatus::Denied))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    sql_query("REFRESH MATERIALIZED VIEW aredl.submission_totals")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = TestRequest::get()
        .uri("/aredl/statistics/submissions")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedQueueLevelSubmissionsRow> = test::read_body_json(resp).await;

    assert_eq!(arr.len(), 2);

    assert!(arr[0].level.is_none());
    assert_eq!(arr[0].submissions, 1);
    assert!((arr[0].percent_of_queue - 100.0).abs() < 1e-6);

    assert_eq!(arr[1].level.as_ref().unwrap().id, level1);
    assert_eq!(arr[1].submissions, 1);
    assert!((arr[1].percent_of_queue - 100.0).abs() < 1e-6);
}
