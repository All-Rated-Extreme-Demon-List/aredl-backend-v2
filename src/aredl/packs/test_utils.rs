#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use crate::schema::aredl::{pack_tiers, packs};
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};

#[cfg(test)]
pub async fn create_test_pack_tier(db: &Arc<DbAppState>) -> Uuid {
    let tier_id = Uuid::new_v4();
    diesel::insert_into(pack_tiers::table)
        .values((
            pack_tiers::id.eq(tier_id),
            pack_tiers::name.eq("Test Tier"),
            pack_tiers::color.eq("#abcdef"),
            pack_tiers::placement.eq(1),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test pack tier");
    tier_id
}

#[cfg(test)]
pub async fn create_test_pack(db: &Arc<DbAppState>) -> Uuid {
    let tier_id = create_test_pack_tier(db).await;
    let pack_id = Uuid::new_v4();
    diesel::insert_into(packs::table)
        .values((
            packs::id.eq(pack_id),
            packs::name.eq("Test Pack"),
            packs::tier.eq(tier_id),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to create test pack");

    pack_id
}
