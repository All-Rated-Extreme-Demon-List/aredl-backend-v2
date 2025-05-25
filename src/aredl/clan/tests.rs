#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    schema::{clans, clan_members},
    aredl::records::tests::create_test_record
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{RunQueryDsl,ExpressionMethods};
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn get_clan() {
    let (app, mut conn, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let record = create_test_record(&mut conn, user).await;

    let clan_id = diesel::insert_into(clans::table)
        .values((
            clans::global_name.eq("Test Clan"),
            clans::tag.eq("PMO")
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

    let req = test::TestRequest::get()
        .uri(format!("/aredl/clan/{clan_id}").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    let has_record = 
        body["records"].as_array().unwrap()
            .iter()
            .any(|record_iter| 
                record_iter["id"].as_str().unwrap().to_string() == record.id.to_string()
            );
        
    assert!(has_record);
}
