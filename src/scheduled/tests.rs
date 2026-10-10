#[cfg(test)]
use {
    super::{sync_patreon_plus::apply_patreon_plus_sync, test_utils::get_matview_refresh_time},
    crate::{
        auth::{
            create_test_token, oauth::OAuthProvider, test_utils::seed_connected_account, Permission,
        },
        list::{
            levels::test_utils::{create_test_level, create_test_level_for_list},
            submissions::{
                test_utils::{
                    create_test_submission, create_test_submission_for_list,
                    set_test_submission_status, test_submission_priorities,
                },
                SubmissionStatus,
            },
            List,
        },
        roles::test_utils::{
            add_user_to_role, create_test_role_with_desc, create_test_role_with_permission,
            users_with_role,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::{http::StatusCode, test},
    std::collections::HashSet,
};

#[actix_web::test]
async fn patreon_sync_sets_plus_role_to_active_linked_patrons() {
    let (_, db, _, _) = init_test_app().await;
    let plus_role = create_test_role_with_permission(&db, 5, Permission::SubmissionPriority).await;
    let stale_user_role = create_test_role_with_desc(&db, 1, "stale").await;

    let (active_user, _) = create_test_user(&db, None).await;
    let (inactive_user, _) = create_test_user(&db, None).await;
    let (unlinked_user, _) = create_test_user(&db, None).await;

    seed_connected_account(
        &db,
        active_user,
        OAuthProvider::Patreon,
        "patron_active",
        None,
    );
    seed_connected_account(
        &db,
        inactive_user,
        OAuthProvider::Patreon,
        "patron_inactive",
        None,
    );
    add_user_to_role(&db, plus_role, inactive_user).await;
    add_user_to_role(&db, stale_user_role, unlinked_user).await;

    let active_ids = HashSet::from(["patron_active".to_owned(), "patron_unlinked".to_owned()]);

    let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();
    assert_eq!(synced.matched_user_ids, vec![active_user]);
    assert_eq!(synced.removed_user_count, 1);
    assert_eq!(synced.prioritized_count, 0);

    let plus_users = users_with_role(&db, plus_role);
    assert_eq!(plus_users, HashSet::from([active_user]));

    let stale_users = users_with_role(&db, stale_user_role);
    assert_eq!(stale_users, HashSet::from([unlinked_user]));
}

#[actix_web::test]
async fn maintenance_routes_require_maintenance_permission() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, Some(Permission::UserModify)).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    for uri in [
        "/scheduled/matviews/refresh",
        "/scheduled/patreon/sync",
        "/scheduled/level-data/edel/refresh",
        "/scheduled/level-data/nlw/refresh",
        "/scheduled/cleanup",
        "/classic/levels/history/rebuild",
        "/classic/statistics/submissions/daily/rebuild",
        "/classic/levels/1/gddl/refresh",
        "/shifts/recurring/create-shifts",
    ] {
        let response =
            test::call_service(&app, test::TestRequest::post().uri(uri).to_request()).await;
        assert_error_response!(response, StatusCode::UNAUTHORIZED, None);
        let request = test::TestRequest::post()
            .uri(uri)
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_error_response!(response, StatusCode::FORBIDDEN, None);
    }
}

#[actix_web::test]
async fn matview_refresh_rejects_unknown_view() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, Some(Permission::MaintenanceRun)).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let request = test::TestRequest::post()
        .uri("/scheduled/matviews/refresh?view=users")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_error_response!(
        response,
        StatusCode::BAD_REQUEST,
        Some("Query deserialize error: unknown variant `users`, expected one of `user_leaderboard`, `country_leaderboard`, `clans_leaderboard`, `country_created_levels`, `clans_created_levels`, `record_totals`, `submission_totals`")
    );
}

#[actix_web::test]
async fn matview_refresh_requires_view() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, Some(Permission::MaintenanceRun)).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let request = test::TestRequest::post()
        .uri("/scheduled/matviews/refresh")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_error_response!(
        response,
        StatusCode::BAD_REQUEST,
        Some("Query deserialize error: missing field `view`")
    );
}

#[actix_web::test]
async fn matview_refresh_updates_timestamp() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, Some(Permission::MaintenanceRun)).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let started_at = chrono::Utc::now();
    let request = test::TestRequest::post()
        .uri("/scheduled/matviews/refresh?view=record_totals")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(test::read_body(response).await.is_empty());
    assert!(get_matview_refresh_time(&db, "record_totals") >= started_at);
}

#[actix_web::test]
async fn patreon_sync_clears_plus_role_when_no_active_patrons_match() {
    let (_, db, _, _) = init_test_app().await;
    let plus_role = create_test_role_with_permission(&db, 5, Permission::SubmissionPriority).await;
    let (inactive_user, _) = create_test_user(&db, None).await;

    seed_connected_account(
        &db,
        inactive_user,
        OAuthProvider::Patreon,
        "patron_inactive",
        None,
    );
    add_user_to_role(&db, plus_role, inactive_user).await;

    let active_ids = HashSet::new();
    let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();
    assert!(synced.matched_user_ids.is_empty());
    assert_eq!(synced.removed_user_count, 1);
    assert!(users_with_role(&db, plus_role).is_empty());
}

#[actix_web::test]
async fn patreon_sync_prioritizes_pending_submissions_from_active_linked_patrons() {
    let (_, db, _, _) = init_test_app().await;
    create_test_role_with_permission(&db, 5, Permission::SubmissionPriority).await;

    let (active_user, _) = create_test_user(&db, None).await;
    let (inactive_user, _) = create_test_user(&db, None).await;

    seed_connected_account(
        &db,
        active_user,
        OAuthProvider::Patreon,
        "patron_active",
        None,
    );
    seed_connected_account(
        &db,
        inactive_user,
        OAuthProvider::Patreon,
        "patron_inactive",
        None,
    );

    let aredl_active_level = create_test_level(&db).await;
    let aredl_active_submission =
        create_test_submission(aredl_active_level, active_user, &db).await;
    let aredl_inactive_level = create_test_level(&db).await;
    let aredl_inactive_submission =
        create_test_submission(aredl_inactive_level, inactive_user, &db).await;
    let aredl_non_pending_level = create_test_level(&db).await;
    let aredl_non_pending_submission =
        create_test_submission(aredl_non_pending_level, active_user, &db).await;
    set_test_submission_status(&db, aredl_non_pending_submission, SubmissionStatus::Claimed);

    let platformer_level = create_test_level_for_list(&db, List::Platformer).await;
    let platformer_submission =
        create_test_submission_for_list(platformer_level, active_user, &db, List::Platformer).await;

    let active_ids = HashSet::from(["patron_active".to_owned()]);
    let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();

    assert_eq!(synced.matched_user_ids, vec![active_user]);
    assert_eq!(synced.prioritized_count, 2);

    let aredl_priorities = test_submission_priorities(
        &db,
        [
            aredl_active_submission,
            aredl_inactive_submission,
            aredl_non_pending_submission,
            platformer_submission,
        ],
    );

    assert!(aredl_priorities[&aredl_active_submission]);
    assert!(aredl_priorities[&platformer_submission]);
    assert!(!aredl_priorities[&aredl_inactive_submission]);
    assert!(!aredl_priorities[&aredl_non_pending_submission]);
}
