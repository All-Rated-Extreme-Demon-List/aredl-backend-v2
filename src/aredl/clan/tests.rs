#[cfg(test)]
use {
    crate::{
        aredl::{
            clan::test_utils::refresh_test_clan_created_levels,
            levels::test_utils::{
                add_test_level_creators, create_test_level_with_publisher,
                create_test_level_with_record,
            },
        },
        clans::test_utils::{create_test_clan, create_test_clan_member},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
};

#[actix_web::test]
async fn get_clan() {
    let (app, db, _, _) = init_test_app().await;

    let (user, _) = create_test_user(&db, None).await;
    let (_, record_id) = create_test_level_with_record(&db, user).await;

    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, user, 0).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/clan/{clan_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

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
async fn get_clan_includes_created_levels_and_matching_creators() {
    let (app, db, _, _) = init_test_app().await;

    let (creator_a, _) = create_test_user(&db, None).await;
    let (creator_b, _) = create_test_user(&db, None).await;
    let (outsider, _) = create_test_user(&db, None).await;

    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, creator_a, 0).await;
    create_test_clan_member(&db, clan_id, creator_b, 0).await;

    let published_level_id = create_test_level_with_publisher(&db, creator_a).await;
    let created_level_id = create_test_level_with_publisher(&db, outsider).await;
    add_test_level_creators(&db, created_level_id, &[creator_a, creator_b, outsider]).await;
    refresh_test_clan_created_levels(&db).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/clan/{clan_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let created = body["created"].as_array().unwrap();
    let published = body["published"].as_array().unwrap();

    let published_fallback = created
        .iter()
        .find(|entry| entry["id"].as_str() == Some(&published_level_id.to_string()))
        .expect("Published clan level without creators should appear in created");
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
        .expect("Explicitly created clan level should appear in created");
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
