// @generated automatically by Diesel CLI.

diesel::table! {
    packaging_units (id) {
        id -> Uuid,
        name -> Text,
        contained_base_units -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    product_offerings (id) {
        id -> Uuid,
        product_id -> Uuid,
        packaging_unit_id -> Uuid,
        price -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        name -> Text,
        base_unit_name -> Text,
        total_stock_in_base_units -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sale_items (id) {
        id -> Uuid,
        sale_id -> Uuid,
        product_offering_id -> Uuid,
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

diesel::joinable!(product_offerings -> packaging_units (packaging_unit_id));
diesel::joinable!(product_offerings -> products (product_id));
diesel::joinable!(sale_items -> product_offerings (product_offering_id));
diesel::joinable!(sale_items -> sales (sale_id));

diesel::allow_tables_to_appear_in_same_query!(
    packaging_units,
    product_offerings,
    products,
    sale_items,
    sales,
    users,
);
