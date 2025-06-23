-- migrations/{timestamp}_initial_schema/up.sql

-- Créer une fonction pour mettre à jour automatiquement le champ 'updated_at'
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Table des utilisateurs
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    must_change_password BOOLEAN NOT NULL DEFAULT FALSE, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des produits de base (ex: "Guinness", "Coca-Cola")
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    base_unit_name TEXT NOT NULL, -- ex: "bouteille", "canette"
    total_stock_in_base_units INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON products FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des unités de conditionnement (ex: "Casier de 24", "Bouteille unique")
CREATE TABLE packaging_units (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    contained_base_units INT NOT NULL, -- Le nombre d'unités de base (ex: 24)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON packaging_units FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table de liaison: définit les "produits vendables" avec leur prix
CREATE TABLE product_offerings (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    packaging_unit_id UUID NOT NULL REFERENCES packaging_units(id) ON DELETE RESTRICT,
    price NUMERIC(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Un produit ne peut être offert qu'une seule fois dans le même conditionnement
    UNIQUE (product_id, packaging_unit_id)
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON product_offerings FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des ventes (l'en-tête de la facture)
CREATE TABLE sales (
    id UUID PRIMARY KEY,
    sale_number TEXT NOT NULL UNIQUE,
    total_amount NUMERIC(10, 2) NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON sales FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des articles de vente (les lignes de la facture)
CREATE TABLE sale_items (
    id UUID PRIMARY KEY,
    sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_offering_id UUID NOT NULL REFERENCES product_offerings(id) ON DELETE RESTRICT,
    quantity INT NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL, -- Prix au moment de la vente
    total_price NUMERIC(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON sale_items FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();