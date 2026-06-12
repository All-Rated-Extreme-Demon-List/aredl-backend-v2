#[cfg(test)]
use {
    crate::{
        arepl::{
            country::test_utils::refresh_test_country_created_levels,
            levels::test_utils::{
                add_test_level_creators, create_test_level, create_test_level_with_publisher,
                create_test_level_with_record, set_test_level_status,
            },
            levels::LevelStatus,
            records::test_utils::{
                create_test_record, set_test_record_achieved_at, set_test_record_verification,
            },
        },
        test_utils::*,
        users::test_utils::{create_test_user, set_test_user_ban_level, set_test_user_country},
    },
    actix_web::test::{self, read_body_json},
    chrono::{DateTime, Utc},
    uuid::Uuid,
};

#[actix_web::test]
async fn get_country() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let (_, record_id) = create_test_level_with_record(&db, user).await;

    let us_id = 840;

    set_test_user_country(&db, user, Some(us_id)).await;

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

#[actix_web::test]
async fn get_country_includes_created_levels_and_matching_creators() {
    let (app, db, _, _) = init_test_app().await;
    let country_id = 840;

    let (creator_a, _) = create_test_user(&db, None).await;
    let (creator_b, _) = create_test_user(&db, None).await;
    let (outsider, _) = create_test_user(&db, None).await;

    set_test_user_country(&db, creator_a, Some(country_id)).await;
    set_test_user_country(&db, creator_b, Some(country_id)).await;

    let published_level_id = create_test_level_with_publisher(&db, creator_a).await;
    let created_level_id = create_test_level_with_publisher(&db, outsider).await;
    add_test_level_creators(&db, created_level_id, &[creator_a, creator_b, outsider]).await;
    refresh_test_country_created_levels(&db).await;

    let req = test::TestRequest::get()
        .uri(format!("/arepl/country/{country_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let created = body["created"].as_array().unwrap();
    let published = body["published"].as_array().unwrap();

    let published_fallback = created
        .iter()
        .find(|entry| entry["id"].as_str() == Some(&published_level_id.to_string()))
        .expect("Published level without creators should appear in created");
    let fallback_creators = published_fallback["creators"].as_array().unwrap();
    let creator_a_id = creator_a.to_string();
    assert_eq!(fallback_creators.len(), 1);
    assert_eq!(
        fallback_creators[0]["id"].as_str(),
        Some(creator_a_id.as_str())
    );

    let shared_level = created
        .iter()
        .find(|entry| entry["id"].as_str() == Some(&created_level_id.to_string()))
        .expect("Explicitly created level should appear in created");
    let shared_creator_ids = shared_level["creators"]
        .as_array()
        .unwrap()
        .iter()
        .map(|creator| creator["id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let mut expected_creator_ids = vec![creator_a.to_string(), creator_b.to_string()];
    let mut actual_creator_ids = shared_creator_ids;
    expected_creator_ids.sort();
    actual_creator_ids.sort();
    assert_eq!(actual_creator_ids, expected_creator_ids);

    assert!(published
        .iter()
        .any(|entry| entry["id"].as_str() == Some(&published_level_id.to_string())));
    assert!(!published
        .iter()
        .any(|entry| entry["id"].as_str() == Some(&created_level_id.to_string())));
}

#[actix_web::test]
async fn get_country_includes_completion_counts_and_member_points() {
    let (app, db, _, _) = init_test_app().await;
    let country_id = 840;
    let other_country = 124;

    let (member_a, _) = create_test_user(&db, None).await;
    let (member_b, _) = create_test_user(&db, None).await;
    let (banned_member, _) = create_test_user(&db, None).await;
    let (outsider, _) = create_test_user(&db, None).await;

    set_test_user_country(&db, member_a, Some(country_id)).await;
    set_test_user_country(&db, member_b, Some(country_id)).await;
    set_test_user_country(&db, banned_member, Some(country_id)).await;
    set_test_user_country(&db, outsider, Some(other_country)).await;

    let shared_level = create_test_level(&db).await;
    let member_a_shared = create_test_record(&db, member_a, shared_level).await;
    let member_b_shared = create_test_record(&db, member_b, shared_level).await;
    let _banned_shared = create_test_record(&db, banned_member, shared_level).await;
    let _outsider_shared = create_test_record(&db, outsider, shared_level).await;

    let solo_level = create_test_level(&db).await;
    create_test_record(&db, member_a, solo_level).await;

    let legacy_level = create_test_level(&db).await;
    create_test_record(&db, member_b, legacy_level).await;

    let removed_level = create_test_level(&db).await;
    create_test_record(&db, member_a, removed_level).await;

    set_test_record_verification(&db, member_b_shared, true).await;
    set_test_user_ban_level(&db, banned_member, 1).await;
    set_test_level_status(&db, legacy_level, LevelStatus::Legacy, Some(4)).await;
    set_test_level_status(&db, removed_level, LevelStatus::Removed, None).await;

    let req = test::TestRequest::get()
        .uri(format!("/arepl/country/{country_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let shared_record = find_profile_record_for_level(&body, shared_level);
    assert_eq!(shared_record["completion_count"].as_i64(), Some(2));
    assert_eq!(
        find_profile_record_for_level(&body, legacy_level)["completion_count"].as_i64(),
        Some(1)
    );
    assert!(!body["records"]
        .as_array()
        .unwrap()
        .iter()
        .any(|record| record["level"]["id"].as_str() == Some(&removed_level.to_string())));

    let shared_points = shared_record["level"]["points"].as_f64().unwrap();
    let solo_points = find_profile_record_for_level(&body, solo_level)["level"]["points"]
        .as_f64()
        .unwrap();

    let member_a_points = find_member_points(&body, member_a);
    assert_eq!(member_a_points["completed_levels"].as_i64(), Some(2));
    assert_float_eq(
        member_a_points["contributed_points"].as_f64().unwrap(),
        shared_points / 2.0 + solo_points,
    );

    let member_b_points = find_member_points(&body, member_b);
    assert_eq!(member_b_points["completed_levels"].as_i64(), Some(2));
    assert_float_eq(
        member_b_points["contributed_points"].as_f64().unwrap(),
        shared_points / 2.0,
    );

    assert!(!body["members_points"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["member"]["id"].as_str() == Some(&banned_member.to_string())));

    let old_time: DateTime<Utc> = "2020-01-01T00:00:00Z".parse().unwrap();
    let new_time: DateTime<Utc> = "2021-01-01T00:00:00Z".parse().unwrap();
    set_test_record_achieved_at(&db, member_a_shared, old_time).await;
    set_test_record_achieved_at(&db, member_b_shared, new_time).await;

    let req = test::TestRequest::get()
        .uri(format!("/arepl/country/{country_id}/levels/{shared_level}/records").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let records: serde_json::Value = read_body_json(resp).await;
    let records = records.as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0]["id"].as_str(),
        Some(member_a_shared.to_string().as_str())
    );
    assert_eq!(
        records[1]["id"].as_str(),
        Some(member_b_shared.to_string().as_str())
    );
}

fn find_profile_record_for_level(body: &serde_json::Value, level_id: Uuid) -> &serde_json::Value {
    body["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["level"]["id"].as_str() == Some(&level_id.to_string()))
        .expect("profile record not found")
}

fn find_member_points(body: &serde_json::Value, user_id: Uuid) -> &serde_json::Value {
    body["members_points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["member"]["id"].as_str() == Some(&user_id.to_string()))
        .expect("member points entry not found")
}

fn assert_float_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000001,
        "expected {expected}, got {actual}"
    );
}
