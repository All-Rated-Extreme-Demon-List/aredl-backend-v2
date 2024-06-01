use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_levels, aredl_packs};

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_packs)]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
}

impl Pack {

    pub fn find_all(level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let packs = aredl_packs::table
            .inner_join(aredl_pack_levels::table.on(aredl_pack_levels::pack_id.eq(aredl_packs::id)))
            .filter(aredl_pack_levels::level_id.eq(level_id))
            .select(Pack::as_select())
            .load::<Self>(&mut db::connection()?)?;
        Ok(packs)
    }
}