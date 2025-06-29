// src/queries/mod.rs

// Déclarer les sous-modules
pub mod dashboard_queries;
pub mod password_queries;
pub mod product_queries;
pub mod user_queries;

// Rendre toutes les fonctions publiques accessibles directement via `queries::...`
pub use dashboard_queries::*;
pub use password_queries::*;
pub use product_queries::*;
pub use user_queries::*;
