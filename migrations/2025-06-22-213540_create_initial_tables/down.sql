-- down.sql

DROP TABLE IF EXISTS sale_items;
DROP TABLE IF EXISTS sales;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS users;

DROP FUNCTION IF EXISTS trigger_set_timestamp;
