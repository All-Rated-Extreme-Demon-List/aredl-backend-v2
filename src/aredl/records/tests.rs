#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    db::DbConnection,
    aredl::records::Record,
    schema::aredl::records
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{RunQueryDsl, ExpressionMethods, SelectableHelper};
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;

async fn create_test_record(conn: &mut DbConnection, submitter: Uuid) -> Record {
    let level = create_test_level(conn).await;
    let record = diesel::insert_into(records::table)
        .values((
            records::level_id.eq(level),
            records::submitted_by.eq(submitter),
            records::mobile.eq(false),
            records::video_url.eq("https://video.com")
        ))
        .returning(Record::as_select())
        .get_result::<Record>(conn)
        .expect("Failed to create record!");

    record

}

#[actix_web::test]
async fn create_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level = create_test_level(&mut conn).await;
    
    let record_data = json!({
        "submitted_by": user_id.to_string(),
        "mobile": false,
        "level_id": level.to_string(),
        "video_url": "https://video.com",
        "is_verification": false,
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&record_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    
    assert_eq!(body["submitted_by"].as_str().unwrap(), user_id.to_string().as_str(), "Names do not match!")
}

#[actix_web::test]
async fn get_record_list() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let req = test::TestRequest::get()
        .uri("/aredl/records")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn update_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let record = create_test_record(&mut conn, user_id).await;
    let update_data = json!({
        "video_url": "https://updated.com"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/records/{}", record.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn get_own_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_record(&mut conn, user_id).await;
    let req = test::TestRequest::get()
        .uri("/aredl/records/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("{:?}", body);
    assert_ne!(body["data"].as_array().unwrap().len(), 0, "Did not return any data!");
    // lmao
    assert_eq!(body["data"].as_array().unwrap()[0].as_object().unwrap()["submitted_by"].as_object().unwrap()["id"].as_str().unwrap(), user_id.to_string().as_str(), "Submitters do not match!")
}

#[actix_web::test]
async fn delete_record() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::RecordModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let record = create_test_record(&mut conn, user_id).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/aredl/records/{}", record.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}
