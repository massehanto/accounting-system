use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationParams {
    pub const DEFAULT_LIMIT: i64 = 50;
    pub const MAX_LIMIT: i64 = 1000;

    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self { limit, offset }
    }

    pub fn limit(&self) -> i64 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::MAX_LIMIT)
            .max(1)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn page(&self) -> i64 {
        (self.offset() / self.limit()) + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: i64,
    pub total_pages: i64,
    pub total_items: i64,
    pub items_per_page: i64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(
        data: Vec<T>,
        params: &PaginationParams,
        total_items: i64,
    ) -> Self {
        let items_per_page = params.limit();
        let current_page = params.page();
        let total_pages = (total_items + items_per_page - 1) / items_per_page;

        let pagination = PaginationInfo {
            current_page,
            total_pages,
            total_items,
            items_per_page,
            has_next_page: current_page < total_pages,
            has_previous_page: current_page > 1,
        };

        Self { data, pagination }
    }
}

pub struct PaginationBuilder<T> {
    data: Vec<T>,
    total_items: i64,
    params: PaginationParams,
}

impl<T> PaginationBuilder<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            total_items: data.len() as i64,
            data,
            params: PaginationParams::new(None, None),
        }
    }

    pub fn with_params(mut self, params: PaginationParams) -> Self {
        self.params = params;
        self
    }

    pub fn with_total_items(mut self, total: i64) -> Self {
        self.total_items = total;
        self
    }

    pub fn build(self) -> PaginatedResponse<T> {
        PaginatedResponse::new(self.data, &self.params, self.total_items)
    }
}

// Helper trait for paginating database queries
pub trait Paginate {
    fn paginate(self, params: &PaginationParams) -> Self;
}

// This would be implemented for SQL query builders
// Example implementation would depend on the specific database library being used