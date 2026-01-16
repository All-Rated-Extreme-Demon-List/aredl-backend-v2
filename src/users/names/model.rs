use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::roles::RoleResolved;

impl RoleResolved {
    pub fn find_all_public(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        Ok(Self::find_all(conn)?
            .into_iter()
            .filter(|resolved| !resolved.role.hide)
            .filter(|resolved| !resolved.users.is_empty())
            .collect())
    }
}
