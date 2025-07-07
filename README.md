# Dépôt de Boissons - Système de Gestion

![Screenshot de l'application](/.screenshots/dashboard.png) <!-- Suggestion: Ajoutez un screenshot de votre application ici -->

**Dépôt de Boissons** est une application de bureau moderne conçue pour la gestion complète d'un dépôt de boissons. Développée en **Rust** avec l'interface utilisateur **Slint**, elle offre une solution robuste, performante et multi-plateforme (Windows, macOS, Linux) pour gérer les stocks, les ventes, les clients et les utilisateurs.

L'application est pensée pour fonctionner **offline**, avec toutes les données stockées localement dans une base de données PostgreSQL, garantissant ainsi rapidité et confidentialité.

## ✨ Fonctionnalités Clés

-   **Tableau de Bord Intuitif** : Visualisez en temps réel le chiffre d'affaires du jour, le nombre de ventes et les produits à stock faible.
-   **Gestion des Produits (SKU)** : Chaque produit est géré comme une unité de vente unique (ex: "Casier de 12 bouteilles 65cl"), avec son propre stock et son propre prix, reflétant la logique métier d'un dépôt.
-   **Système de Vente Complet** :
    -   Interface de point de vente (POS) pour créer de nouvelles ventes rapidement.
    -   Gestion d'un panier d'achat avec validation des stocks en temps réel.
    -   Génération et impression de tickets de caisse détaillés.
-   **Historique des Ventes** : Consultez l'historique complet des transactions avec des outils de recherche, de filtrage (par date) et de tri avancés.
-   **Gestion des Utilisateurs et Permissions** :
    -   Système de rôles (Admin, Utilisateur).
    -   Les administrateurs peuvent gérer les comptes utilisateurs (créer, modifier le rôle, supprimer).
    -   Flux de travail sécurisé : les nouveaux utilisateurs et ceux dont le mot de passe a été réinitialisé doivent obligatoirement changer leur mot de passe à leur première connexion.
-   **Sécurité** : Mots de passe hachés avec `bcrypt`, assurant que personne, pas même un administrateur, ne peut voir les mots de passe des utilisateurs.
-   **Fonctionnement Offline** : Toutes les données sont locales, garantissant un accès rapide et une utilisation sans connexion internet.

## 🛠️ Stack Technique

-   **Langage Backend** : [**Rust**](https://www.rust-lang.org/) - Pour sa performance, sa sécurité mémoire et sa robustesse.
-   **Interface Utilisateur** : [**Slint**](https://slint.dev/) - Un toolkit déclaratif et léger pour créer des interfaces natives fluides.
-   **Base de Données** : [**PostgreSQL**](https://www.postgresql.org/) - Un système de gestion de base de données relationnelle open-source puissant et fiable.
-   **ORM / Accès BDD** : [**Diesel**](https://diesel.rs/) - Un ORM et un Query Builder performant et sûr pour Rust.

## 🚀 Démarrage Rapide

### Prérequis

1.  **Toolchain Rust** : Assurez-vous d'avoir Rust et Cargo installés. [Instructions d'installation](https://www.rust-lang.org/tools/install).
2.  **PostgreSQL** : Une instance de PostgreSQL doit être en cours d'exécution. [Instructions d'installation](https://www.postgresql.org/download/).
3.  **Diesel CLI** : Outil en ligne de commande pour gérer les migrations de la base de données.
    ```bash
    cargo install diesel_cli --no-default-features --features postgres
    ```

### Installation et Lancement

1.  **Cloner le dépôt** :
    ```bash
    git clone https://github.com/nareph/depot-boissons.git
    cd depot-boissons
    ```

2.  **Configuration de l'Environnement** :
    -   Copiez le fichier `.env.example` en `.env`.
    -   Modifiez le fichier `.env` pour y mettre l'URL de votre base de données PostgreSQL :
        ```env
        DATABASE_URL=postgres://votre_user:votre_mot_de_passe@localhost/depot_boissons_db
        ```

3.  **Créer la Base de Données** :
    -   Connectez-vous à `psql` et exécutez :
        ```sql
        CREATE DATABASE depot_boissons_db;
        ```

4.  **Appliquer les Migrations** :
    -   Cette commande va créer toutes les tables nécessaires dans votre base de données.
    ```bash
    diesel migration run
    ```

5.  **Peupler la base avec des données de test (Optionnel mais recommandé)** :
    -   L'application contient un "seeder" pour remplir la base avec des utilisateurs, produits et ventes de démonstration.
    -   Lancez l'application avec l'argument `--seed`. Le double tiret (`--`) est important pour passer l'argument à votre programme et non à Cargo.
    ```bash
    cargo run -- --seed
    ```
    -   **Identifiants par défaut** :
        -   Utilisateur : `Administrateur`
        -   Mot de passe : `admin123`
        *(Il vous sera demandé de changer ce mot de passe à la première connexion.)*

6.  **Lancer l'application normally** :
    -   Pour les lancements suivants, utilisez simplement `cargo run`.
    ```bash
    cargo run
    ```

## 🤝 Contribuer

Les contributions sont les bienvenues ! Si vous souhaitez améliorer ce projet, veuillez forker le dépôt, créer une branche pour votre fonctionnalité (`git checkout -b feature/NomDeLaFonctionnalite`), commiter vos changements et ouvrir une Pull Request.

## 📜 Licence

Ce projet est distribué sous la licence MIT. Voir le fichier `LICENSE` pour plus de détails.