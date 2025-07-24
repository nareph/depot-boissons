# Dépôt de Boissons - Système de Gestion

![Screenshot de l'application](/screenshots/dashboard.png) 

**Dépôt de Boissons** est une application de bureau moderne conçue pour la gestion complète d'un dépôt de boissons. Développée en **Rust** avec l'interface utilisateur **Slint**, elle offre une solution robuste, performante et multi-plateforme (Windows, macOS, Linux) pour gérer les stocks, les ventes, les clients et les utilisateurs.

L'application est pensée pour fonctionner **offline**, avec toutes les données stockées localement dans une base de données SQLite, garantissant ainsi rapidité, simplicité de déploiement et confidentialité.

## ✨ Fonctionnalités Clés

- **Tableau de Bord Intuitif** : Visualisez en temps réel le chiffre d'affaires du jour, le nombre de ventes et les produits à stock faible.
- **Gestion des Produits (SKU)** : Chaque produit est géré comme une unité de vente unique avec son propre stock et prix.
- **Reporting Comptable Avancé** :
  - Analyse des ventes par période (7j/30j/année)
  - Export PDF et Excel des données financières
  - Top 5 des produits les plus vendus
- **Import en Lot de Produits** :
  - Validation des fichiers CSV avant import
  - Suivi de progression en temps réel
- **Système de Vente Complet** :
  - Interface POS avec gestion de panier
  - Validation des stocks en direct
  - Impression de tickets de caisse
- **Gestion des Utilisateurs** :
  - Rôles Admin/Utilisateur
  - Changement de mot de passe obligatoire à la première connexion
- **Sécurité Renforcée** :
  - Mots de passe hachés avec bcrypt
  - Toutes les données stockées localement

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
    -   Cette commande va créer le fichier de base de données SQLite et toutes les tables nécessaires et un utilisateur admin par défaut.
    ```bash
    diesel migration run
    ```

4.  **Peupler la base avec des données de test (Optionnel mais recommandé)** :
    -   L'application contient un "seeder" pour remplir la base avec des produits et ventes de démonstration.
    -   Lancez l'application avec l'argument `--seed`. Le double tiret (`--`) est important pour passer l'argument à votre programme et non à Cargo.
    ```bash
    cargo run -- --seed
    ```

5.  **Lancer l'application normalement** :
    -   Pour les lancements suivants, utilisez simplement `cargo run`.
    ```bash
    cargo run
    ```

## 🔐 Première Connexion

### Identifiants Admin par Défaut
Lors de la première installation, un utilisateur admin est automatiquement créé avec les credentials suivants :

- **Nom d'utilisateur**: `admin`  
- **Mot de passe**: `admin`  

**Important**:  
🔒 Pour des raisons de sécurité, le système **exigera** le changement de ce mot de passe lors de la première connexion.  
⚠️ Changez-le immédiatement par un mot de passe fort et gardez-le secret !

---

### Processus de Première Connexion
1. Lancez l'application
2. Entrez les identifiants ci-dessus
3. Suivez les instructions pour :
   - Définir un nouveau mot de passe sécurisé
4. Vous serez redirigé vers le tableau de bord

> 💡 Conseil : Après la première connexion, créez des comptes supplémentaires pour vos collaborateurs via l'interface Admin.

## 📊 Reporting Comptable

La fonctionnalité de reporting offre des outils puissants pour l'analyse des ventes et la comptabilité.

### Fonctionnalités

- **Indicateurs Clés** :
  - Chiffre d'affaires total
  - Nombre de transactions
  - Panier moyen
- **Périodes Personnalisables** :
  - 7 derniers jours
  - 30 derniers jours
  - Année en cours
- **Exports Professionnels** :
  - **PDF** : Rapport structuré prêt à imprimer
  - **Excel** : Données brutes pour analyse approfondie

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
