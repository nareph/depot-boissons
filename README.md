# Dépôt de Boissons - Système de Gestion

![Screenshot de l'application](/screenshots/dashboard.png) 

**Dépôt de Boissons** est une application de bureau moderne conçue pour la gestion complète d'un dépôt de boissons. Développée en **Rust** avec l'interface utilisateur **Slint**, elle offre une solution robuste, performante et multi-plateforme (Windows, macOS, Linux) pour gérer les stocks, les ventes, les clients et les utilisateurs.

L'application est pensée pour fonctionner **offline**, avec toutes les données stockées localement dans une base de données SQLite, garantissant ainsi rapidité, simplicité de déploiement et confidentialité.

## ✨ Fonctionnalités Clés

-   **Tableau de Bord Intuitif** : Visualisez en temps réel le chiffre d'affaires du jour, le nombre de ventes et les produits à stock faible.
-   **Gestion des Produits (SKU)** : Chaque produit est géré comme une unité de vente unique (ex: "Casier de 12 bouteilles 65cl"), avec son propre stock et son propre prix, reflétant la logique métier d'un dépôt.
-   **Import en Lot de Produits** : 
    -   Importation massive de produits via fichiers CSV.
    -   Template CSV téléchargeable avec les colonnes requises (Nom, Packaging, Stock, Prix).
    -   Validation préalable des données avec rapport d'erreurs détaillé.
    -   Suivi de progression en temps réel lors de l'import.
    -   Navigateur de fichiers intégré pour sélectionner facilement les fichiers CSV.
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
-   **Base de Données** : [**SQLite**](https://www.sqlite.org/) - Une base de données légère, embarquée et sans serveur, parfaite pour les applications desktop.
-   **ORM / Accès BDD** : [**Diesel**](https://diesel.rs/) - Un ORM et un Query Builder performant et sûr pour Rust.
-   **Interface Système** : [**rfd**](https://crates.io/crates/rfd) - Pour les boîtes de dialogue natives (sélection de fichiers, sauvegarde).

## 🚀 Démarrage Rapide

### Prérequis

1.  **Toolchain Rust** : Assurez-vous d'avoir Rust et Cargo installés. [Instructions d'installation](https://www.rust-lang.org/tools/install).
2.  **Diesel CLI** : Outil en ligne de commande pour gérer les migrations de la base de données.
    ```bash
    cargo install diesel_cli --no-default-features --features sqlite
    ```

### Installation et Lancement

1.  **Cloner le dépôt** :
    ```bash
    git clone https://github.com/nareph/depot-boissons.git
    cd depot-boissons
    ```

2.  **Configuration de l'Environnement** :
    -   Copiez le fichier `.env.example` en `.env`.
    -   Modifiez le fichier `.env` pour y mettre le chemin vers votre base de données SQLite :
        ```env
        DATABASE_URL=database.db
        ```

3.  **Appliquer les Migrations** :
    -   Cette commande va créer le fichier de base de données SQLite et toutes les tables nécessaires.
    ```bash
    diesel migration run
    ```

4.  **Peupler la base avec des données de test (Optionnel mais recommandé)** :
    -   L'application contient un "seeder" pour remplir la base avec des utilisateurs, produits et ventes de démonstration.
    -   Lancez l'application avec l'argument `--seed`. Le double tiret (`--`) est important pour passer l'argument à votre programme et non à Cargo.
    ```bash
    cargo run -- --seed
    ```
    -   **Identifiants par défaut** :
        -   Utilisateur : `Administrateur`
        -   Mot de passe : `admin123`
        *(Il vous sera demandé de changer ce mot de passe à la première connexion.)*

5.  **Lancer l'application normalement** :
    -   Pour les lancements suivants, utilisez simplement `cargo run`.
    ```bash
    cargo run
    ```

## 📊 Import de Produits en Lot

L'application supporte l'import massif de produits via des fichiers CSV, idéal pour migrer depuis un autre système ou ajouter de nombreux produits rapidement :

### Format CSV Requis

Le fichier CSV doit contenir les colonnes suivantes (dans l'ordre) :
- **Nom** : Nom du produit
- **Packaging** : Description du packaging (ex: "Casier de 12 bouteilles 65cl")
- **Stock** : Quantité en stock (nombre entier)
- **Prix** : Prix unitaire (nombre décimal, sans symbole monétaire)

### Processus d'Import

1. **Téléchargez le template** : L'application génère un fichier CSV d'exemple avec les bonnes colonnes
2. **Remplissez vos données** : Complétez le fichier avec vos produits
3. **Validez avant import** : L'outil de validation vérifie la syntaxe et signale les erreurs
4. **Importez** : Lancez l'import avec suivi de progression en temps réel

### Gestion des Erreurs

- Validation ligne par ligne avec messages d'erreur détaillés
- Les lignes valides sont importées même si certaines contiennent des erreurs
- Rapport complet d'import avec statistiques de réussite/échec

## 🤝 Contribuer

Les contributions sont les bienvenues ! Si vous souhaitez améliorer ce projet, veuillez forker le dépôt, créer une branche pour votre fonctionnalité (`git checkout -b feature/NomDeLaFonctionnalite`), commiter vos changements et ouvrir une Pull Request.

## 📜 Licence

Ce projet est distribué sous la licence MIT.