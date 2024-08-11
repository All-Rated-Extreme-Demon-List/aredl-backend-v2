use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PageQuery<const D: i64> {
    pub per_page: Option<i64>,
    pub page: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Paginated<T> {
    pub page: i64,
    pub per_page: i64,
    pub pages: i64,
    #[serde(flatten)]
    pub data: T,
}

impl<T> Paginated<T> {
    pub fn from_data<const D: i64>(query: PageQuery<D>, pages: i64, data: T) -> Self {
        Self {
            page: query.page,
            per_page: query.per_page.unwrap_or(D),
            pages,
            data,
        }
    }
}

impl<const D: i64> PageQuery<D> {
    pub fn offset(&self) -> i64 {
        self.per_page.unwrap_or(D) * (self.page - 1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(D)
    }
}