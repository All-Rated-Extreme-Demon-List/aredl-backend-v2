use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PageQuery {
    pub per_page: Option<i64>,
    pub page: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Paginated<T> {
    pub page: i64,
    pub per_page: i64,
    pub pages: i64,
    pub data: Vec<T>,
}

impl<T> Paginated<T> {
    pub fn from_data(query: PageQuery, pages: i64, data: Vec<T>) -> Self {
        Self {
            page: query.page,
            per_page: query.per_page.unwrap_or(20),
            pages,
            data,
        }
    }
}

impl PageQuery {
    pub fn offset(&self) -> i64 {
        self.per_page.unwrap_or(20) * (self.page - 1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(20)
    }
}