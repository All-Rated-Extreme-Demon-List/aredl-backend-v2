#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        schema::shifts,
        shifts::{
            recurring::RecurringShift,
            test_utils::{create_test_recurring_shift, create_test_shift},
        },
        {test_utils::*, users::test_utils::create_test_user},
    },
    actix_web::test::{self, read_body_json},
    chrono::NaiveDate,
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    serde_json::json,
};

#[actix_web::test]
async fn get_shifts_list() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_shift(&db, user_id, false).await;
    let req = test::TestRequest::get()
        .uri("/shifts")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_ne!(body["data"].as_array().unwrap().len(), 0)
}

#[actix_web::test]
async fn get_my_shifts() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_shift(&db, user_id, false).await;
    let req = test::TestRequest::get()
        .uri("/shifts/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["user"].as_object().unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
            == user_id.to_string()))
}

#[actix_web::test]
async fn patch_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let shift_id = create_test_shift(&db, user_id, false).await;
    let patch_data = json!({
        "status": "Completed"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/shifts/{}", shift_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&patch_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        patch_data["status"].as_str().unwrap(),
        body["status"].as_str().unwrap(),
        "Statuses do not match!"
    )
}

#[actix_web::test]
async fn delete_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let shift_id = create_test_shift(&db, user_id, false).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/shifts/{}", shift_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn create_recurring_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let insert_data = json!({
        "user_id": user_id,
        "weekday": "Friday",
        "start_hour": 12,
        "duration": 1,
        "target_count": 20
    });
    let req = test::TestRequest::post()
        .uri("/shifts/recurring")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&insert_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["user_id"].as_str().unwrap(), user_id.to_string());
}

#[actix_web::test]
async fn list_recurring_shifts() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_recurring_shift(&db, user_id).await;
    let req = test::TestRequest::get()
        .uri("/shifts/recurring")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["user"]["id"].as_str().unwrap() == user_id.to_string()));
}

#[actix_web::test]
async fn patch_recurring_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let recurring_id = create_test_recurring_shift(&db, user_id).await;
    let patch_data = json!({
        "target_count": 42
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/shifts/recurring/{}", recurring_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&patch_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["target_count"].as_i64().unwrap(), 42);
}

#[actix_web::test]
async fn delete_recurring_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let recurring_id = create_test_recurring_shift(&db, user_id).await;
    let req = test::TestRequest::delete()
        .uri(&format!("/shifts/recurring/{}", recurring_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), recurring_id.to_string());
}

#[actix_web::test]
async fn create_shifts_from_recurring() {
    let (_, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;

    create_test_recurring_shift(&db, user_id).await;

    let friday_date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();
    let created_shifts = RecurringShift::create_shifts(&mut db.connection().unwrap(), friday_date)
        .expect("Failed to create shifts from recurring template");

    assert_eq!(created_shifts.len(), 1, "Should create one shift");
    assert_eq!(
        created_shifts[0].user_id, user_id,
        "Shift should be assigned to the correct user"
    );
    assert_eq!(
        created_shifts[0].target_count, 20,
        "Target count should match recurring shift"
    );

    let db_shifts: Vec<crate::shifts::Shift> = shifts::table
        .filter(shifts::user_id.eq(user_id))
        .load(&mut db.connection().unwrap())
        .expect("Failed to load shifts from database");

    assert_eq!(db_shifts.len(), 1, "Should have one shift in database");
    assert_eq!(
        db_shifts[0].user_id, user_id,
        "Database shift should be assigned to correct user"
    );
    assert_eq!(
        db_shifts[0].target_count, 20,
        "Database shift target count should match"
    );
}

#[actix_web::test]
async fn create_shifts_no_duplicates() {
    use crate::schema::shifts;
    use crate::shifts::recurring::RecurringShift;
    use chrono::NaiveDate;
    use diesel::{QueryDsl, RunQueryDsl};

    let (_, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;

    create_test_recurring_shift(&db, user_id).await;

    let friday_date = NaiveDate::from_ymd_opt(2025, 7, 11).unwrap();

    let created_shifts_1 =
        RecurringShift::create_shifts(&mut db.connection().unwrap(), friday_date)
            .expect("Failed to create shifts from recurring template (first call)");

    let created_shifts_2 =
        RecurringShift::create_shifts(&mut db.connection().unwrap(), friday_date)
            .expect("Failed to create shifts from recurring template (second call)");

    assert_eq!(
        created_shifts_1.len(),
        1,
        "First call should create one shift"
    );
    assert_eq!(
        created_shifts_2.len(),
        0,
        "Second call should create no shifts (no duplicates)"
    );

    let db_shifts: Vec<crate::shifts::Shift> = shifts::table
        .filter(shifts::user_id.eq(user_id))
        .load(&mut db.connection().unwrap())
        .expect("Failed to load shifts from database");

    assert_eq!(db_shifts.len(), 1, "Should have only one shift in database");
}

#[actix_web::test]
async fn create_shifts_wrong_weekday() {
    use crate::shifts::recurring::RecurringShift;
    use chrono::NaiveDate;

    let (_, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;

    create_test_recurring_shift(&db, user_id).await;

    let monday_date = NaiveDate::from_ymd_opt(2025, 7, 7).unwrap();

    let created_shifts = RecurringShift::create_shifts(&mut db.connection().unwrap(), monday_date)
        .expect("Failed to create shifts from recurring template");

    assert_eq!(
        created_shifts.len(),
        0,
        "Should create no shifts for wrong weekday"
    );
}
