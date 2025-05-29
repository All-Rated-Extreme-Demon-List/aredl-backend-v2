#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    aredl::records::tests::create_test_record, 
    schema::{aredl::levels, users, clans, clan_members},
    db::DbConnection
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
async fn refresh_test_leaderboards(conn: &mut DbConnection) {
    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.user_leaderboard")
        .execute(conn)
        .expect("Failed to update leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.country_leaderboard")
        .execute(conn)
        .expect("Failed to update country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.clans_leaderboard")
        .execute(conn)
        .expect("Failed to update clans leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl.position_history_full_view")
        .execute(conn)
        .expect("Failed to update position history");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.user_leaderboard")
        .execute(conn)
        .expect("Failed to update platformer leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.country_leaderboard")
        .execute(conn)
        .expect("Failed to update platformer country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.clans_leaderboard")
        .execute(conn)
        .expect("Failed to update platformer clans leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW arepl.position_history_full_view")
        .execute(conn)
        .expect("Failed to update platformer position history");
}

#[actix_web::test]
async fn list_leaderboard() {
    let (app, mut conn, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let record = create_test_record(&mut conn, user).await;

    refresh_test_leaderboards(&mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/leaderboard/")
        .to_request();

    let level_score = i64::from(
        levels::table
            .filter(levels::id.eq(record.level_id))
            .select(levels::points)
            .get_result::<i32>(&mut conn)
            .expect("hi"),
    );

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let data = body["data"].as_array().unwrap();

    assert!(data.len() > 0, "No data was returned!");
    assert_eq!(
        data[0]["rank"].as_i64().unwrap(),
        1,
        "This user should be top 1!"
    );

    let user_entry = data
        .iter()
        .find(|entry| entry["user"]["id"].as_str().unwrap().to_string() == user.to_string())
        .expect("User not found in leaderboard!");

    assert_eq!(
        user_entry["hardest"]["id"].as_str().unwrap().to_string(),
        record.level_id.to_string(),
        "Hardest does not match this user's hardest (and only) record!"
    );
    assert_eq!(
        user_entry["extremes"].as_i64().unwrap(),
        1,
        "User should only have 1 record!"
    );
    assert_eq!(
        level_score,
        user_entry["total_points"].as_i64().unwrap(),
        "User's score does not match!"
    )
}

#[actix_web::test]
async fn get_country_lb() {

    let (app, mut conn, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    create_test_record(&mut conn, user).await;

    let us_id = 840;

    diesel::update(users::table)
        .filter(users::id.eq(user))
        .set(users::country.eq(us_id)) // united states
        .execute(&mut conn)
        .expect("Failed to assign country to user!");

    refresh_test_leaderboards(&mut conn).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/leaderboard/countries").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let arr = body["data"].as_array().unwrap();

    assert_ne!(arr.len(), 0, "No countries were returned!");

    assert!(
        arr.iter().any(
            |x| x["country"] == i64::from(us_id)
        ),
        "Country codes do not match!"
    );
}

#[actix_web::test]
async fn get_clans_lb() {
    let (app, mut conn, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    create_test_record(&mut conn, user).await;

    let clan_id = diesel::insert_into(clans::table)
        .values((
            clans::global_name.eq("Test Clan"),
            clans::tag.eq("TS")
        ))
        .returning(clans::id)
        .get_result::<Uuid>(&mut conn)
        .expect("Failed to create clan");

    diesel::insert_into(clan_members::table)
        .values((
            clan_members::clan_id.eq(clan_id),
            clan_members::user_id.eq(user)
        ))
        .execute(&mut conn)
        .expect("Failed to add user to clan");

    refresh_test_leaderboards(&mut conn).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/leaderboard/clans").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let arr = body["data"].as_array().unwrap();

    assert_ne!(arr.len(), 0, "No countries were returned!");

    assert!(
        arr.iter().any(
            |x| x["clan"]["id"] == clan_id.to_string()
        ),
        "Country codes do not match!"
    );

}
