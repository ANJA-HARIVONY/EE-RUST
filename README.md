# EE Salon

Application de bureau pour la gestion d'un salon de beauté : agenda, clients, prestations, stock, employés et factures.

Le métier et la base SQLite sont en Rust. L'interface, inspirée d'un logiciel de messagerie (navigation à gauche, liste ou calendrier, volet de détail), est en Svelte.

## Prérequis

- [Rust](https://rustup.rs/) (édition stable, 1.85 ou plus récente)
- [Node.js](https://nodejs.org/) 20 ou plus récent
- Sur macOS, les outils de ligne de commande Xcode (`xcode-select --install`)

## Configuration

Les variables vivent dans `.env`, à la racine du projet. Un exemple est fourni dans `.env.example`.

| Variable | Rôle |
| --- | --- |
| `SALON_NAME` | Nom du salon |
| `SALON_ADDRESS` | Adresse |
| `SALON_PHONE` | Téléphone |
| `SALON_EMAIL` | E-mail |
| `SALON_SIRET` | SIRET, peut rester vide |
| `CURRENCY` | Devise ISO, par exemple `EUR` |
| `VAT_RATE_BPS` | TVA en points de base. `2000` = 20,00 % |
| `INVOICE_PREFIX` | Préfixe des factures, par exemple `FAC` → `FAC-2026-0001` |
| `DEFAULT_USER_FIRST_NAME` | Prénom de l'utilisateur créé au premier lancement |
| `DEFAULT_USER_LAST_NAME` | Nom de cet utilisateur |
| `DEFAULT_USER_EMAIL` | E-mail, sert aussi à le reconnaître comme session |
| `DEFAULT_USER_PHONE` | Téléphone, peut rester vide |
| `DEFAULT_USER_ROLE` | `admin`, `manager`, `receptionist` ou `stylist` |
| `DEFAULT_USER_COLOR` | Couleur agenda, format `#RRGGBB` |
| `DATABASE_FILENAME` | Nom du fichier SQLite |
| `AGENDA_OPEN_HOUR` | Heure d'ouverture de la grille |
| `AGENDA_CLOSE_HOUR` | Heure de fermeture de la grille |
| `AGENDA_SLOT_MINUTES` | `5`, `10`, `15`, `20`, `30` ou `60` |
| `LOCALE` | Affichage des dates et des montants, par exemple `fr-FR` |
| `LOW_STOCK_DEFAULT` | Seuil proposé à la création d'un produit |

Au premier lancement, le nom du salon, la TVA, la devise, le préfixe de facture et l'utilisateur par défaut sont copiés dans SQLite. Les lancer de nouveau ne les réécrit pas : l'écran Réglages modifie ensuite la base. Les horaires d'agenda, la locale, le seuil proposé et le nom du fichier sont relus dans `.env` à chaque démarrage.

Les prix sont stockés en centimes. Une facture émise fige le taux de TVA et les prix des lignes.

## Lancement

```bash
npm install
npm run tauri dev
```

Le fichier `.cargo/config.toml` pointe Cargo vers un miroir de crates.io, parce que l'index officiel ne répond pas depuis ce réseau. Supprimez-le si `index.crates.io` est accessible chez vous.

La base est créée automatiquement dans le dossier de données de l'application. Sur macOS :

```text
~/Library/Application Support/com.eesalon.app/salon.db
```

Le chemin exact est aussi affiché dans Réglages. Pour repartir d'une base vide, quittez l'application et supprimez ce fichier : le prochain lancement réapplique `.env`.

## Vérifier le métier sans ouvrir la fenêtre

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Le test couvre le premier lancement (réglages et utilisateur par défaut), le refus d'un chevauchement d'agenda, l'émission d'une facture (prestation + produit, TVA) et le retour du stock si la facture est annulée.

## Modules

- **Agenda** — vues jour (une colonne par employé), semaine et mois. Un créneau libre prépare un rendez-vous. Un employé ne peut pas avoir deux rendez-vous qui se chevauchent.
- **Clients** — fiches utilisées par l'agenda et les factures.
- **Services** — nom, durée, prix HT, description, activation.
- **Produits** — stock, entrées, sorties, ajustement, alerte sous le seuil. La vente passe par une facture.
- **Employés** — fiche, rôle, couleur d'agenda, activation. Il reste toujours au moins un administrateur actif.
- **Factures** — brouillon, import d'un rendez-vous, lignes produit, émission, paiement, annulation. L'émission décrémente le stock. L'annulation le restaure.
- **Réglages** — identité du salon, devise, TVA et préfixe, initialisés depuis `.env`.
