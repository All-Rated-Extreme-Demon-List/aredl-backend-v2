#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    schema::{users, aredl::levels},
    aredl::records::tests::create_test_record
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{RunQueryDsl, ExpressionMethods, QueryDsl};

#[actix_web::test]
async fn list_leaderboard() {
    let (app, mut conn, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let record = create_test_record(&mut conn, user).await;

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.user_leaderboard")
        .execute(&mut conn)
        .expect("Failed to refresh leaderboard!");

    let req = test::TestRequest::get()
        .uri("/aredl/leaderboard/")
        .to_request();

    let level_score = i64::from(
        levels::table
            .filter(levels::id.eq(record.level_id))
            .select(levels::points)
            .get_result::<i32>(&mut conn)
            .expect("hi")
    );

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let data = body["data"].as_array().unwrap();
    
    assert!(data.len() > 0, "No data was returned!");
    assert_eq!(data[0]["rank"].as_i64().unwrap(), 1, "This user should be top 1!");

    let user_entry = data.iter().find(
        |entry| entry["user"]["id"].as_str().unwrap().to_string() == user.to_string()
    ).expect("User not found in leaderboard!");

    assert_eq!(user_entry["hardest"]["id"].as_str().unwrap().to_string(), record.level_id.to_string(), "Hardest does not match this user's hardest (and only) record!");
    assert_eq!(user_entry["extremes"].as_i64().unwrap(), 1, "User should only have 1 record!");
    assert_eq!(level_score, user_entry["total_points"].as_i64().unwrap(), "User's score does not match!")
}
