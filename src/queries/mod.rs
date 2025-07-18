// src/queries/mod.rs

// Déclarer les sous-modules
pub mod dashboard_queries;
pub mod password_queries;
pub mod product_queries;
pub mod reporting_queries;
pub mod sale_queries;
pub mod user_queries;

// Rendre toutes les fonctions publiques accessibles directement via `queries::...`
pub use dashboard_queries::*;
pub use password_queries::*;
pub use product_queries::*;
pub use reporting_queries::*;
pub use sale_queries::*;
pub use user_queries::*;

// Types communs pour la pagination et le tri
#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 10,
        }
    }
}

// Trait générique pour les résultats paginés
pub trait PaginatedResult<T> {
    fn new(items: Vec<T>, total_count: i64, current_page: i64, per_page: i64) -> Self;

    fn total_pages(&self) -> i64;
    fn has_previous(&self) -> bool;
    fn has_next(&self) -> bool;
}

// Implémentation générique d'un résultat paginé
#[derive(Debug, Clone)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub current_page: i64,
    pub total_pages: i64,
    pub per_page: i64,
}

impl<T> PaginatedResult<T> for PaginatedList<T> {
    fn new(items: Vec<T>, total_count: i64, current_page: i64, per_page: i64) -> Self {
        let total_pages = (total_count + per_page - 1) / per_page;
        Self {
            items,
            total_count,
            current_page,
            total_pages,
            per_page,
        }
    }

    fn total_pages(&self) -> i64 {
        self.total_pages
    }

    fn has_previous(&self) -> bool {
        self.current_page > 1
    }

    fn has_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

// Fonctions utilitaires communes
pub fn parse_sort_order(sort_order: &str) -> SortOrder {
    match sort_order.to_lowercase().as_str() {
        "desc" | "descending" => SortOrder::Desc,
        _ => SortOrder::Asc,
    }
}

// Macro pour simplifier la création de filtres avec tri
#[macro_export]
macro_rules! create_filter {
    ($filter_type:ty, $($field:ident: $value:expr),*) => {
        <$filter_type>::new($($field: $value),*)
    };
}
