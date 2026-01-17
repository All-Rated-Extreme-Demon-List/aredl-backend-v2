#[cfg(test)]
use {
    crate::{
        aredl::levels::test_utils::create_test_level_with_record,
        schema::{clan_members, clans},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, RunQueryDsl},
    uuid::Uuid,
};

#[actix_web::test]
async fn get_clan() {
    let (app, db, _, _) = init_test_app().await;

    let (user, _) = create_test_user(&db, None).await;
    let (_, record_id) = create_test_level_with_record(&db, user).await;

    let clan_id = diesel::insert_into(clans::table)
        .values((clans::global_name.eq("Test Clan"), clans::tag.eq("PMO")))
        .returning(clans::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create clan");

    diesel::insert_into(clan_members::table)
        .values((
            clan_members::clan_id.eq(clan_id),
            clan_members::user_id.eq(user),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to add user to clan");

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
