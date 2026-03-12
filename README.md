# InterruptLog

> Première brique de l'écosystème **Omistone** — outils métier pour dessinateurs projeteurs en menuiserie tertiaire.

Mesure et documente les interruptions non planifiées du poste de travail pour en avoir des données terrain réelles.

---

## Fonctionnement

1. Cliquer sur le nom d'un collaborateur → le chrono démarre
2. Le fichier DWG/DXF actif dans ZWCAD est détecté automatiquement
3. Cliquer **Terminer** → l'interruption est enregistrée (durée, clics souris, fichier CAD)
4. Le journal du jour et les stats se mettent à jour en temps réel
5. Export CSV vers `Documents/InterruptLog/`

---

## Stack technique

| Composant | Technologie |
|---|---|
| App desktop | [Tauri v2](https://tauri.app) |
| Backend | Rust + SQLite (rusqlite, bundled) |
| Frontend | HTML / CSS / JS vanilla |
| Persistance | Local uniquement — zéro serveur |

---

## Structure du projet

```
INTERRUPTLOG/
├── src/                    Frontend (HTML, CSS, JS)
│   ├── index.html
│   ├── style.css
│   └── main.js
└── src-tauri/              Backend Rust
    ├── src/
    │   ├── main.rs         Point d'entrée + setup DB
    │   ├── db.rs           Schéma SQLite
    │   └── commands.rs     Commandes Tauri exposées au JS
    ├── Cargo.toml
    └── tauri.conf.json
```

---

## Base de données

La DB SQLite est créée automatiquement dans :
```
%APPDATA%\com.omistone.interruptlog\interruptlog.db
```

### Schéma

```sql
people (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  role        TEXT NOT NULL DEFAULT '',
  created_at  TEXT NOT NULL
)

interruptions (
  id               INTEGER PRIMARY KEY,
  person_id        INTEGER,
  person_name      TEXT NOT NULL,   -- dénormalisé (historique préservé)
  start_time       TEXT NOT NULL,
  end_time         TEXT,
  duration_seconds INTEGER,
  mouse_clicks     INTEGER,
  active_window    TEXT,            -- fichier DWG/DXF détecté
  notes            TEXT,
  created_at       TEXT NOT NULL
)
```

---

## Installation (développement)

### Prérequis

- [Rust](https://rustup.rs/) avec toolchain `stable-x86_64-pc-windows-gnu`
- [Node.js](https://nodejs.org/) LTS
- MinGW-w64 (installé via winget : `BrechtSanders.WinLibs.POSIX.UCRT`)
- WebView2 Runtime (inclus dans Windows 10/11 récent — **bundlé automatiquement dans le setup.exe** pour les machines qui ne l'ont pas)

### Lancer

```bash
npm install
npm run dev
```

La première compilation prend 3–5 minutes (Tauri + SQLite bundled).

### Builder le setup .exe

```bash
npm run build
```

Génère un installateur NSIS dans :
```
src-tauri/target/release/bundle/nsis/InterruptLog_0.1.0_x64-setup.exe
```

---

## Données capturées par session

| Champ | Description |
|---|---|
| `person_name` | Collaborateur sélectionné |
| `start_time` | Heure de début |
| `duration_seconds` | Durée totale |
| `mouse_clicks` | Nombre de clics souris pendant l'interruption |
| `active_window` | Fichier DWG/DXF actif dans ZWCAD au moment du stop |

---

## Roadmap Omistone

- [ ] Détection automatique du fichier CAD actif (ZWCAD API / COM)
- [ ] Statistiques hebdomadaires / mensuelles
- [ ] Catégorisation des interruptions (urgence, question, livraison…)
- [ ] Intégration avec les autres briques Omistone
