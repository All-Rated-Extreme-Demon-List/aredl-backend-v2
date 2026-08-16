#[cfg(test)]
use {
    crate::{
        arepl::changelog::test_utils::insert_history_entry,
        arepl::levels::{
            test_utils::{create_test_level, set_test_level_position, set_test_level_status},
            LevelStatus,
        },
        test_utils::*,
    },
    actix_web::test::{self, read_body_json},
};
#[actix_web::test]
async fn get_changelog() {
    let (app, _, _, _) = init_test_app().await;
    let req = test::TestRequest::get()
        .uri("/arepl/changelog")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn changelog_actions_and_pagination() {
    let (app, db, _, _) = init_test_app().await;
    let l1 = create_test_level(&db).await;
    create_test_level(&db).await;
    let l3 = create_test_level(&db).await;

    // raise and lower
    set_test_level_position(&db, l1, Some(1)).await;
    set_test_level_position(&db, l1, Some(3)).await;
    // swap
    set_test_level_position(&db, l3, Some(2)).await;
    // move to legacy and back
    set_test_level_status(&db, l1, LevelStatus::Legacy, Some(3)).await;
    set_test_level_status(&db, l1, LevelStatus::MainList, Some(3)).await;

    // removed
    insert_history_entry(
        &db,
        None,
        Some(3),
        Some(LevelStatus::MainList),
        LevelStatus::Removed,
        l1,
        None,
        None,
    );
    // unknown
    insert_history_entry(
        &db,
        Some(5),
        Some(5),
        Some(LevelStatus::MainList),
        LevelStatus::MainList,
        l1,
        None,
        None,
    );

    let req = test::TestRequest::get()
        .uri("/arepl/changelog?per_page=20")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 9);
    let mut kinds = Vec::new();
    for entry in data {
        if let Some(k) = entry["action"]
            .as_object()
            .and_then(|o| o.keys().next())
            .cloned()
        {
            kinds.push(k);
        }
    }
    for expected in [
        "Placed",
        "Raised",
        "Lowered",
        "Swapped",
        "MovedToLegacy",
        "MovedFromLegacy",
        "Removed",
        "Unknown",
    ] {
        assert!(kinds.iter().any(|k| k == expected));
    }

    // pagination
    let req = test::TestRequest::get()
        .uri("/arepl/changelog?per_page=5&page=2")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["page"], 2);
}
