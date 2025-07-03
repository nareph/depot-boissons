// @generated automatically by Diesel CLI.

diesel::table! {
    products (id) {
        id -> Uuid,
        name -> Text,
        packaging_description -> Text,
        sku -> Nullable<Text>,
        stock_in_sale_units -> Int4,
        price_per_sale_unit -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sale_items (id) {
        id -> Uuid,
        sale_id -> Uuid,
        product_id -> Uuid,
        quantity -> Int4,
        unit_price -> Numeric,
        total_price -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sales (id) {
        id -> Uuid,
        sale_number -> Text,
        total_amount -> Numeric,
        date -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        password -> Text,
        name -> Text,
        role -> Text,
        must_change_password -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(sale_items -> products (product_id));
diesel::joinable!(sale_items -> sales (sale_id));

diesel::allow_tables_to_appear_in_same_query!(
    products,
    sale_items,
    sales,
    users,
);
