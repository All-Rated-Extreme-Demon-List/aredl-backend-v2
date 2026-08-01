use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const DEFAULT_MAX_PER_PAGE: i64 = 100;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PageQuery<const D: i64, const M: i64 = DEFAULT_MAX_PER_PAGE> {
    pub per_page: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Paginated<T> {
    /// The currently requested page.
    pub page: i64,
    /// The amount of items per page.
    pub per_page: i64,
    /// The total amount of available pages with these settings.
    pub pages: i64,
    /// The total amount of available items with these settings.
    pub count: i64,
    #[serde(flatten)]
    pub data: T,
}

impl<T> Paginated<T> {
    pub fn from_data<const D: i64, const M: i64>(
        query: PageQuery<D, M>,
        count: i64,
        data: T,
    ) -> Self {
        let count = count.max(0);
        let per_page = query.per_page();
        let pages = if count == 0 {
            0
        } else {
            ((count - 1) / per_page) + 1
        };
        Self {
            page: query.page(),
            per_page,
            pages,
            count,
            data,
        }
    }
}

impl<const D: i64, const M: i64> PageQuery<D, M> {
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(D).clamp(1, M)
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn offset(&self) -> i64 {
        self.per_page().saturating_mul(self.page() - 1)
    }
}
