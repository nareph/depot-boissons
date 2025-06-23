-- migrations/{timestamp}_initial_schema/down.sql

DROP TABLE IF EXISTS sale_items;
DROP TABLE IF EXISTS sales;
DROP TABLE IF EXISTS product_offerings;
DROP TABLE IF EXISTS packaging_units;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS users;

DROP FUNCTION IF EXISTS trigger_set_timestamp;