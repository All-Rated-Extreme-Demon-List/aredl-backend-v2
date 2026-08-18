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
        Self::from_data_maybe_limit(query, count, data, false)
    }
    
    pub fn from_data_maybe_limit<const D: i64, const M: i64>(
        query: PageQuery<D, M>,
        count: i64,
        data: T,
        ignore_limit: bool,
    ) -> Self {
        let count = count.max(0);
        let per_page = query.per_page_maybe_limit(ignore_limit);
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
        self.per_page_maybe_limit(false)
    }

    pub fn per_page_maybe_limit(&self, ignore_limit: bool) -> i64 {
        let requested = self.per_page.unwrap_or(D);
        if ignore_limit {
            requested.max(1)
        } else {
            requested.clamp(1, M)
        }
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn offset(&self) -> i64 {
        self.offset_maybe_limit(false)
    }

    pub fn offset_maybe_limit(&self, ignore_limit: bool) -> i64 {
        self.per_page_maybe_limit(ignore_limit)
            .saturating_mul(self.page() - 1)
    }
}
