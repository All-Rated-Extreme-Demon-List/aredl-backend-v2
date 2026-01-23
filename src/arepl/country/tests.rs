#[cfg(test)]
use {
    crate::{
        arepl::levels::test_utils::create_test_level_with_record,
        schema::users,
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, RunQueryDsl},
};

#[actix_web::test]
async fn get_country() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let (_, record_id) = create_test_level_with_record(&db, user).await;

    let us_id = 840;

    diesel::update(users::table)
        .filter(users::id.eq(user))
        .set(users::country.eq(us_id)) // united states
        .execute(&mut db.connection().unwrap())
        .expect("Failed to assign country to user!");

    let req = test::TestRequest::get()
        .uri(format!("/arepl/country/{us_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body["country"].as_i64().unwrap(),
        i64::from(us_id),
        "Country codes do not match!"
    );

    let has_record = body["records"]
        .as_array()
        .unwrap()
        .iter()
        .any(|record_iter| {
            record_iter["id"].as_str().unwrap().to_string() == record_id.to_string()
        });

    assert!(has_record);
}
