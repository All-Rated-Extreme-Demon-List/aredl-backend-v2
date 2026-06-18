use super::sync_patreon_plus::apply_patreon_plus_sync;
use crate::aredl::{
    levels::test_utils::create_test_level as create_aredl_test_level,
    submissions::{
        test_utils::{
            create_test_submission as create_aredl_test_submission,
            set_test_submission_status as set_aredl_submission_status,
            test_submission_priorities as aredl_submission_priorities,
        },
        SubmissionStatus as AredlSubmissionStatus,
    },
};
use crate::arepl::{
    levels::test_utils::create_test_level as create_arepl_test_level,
    submissions::{
        test_utils::{
            create_test_submission as create_arepl_test_submission,
            set_test_submission_status as set_arepl_submission_status,
            test_submission_priorities as arepl_submission_priorities,
        },
        SubmissionStatus as AreplSubmissionStatus,
    },
};
use crate::auth::{oauth::OAuthProvider, test_utils::seed_connected_account};
use crate::roles::test_utils::{add_user_to_role, create_test_role_with_desc, users_with_role};
use crate::test_utils::init_test_app;
use crate::users::test_utils::create_test_user;
use std::collections::HashSet;

#[actix_web::test]
async fn patreon_sync_sets_plus_role_to_active_linked_patrons() {
    let (_, db, _, _) = init_test_app().await;
    let plus_role = create_test_role_with_desc(&db, 5, "plus").await;
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

    let active_ids = HashSet::from(["patron_active".to_string(), "patron_unlinked".to_string()]);

    let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();
    assert_eq!(synced.matched_user_ids, vec![active_user]);
    assert_eq!(synced.removed_user_count, 1);
    assert_eq!(synced.aredl_prioritized_count, 0);
    assert_eq!(synced.arepl_prioritized_count, 0);

    let plus_users = users_with_role(&db, plus_role);
    assert_eq!(plus_users, HashSet::from([active_user]));

    let stale_users = users_with_role(&db, stale_user_role);
    assert_eq!(stale_users, HashSet::from([unlinked_user]));
}

#[actix_web::test]
async fn patreon_sync_clears_plus_role_when_no_active_patrons_match() {
    let (_, db, _, _) = init_test_app().await;
    let plus_role = create_test_role_with_desc(&db, 5, "plus").await;
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
    create_test_role_with_desc(&db, 5, "plus").await;

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

    let aredl_active_level = create_aredl_test_level(&db).await;
    let aredl_active_submission =
        create_aredl_test_submission(aredl_active_level, active_user, &db).await;
    let aredl_inactive_level = create_aredl_test_level(&db).await;
    let aredl_inactive_submission =
        create_aredl_test_submission(aredl_inactive_level, inactive_user, &db).await;
    let aredl_non_pending_level = create_aredl_test_level(&db).await;
    let aredl_non_pending_submission =
        create_aredl_test_submission(aredl_non_pending_level, active_user, &db).await;
    set_aredl_submission_status(
        &db,
        aredl_non_pending_submission,
        AredlSubmissionStatus::Claimed,
    );

    let arepl_active_level = create_arepl_test_level(&db).await;
    let arepl_active_submission =
        create_arepl_test_submission(arepl_active_level, active_user, &db).await;
    let arepl_inactive_level = create_arepl_test_level(&db).await;
    let arepl_inactive_submission =
        create_arepl_test_submission(arepl_inactive_level, inactive_user, &db).await;
    let arepl_non_pending_level = create_arepl_test_level(&db).await;
    let arepl_non_pending_submission =
        create_arepl_test_submission(arepl_non_pending_level, active_user, &db).await;
    set_arepl_submission_status(
        &db,
        arepl_non_pending_submission,
        AreplSubmissionStatus::Claimed,
    );

    let active_ids = HashSet::from(["patron_active".to_string()]);
    let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();

    assert_eq!(synced.matched_user_ids, vec![active_user]);
    assert_eq!(synced.aredl_prioritized_count, 1);
    assert_eq!(synced.arepl_prioritized_count, 1);

    let aredl_priorities = aredl_submission_priorities(
        &db,
        [
            aredl_active_submission,
            aredl_inactive_submission,
            aredl_non_pending_submission,
        ],
    );

    assert!(aredl_priorities[&aredl_active_submission]);
    assert!(!aredl_priorities[&aredl_inactive_submission]);
    assert!(!aredl_priorities[&aredl_non_pending_submission]);

    let arepl_priorities = arepl_submission_priorities(
        &db,
        [
            arepl_active_submission,
            arepl_inactive_submission,
            arepl_non_pending_submission,
        ],
    );

    assert!(arepl_priorities[&arepl_active_submission]);
    assert!(!arepl_priorities[&arepl_inactive_submission]);
    assert!(!arepl_priorities[&arepl_non_pending_submission]);
}
