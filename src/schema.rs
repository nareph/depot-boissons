// @generated automatically by Diesel CLI.

diesel::table! {
    products (id) {
        id -> Text,
        name -> Text,
        packaging_description -> Text,
        sku -> Nullable<Text>,
        stock_in_sale_units -> Integer,
        price_per_sale_unit -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    sale_items (id) {
        id -> Text,
        sale_id -> Text,
        product_id -> Text,
        quantity -> Integer,
        unit_price -> Text,
        total_price -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    sales (id) {
        id -> Text,
        user_id -> Text,
        sale_number -> Text,
        total_amount -> Text,
        date -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        password -> Text,
        name -> Text,
        role -> Text,
        must_change_password -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(sale_items -> products (product_id));
diesel::joinable!(sale_items -> sales (sale_id));
diesel::joinable!(sales -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(products, sale_items, sales, users,);
