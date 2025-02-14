#[cfg(test)]
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use crate::auth::create_test_token;
#[cfg(test)]
use crate::schema::users;
#[cfg(test)]
use crate::test_utils::{create_test_user, init_test_app};

#[actix_web::test]
async fn get_authenticated_user() {
	let (app, mut conn, auth) = init_test_app().await;

	let (user_id, username) = create_test_user(&mut conn, None).await;
	let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

	let req = test::TestRequest::get()
		.uri("/users/@me")
		.insert_header(("Authorization", format!("Bearer {}", token)))
		.to_request();

	let resp = test::call_service(&app, req).await;
	assert!(resp.status().is_success());

	let user: serde_json::Value = test::read_body_json(resp).await;
	assert_eq!(user["username"], username);
}

#[actix_web::test]
async fn update_authenticated_user() {
	let (app, mut conn, auth) = init_test_app().await;
	let (user_id, _) = create_test_user(&mut conn, None).await;
	let user_token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

	let update_payload = json!({
		"global_name": "Updated Name",
		"description": "Updated description",
		"ban_level": 1,
		"country": 10
	});

	let req = test::TestRequest::patch()
		.uri("/users/@me")
		.insert_header(("Authorization", format!("Bearer {}", user_token)))
		.set_json(&update_payload)
		.to_request();
	
	let resp = test::call_service(&app, req).await;
	assert!(resp.status().is_success());

	let updated_user: serde_json::Value = test::read_body_json(resp).await;
	assert_eq!(updated_user["global_name"], "Updated Name");
	assert_eq!(updated_user["description"], "Updated description");
	assert_eq!(updated_user["ban_level"], 1);
	assert_eq!(updated_user["country"], 10);

	let req = test::TestRequest::get()
		.uri("/users/@me")
		.insert_header(("Authorization", format!("Bearer {}", user_token)))
		.to_request();

	let resp = test::call_service(&app, req).await;
	assert!(resp.status().is_success());

	let user: serde_json::Value = test::read_body_json(resp).await;
	assert_eq!(user["global_name"], "Updated Name");
	assert_eq!(user["description"], "Updated description");
	assert_eq!(user["ban_level"], 1);
	assert_eq!(user["country"], 10);

}

#[actix_web::test]
async fn update_authenticated_user_banned() {
	let (app, mut conn, auth) = init_test_app().await;
	let (user_id, _) = create_test_user(&mut conn, None).await;
	let user_token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

	diesel::update(users::table.filter(users::id.eq(user_id)))
		.set(users::ban_level.eq(2))
		.execute(&mut conn)
		.expect("Failed to ban user");

	let update_payload = json!({
		"ban_level": 1
	});

	let req = test::TestRequest::patch()
		.uri("/users/@me")
		.insert_header(("Authorization", format!("Bearer {}", user_token)))
		.set_json(&update_payload)
		.to_request();
	
	let resp = test::call_service(&app, req).await;
	assert_eq!(resp.status().as_u16(), 403);

}

#[actix_web::test]
async fn update_authenticated_user_country_cooldown() {
	let (app, mut conn, auth) = init_test_app().await;
	let (user_id, _) = create_test_user(&mut conn, None).await;
	let user_token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

	diesel::update(users::table.filter(users::id.eq(user_id)))
		.set(users::last_country_update.eq(chrono::Utc::now().naive_utc()))
		.execute(&mut conn)
		.expect("Failed to update last country update");

	let update_payload = json!({
		"country": 10
	});

	let req = test::TestRequest::patch()
		.uri("/users/@me")
		.insert_header(("Authorization", format!("Bearer {}", user_token)))
		.set_json(&update_payload)
		.to_request();
	
	let resp = test::call_service(&app, req).await;
	assert_eq!(resp.status().as_u16(), 400);

}
