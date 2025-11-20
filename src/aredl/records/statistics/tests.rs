#[cfg(test)]
use crate::{
    aredl::{
        levels::test_utils::create_test_level_with_record,
        records::{statistics::ResolvedLevelTotalRecordsRow, test_utils::create_test_record},
    },
    auth::create_test_token,
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
use actix_web::http::header;
use actix_web::test::{self, TestRequest};
use diesel::{sql_query, RunQueryDsl};

#[actix_web::test]
async fn total_records_counts_and_ordering() {
    let (app, db, auth, _db) = init_test_app().await;
    let (user_1, _) = create_test_user(&db, None).await;
    let (user_2, _) = create_test_user(&db, None).await;
    let (user_3, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_1, &auth.jwt_encoding_key).unwrap();

    let (level1, _) = create_test_level_with_record(&db, user_1).await;
    let (level2, _) = create_test_level_with_record(&db, user_1).await;

    create_test_record(&db, user_2, level1).await;
    create_test_record(&db, user_3, level1).await;

    sql_query("REFRESH MATERIALIZED VIEW aredl.record_totals")
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = TestRequest::get()
        .uri("/aredl/records/statistics")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let arr: Vec<ResolvedLevelTotalRecordsRow> = test::read_body_json(resp).await;

    assert_eq!(arr.len(), 3);
    assert!(arr[0].level.is_none());
    assert_eq!(arr[0].records, 4);
    assert_eq!(arr[1].level.as_ref().unwrap().id, level1);
    assert_eq!(arr[1].records, 3);
    assert_eq!(arr[2].level.as_ref().unwrap().id, level2);
    assert_eq!(arr[2].records, 1);
}
