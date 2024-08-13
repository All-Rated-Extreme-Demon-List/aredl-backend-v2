use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PageQuery<const D: i64> {
    pub per_page: Option<i64>,
    pub page: Option<i64>,
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
    pub fn from_data<const D: i64>(query: PageQuery<D>, count: i64, data: T) -> Self {
        let pages = (count / query.per_page()) + 1;
        Self {
            page: query.page(),
            per_page: query.per_page(),
            pages,
            data,
        }
    }
}

impl<const D: i64> PageQuery<D> {
    pub fn offset(&self) -> i64 {
        self.per_page.unwrap_or(D) * (self.page() - 1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(D)
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1)
    }
}