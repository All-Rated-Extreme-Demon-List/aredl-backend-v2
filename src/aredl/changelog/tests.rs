use crate::aredl::changelog::test_utils::insert_history_entry;
#[cfg(test)]
use crate::{aredl::levels::test_utils::create_test_level, schema::aredl::levels, test_utils::*};

#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::query_dsl::methods::FilterDsl;
#[cfg(test)]
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
#[actix_web::test]
async fn get_changelog() {
    let (app, _, _, _) = init_test_app().await;
    let req = test::TestRequest::get()
        .uri("/aredl/changelog")
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

    {
        let conn = &mut db.connection().unwrap();
        // raise and lower
        diesel::update(levels::table.filter(levels::id.eq(l1)))
            .set(levels::position.eq(1))
            .execute(conn)
            .unwrap();
        diesel::update(levels::table.filter(levels::id.eq(l1)))
            .set(levels::position.eq(3))
            .execute(conn)
            .unwrap();
        // swap
        diesel::update(levels::table.filter(levels::id.eq(l3)))
            .set(levels::position.eq(2))
            .execute(conn)
            .unwrap();
        // move to legacy and back
        diesel::update(levels::table.filter(levels::id.eq(l1)))
            .set(levels::legacy.eq(true))
            .execute(conn)
            .unwrap();
        diesel::update(levels::table.filter(levels::id.eq(l1)))
            .set(levels::legacy.eq(false))
            .execute(conn)
            .unwrap();
    }

    // removed
    insert_history_entry(&db, None, Some(3), None, l1, None, None);
    // unknown
    insert_history_entry(&db, Some(5), Some(5), None, l1, None, None);

    let req = test::TestRequest::get()
        .uri("/aredl/changelog?per_page=20")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
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
        .uri("/aredl/changelog?per_page=5&page=2")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["page"], 2);
}
