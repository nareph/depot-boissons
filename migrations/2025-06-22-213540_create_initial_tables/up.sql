-- up.sql (Version Corrigée et Simplifiée)

-- Créer la fonction de trigger pour 'updated_at' (ne change pas)
CREATE OR REPLACE FUNCTION trigger_set_timestamp() RETURNS TRIGGER AS $$
BEGIN NEW.updated_at = NOW(); RETURN NEW; END;
$$ LANGUAGE plpgsql;

-- Table des utilisateurs (ne change pas, sauf si vous retirez l'email)
CREATE TABLE users (
    id UUID PRIMARY KEY,
    password TEXT NOT NULL,
    name TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL,
    must_change_password BOOLEAN NOT NULL DEFAULT FALSE, 
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- *** LA NOUVELLE TABLE PRODUCTS ***
-- Chaque ligne est un produit fini (SKU)
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL, -- ex: "Isenbeck", "Supermont"
    packaging_description TEXT NOT NULL, -- ex: "Casier 65cl de 12", "Palette 1.5L de 12"
    sku TEXT UNIQUE, -- ex: "ISEN-65-CAS12". Code unique pour la gestion. Optionnel mais recommandé.
    stock_in_sale_units INT NOT NULL DEFAULT 0, -- Le stock en casiers, palettes, etc.
    price_per_sale_unit NUMERIC(10, 2) NOT NULL, -- Le prix de vente d'un casier, d'une palette...
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Un produit avec un certain packaging ne peut exister qu'une fois
    UNIQUE (name, packaging_description)
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON products FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des ventes (l'en-tête de la facture, ne change pas)
CREATE TABLE sales (
    id UUID PRIMARY KEY,
    sale_number TEXT NOT NULL UNIQUE,
    total_amount NUMERIC(10, 2) NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON sales FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();

-- Table des articles de vente (simplifiée)
CREATE TABLE sale_items (
    id UUID PRIMARY KEY,
    sale_id UUID NOT NULL REFERENCES sales(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    quantity INT NOT NULL, -- Quantité en unités de vente (ex: 2 casiers, 3 palettes)
    unit_price NUMERIC(10, 2) NOT NULL, -- Prix du casier/palette au moment de la vente
    total_price NUMERIC(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TRIGGER set_timestamp BEFORE UPDATE ON sale_items FOR EACH ROW EXECUTE PROCEDURE trigger_set_timestamp();