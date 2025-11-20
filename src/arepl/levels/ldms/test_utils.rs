#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use super::{LevelLDMStatus, LevelLDMType};
#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use crate::schema::arepl::level_ldms;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_ldm(db: &Arc<DbAppState>, level_id: Uuid, user: Uuid) -> Uuid {
    let mut rng = rand::rng();
    let ldm_id = rng.random_range(1..=100000000);
    let level_uuid = Uuid::new_v4();

    diesel::insert_into(level_ldms::table)
        .values((
            level_ldms::id.eq(level_uuid),
            level_ldms::added_by.eq(user),
            level_ldms::level_id.eq(level_id),
            level_ldms::ldm_id.eq(ldm_id),
            level_ldms::description.eq("Test"),
            level_ldms::status.eq(LevelLDMStatus::Allowed),
            level_ldms::id_type.eq(LevelLDMType::Bugfix),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test LDM id");

    level_uuid
}
