#[cfg(test)]
use {
    crate::{
        arepl::leaderboard::test_utils::refresh_test_leaderboards,
        arepl::{
            levels::test_utils::{
                create_test_level, create_test_level_with_record, get_test_level,
            },
            records::test_utils::create_test_record,
        },
        clans::test_utils::{create_named_test_clan, create_test_clan, create_test_clan_member},
        test_utils::*,
        users::test_utils::{create_test_user, set_test_user_country},
    },
    actix_web::test::{self, read_body_json},
};

#[actix_web::test]
async fn list_leaderboard() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let (level_id, _) = create_test_level_with_record(&db, user).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/")
        .to_request();

    let level_score = i64::from(get_test_level(&db, level_id).await.points);

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let data = body["data"].as_array().unwrap();

    assert!(!data.is_empty(), "No data was returned!");
    assert_eq!(
        data[0]["rank"].as_i64().unwrap(),
        1,
        "This user should be top 1!"
    );

    let user_entry = data
        .iter()
        .find(|entry| entry["user"]["id"].as_str().unwrap() == user.to_string())
        .expect("User not found in leaderboard!");

    assert_eq!(
        user_entry["hardest"]["id"].as_str().unwrap().to_string(),
        level_id.to_string(),
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
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    create_test_level_with_record(&db, user).await;

    let us_id = 840;

    set_test_user_country(&db, user, Some(us_id)).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/countries")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let arr = body["data"].as_array().unwrap();

    assert_ne!(arr.len(), 0, "No countries were returned!");

    assert!(
        arr.iter().any(|x| x["country"] == i64::from(us_id)),
        "Country codes do not match!"
    );
}

#[actix_web::test]
async fn get_clans_lb() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    create_test_level_with_record(&db, user).await;

    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, user, 0).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/clans")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let arr = body["data"].as_array().unwrap();

    assert_ne!(arr.len(), 0, "No countries were returned!");

    assert!(
        arr.iter().any(|x| x["clan"]["id"] == clan_id.to_string()),
        "Country codes do not match!"
    );
}

#[actix_web::test]
async fn leaderboard_filters() {
    let (app, db, _, _) = init_test_app().await;
    let (u1, name1) = create_test_user(&db, None).await;
    let (u2, _name2) = create_test_user(&db, None).await;
    create_test_level_with_record(&db, u1).await;
    create_test_level_with_record(&db, u2).await;
    create_test_level_with_record(&db, u2).await;

    let us_id = 840;
    set_test_user_country(&db, u1, Some(us_id)).await;

    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, u1, 0).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/leaderboard/?name_filter={name1}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/leaderboard/?country_filter={us_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["user"]["id"] == u1.to_string()));

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/leaderboard/?clan_filter={clan_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["user"]["id"] == u1.to_string()));

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/?order=ExtremeCount")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"][0]["user"]["id"], u2.to_string());
}

#[actix_web::test]
async fn country_clan_leaderboard_orders() {
    let (app, db, _, _) = init_test_app().await;
    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;
    let _level1 = create_test_level_with_record(&db, u1).await;
    create_test_level_with_record(&db, u2).await;
    create_test_level_with_record(&db, u2).await;

    set_test_user_country(&db, u1, Some(840)).await;
    set_test_user_country(&db, u2, Some(124)).await;

    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, u1, 0).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/countries?order=ExtremeCount")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"][0]["country"], 124);

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/clans?order=ExtremeCount")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"][0]["clan"]["id"], clan_id.to_string());
}

#[actix_web::test]
async fn get_clans_leaderboard_with_filters() {
    let (app, db, _, _) = init_test_app().await;

    let level_id = create_test_level(&db).await;

    let mkl = create_named_test_clan(
        &db,
        "Mika Lore",
        "MKL",
        Some("This should be searchable via \"MKL\""),
    )
    .await;

    let user1 = create_test_user(&db, None).await.0;
    create_test_clan_member(&db, mkl.id, user1, 0).await;
    create_test_record(&db, user1, level_id).await;

    let clan2 = create_named_test_clan(&db, "Test clan", "TTC", None).await;

    let user2 = create_test_user(&db, None).await.0;
    create_test_clan_member(&db, clan2.id, user2, 0).await;
    create_test_record(&db, user2, level_id).await;

    refresh_test_leaderboards(&db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/clans?name_filter=%Test%")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        uuid::Uuid::parse_str(body["data"][0]["clan"]["id"].as_str().unwrap()).unwrap(),
        clan2.id
    );

    let req = test::TestRequest::get()
        .uri("/arepl/leaderboard/clans?name_filter=%MKL%")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        uuid::Uuid::parse_str(body["data"][0]["clan"]["id"].as_str().unwrap()).unwrap(),
        mkl.id
    );
}
