-- up.sql (Syntaxe pour SQLite)

-- Table des utilisateurs
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, -- UUID stocké comme TEXT
    password TEXT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL,
    must_change_password INTEGER NOT NULL DEFAULT 0, -- BOOLEAN devient INTEGER
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, -- TIMESTAMPTZ devient TEXT
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Trigger pour mettre à jour 'updated_at' sur la table users
CREATE TRIGGER trigger_users_updated_at AFTER UPDATE ON users
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Table des produits
CREATE TABLE products (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    packaging_description TEXT NOT NULL,
    sku TEXT UNIQUE,
    stock_in_sale_units INTEGER NOT NULL DEFAULT 0, -- INT devient INTEGER
    price_per_sale_unit TEXT NOT NULL, -- NUMERIC devient TEXT pour la précision
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name, packaging_description)
);

CREATE TRIGGER trigger_products_updated_at AFTER UPDATE ON products
BEGIN
    UPDATE products SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Table des ventes
CREATE TABLE sales (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL, -- La contrainte de clé étrangère est ajoutée plus tard
    sale_number TEXT NOT NULL UNIQUE,
    total_amount TEXT NOT NULL,
    date TEXT NOT NULL, -- TIMESTAMPTZ devient TEXT
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT
);

CREATE TRIGGER trigger_sales_updated_at AFTER UPDATE ON sales
BEGIN
    UPDATE sales SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE INDEX idx_sales_user_id ON sales (user_id);

-- Table des articles de vente
CREATE TABLE sale_items (
    id TEXT PRIMARY KEY NOT NULL,
    sale_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price TEXT NOT NULL,
    total_price TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sale_id) REFERENCES sales(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT
);

CREATE TRIGGER trigger_sale_items_updated_at AFTER UPDATE ON sale_items
BEGIN
    UPDATE sale_items SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;