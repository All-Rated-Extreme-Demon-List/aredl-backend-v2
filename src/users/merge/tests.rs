#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::test_utils::{create_test_level, create_test_level_with_record},
            records::test_utils::{
                create_test_record, get_test_record, test_records_for_level, test_records_for_user,
            },
            submissions::test_utils::{
                create_test_submission, get_test_submission_optional, set_test_submission_locked,
                test_submission_history_count,
            },
        },
        auth::{create_test_token, Permission},
        test_utils::*,
        users::{
            merge::test_utils::create_test_merge_log,
            test_utils::{create_test_placeholder_user, create_test_user},
        },
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn direct_merge() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, user_1_record_id) = create_test_level_with_record(&db, user_1_id).await;
    let (_, user_2_record_id) = create_test_level_with_record(&db, user_2_id).await;

    let merge_data = json!({
        "primary_user": user_1_id.to_string(),
        "secondary_user": user_2_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&merge_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let records = test_records_for_user(&db, user_1_id);

    assert_eq!(records.len(), 2, "User does not have exactly 2 records!");
    assert!(
        records.iter().any(|x| x.id == user_1_record_id),
        "Did not return first record!"
    );
    assert!(
        records.iter().any(|x| x.id == user_2_record_id),
        "Did not return second record!"
    );
}

#[actix_web::test]
async fn list_merge_logs() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let log_id = create_test_merge_log(&db, user_1_id, user_2_id).await;

    let req = test::TestRequest::get()
        .uri("/users/merge/logs")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["id"] == log_id.to_string()));
}

#[actix_web::test]
async fn direct_merge_secondary_accepted_beats_primary_pending_preserves_history() {
    let (app, db, auth, _) = init_test_app().await;

    let (primary_id, _) = create_test_user(&db, None).await;
    let (secondary_id, _) = create_test_placeholder_user(&db).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::DirectMerge)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();

    let level_id = create_test_level(&db).await;

    let primary_sub_id = create_test_submission(level_id, primary_id, &db).await;

    set_test_submission_locked(&db, primary_sub_id, true);

    let secondary_record_id = create_test_record(&db, secondary_id, level_id).await;
    let secondary_sub_id = get_test_record(&db, secondary_record_id).submission_id;

    set_test_submission_locked(&db, secondary_sub_id, true);

    let primary_hist_before = test_submission_history_count(&db, primary_sub_id);
    let secondary_hist_before = test_submission_history_count(&db, secondary_sub_id);

    assert_eq!(primary_hist_before, 2);
    assert_eq!(secondary_hist_before, 2);

    let merge_data = json!({
        "primary_user": primary_id.to_string(),
        "secondary_user": secondary_id.to_string(),
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&merge_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    // pending submission should be deleted (accepted wins)
    let primary_exists = get_test_submission_optional(&db, primary_sub_id).is_some();
    assert!(!primary_exists);

    // accepted submission stays
    let secondary_exists = get_test_submission_optional(&db, secondary_sub_id).is_some();
    assert!(secondary_exists);

    // history preserved
    let secondary_hist_after = test_submission_history_count(&db, secondary_sub_id);

    assert_eq!(
        secondary_hist_after,
        primary_hist_before + secondary_hist_before
    );

    let rec_owner = get_test_record(&db, secondary_record_id).submitted_by;
    assert_eq!(rec_owner, primary_id);
}

#[actix_web::test]
async fn direct_merge_both_accepted_primary_kept_history_preserved() {
    let (app, db, auth, _) = init_test_app().await;

    let (primary_id, _) = create_test_user(&db, None).await;
    let (secondary_id, _) = create_test_placeholder_user(&db).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::DirectMerge)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();

    let level_id = create_test_level(&db).await;

    let primary_record_id = create_test_record(&db, primary_id, level_id).await;
    let secondary_record_id = create_test_record(&db, secondary_id, level_id).await;

    let primary_sub_id = get_test_record(&db, primary_record_id).submission_id;
    let secondary_sub_id = get_test_record(&db, secondary_record_id).submission_id;

    set_test_submission_locked(&db, primary_sub_id, true);
    set_test_submission_locked(&db, secondary_sub_id, true);

    let primary_hist_before = test_submission_history_count(&db, primary_sub_id);
    let secondary_hist_before = test_submission_history_count(&db, secondary_sub_id);
    assert_eq!(primary_hist_before, 2);
    assert_eq!(secondary_hist_before, 2);

    let merge_data = json!({
        "primary_user": primary_id.to_string(),
        "secondary_user": secondary_id.to_string(),
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&merge_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    // secondary submission should be gone
    let secondary_exists = get_test_submission_optional(&db, secondary_sub_id).is_some();
    assert!(!secondary_exists);

    // primary submission should remain
    let primary_exists = get_test_submission_optional(&db, primary_sub_id).is_some();
    assert!(primary_exists);

    // primary history should now include both
    let primary_hist_after = test_submission_history_count(&db, primary_sub_id);
    assert_eq!(
        primary_hist_after,
        primary_hist_before + secondary_hist_before
    );

    // only one record for the level, owned by primary
    let records = test_records_for_level(&db, level_id);
    assert_eq!(records.len(), 1);

    let kept_owner = records[0].submitted_by;
    assert_eq!(kept_owner, primary_id);
}
