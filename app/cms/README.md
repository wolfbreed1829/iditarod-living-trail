# Trail Studio ? Rust + Leptos

The public website and staff dashboard are one Rust application: Leptos 0.8 renders both interfaces, Axum serves the API, and SQLite stores content. The browser editor is compiled Rust/WASM. PHP, WordPress, MariaDB and Docker are not required.

## Run locally

```powershell
./scripts/start-cms.ps1 -Build
```

Open **http://localhost:3107/admin**. The public site is **http://localhost:3107**. Subsequent starts can omit `-Build`. `start-site.ps1` starts the same unified app.

The existing `trail-admin` login and password have been preserved. Initial credentials are in `.cms/rust/credentials.env` (ignored by version control). Passwords in SQLite use Argon2id. After changing a password in People & access, use that new password; the initial credential file is only used when creating an empty database. Keep that file private.

## Editing

- **Overview:** drafts, scheduled work, and shortcuts.
- **Pages:** homepage and other page content, photos, featured stories and places, search descriptions.
- **Stories / Places:** introductions, body text, photos and descriptions; place regions, seasons and activities.
- **Documents:** upload a PDF in Photos & files and paste its address into a document. The permanent `/api/cms/download/{id}` link resolves to the latest published file. Old same-host `/wp-json/trail/v1/download/{id}` paths remain aliases for migration.
- **Events / Giving:** event dates and locations, campaign descriptions and external buttons. Expired events leave the public listings.
- **Photos & files:** JPEG, PNG, WebP and PDF uploads, limited to 20 MB; photos are cropped for display with CSS. There is no destructive image crop editor.
- **Connections:** HTTPS shortcuts to Square, donor records and mailing services. No automatic integration or payment processing is connected.
- **People & access:** administrator-created accounts, roles, password reset, and access removal. No invitation emails are sent.

Drafts autosave after a short pause. A browser recovery copy protects pending changes if the connection drops. Conflicting versions are rejected instead of overwriting another editor's work. Wait for ?All changes saved? before leaving the browser. Preview opens the actual public template; a preview link is a frozen snapshot that expires after 30 minutes.

Publishing is explicit. Scheduling freezes the draft at the selected local time; later edits remain private. The Rust process checks schedules every 30 seconds, and public content reads also check due schedules. The process must be running for scheduled publication to happen on time. Previously published addresses stay permanent. Restoring a revision creates a draft; it does not immediately change the public website. The latest 75 published/discarded revisions are retained.

Contributors can edit and submit their own drafts. Editors can manage all content and publish. Administrators also manage accounts and connection settings. Removing an account revokes its sessions and transfers its content to the administrator. An administrator cannot remove their own account.

## Storage and backups

All live state is under `.cms/rust/`:

- `trail.sqlite3` plus SQLite WAL files: content, revisions, schedules, accounts, sessions, settings and media records.
- `uploads/`: immutable native uploads.
- `legacy-uploads/`: preserved imported files.
- `import.json`: original content export; only read when initializing an empty database.

```powershell
./scripts/backup-cms.ps1
```

This uses SQLite's online backup API, checks database integrity, and copies uploads into a timestamped `.cms/backups/rust-*` folder. It also exports readable content JSON. Backups omit active sessions and preview links. The in-app JSON export is a content export, not a complete database/media backup. Arrange regular off-machine backups before production use.

To restore: stop the Rust process, retain the current data directory as a rollback copy, then copy `trail.sqlite3`, `uploads/` and `legacy-uploads/` from the selected backup into the configured data directory. Start the app and sign in. Restored credentials are those stored in the backup. Verify content and downloads before replacing any off-machine backups.

## WordPress migration

The original 22 records, drafts, revisions, schedules, content IDs and administrator account were exported. A fresh original SQL/media backup is retained under `.cms/backups/`. Original Docker volumes are preserved for rollback. The one-time migration utility is `scripts/migrate-wordpress.cjs`; do not run it over an active native database. It deliberately stops if additional staff accounts require a password migration.

Legacy WordPress sources are archived under `.cms/legacy-wordpress/`, retaining their original folder layout. The three WordPress/MariaDB/scheduler containers are stopped; their volumes remain intact. To inspect the retired CMS, restore the archived `cms/plugin`, `cms/compose.yaml` and `cms/php.ini` paths into the project, then start that Compose configuration with `.cms/local.env`. This recovers access to the original CMS and its data; the current public-site code reads native SQLite. Runtime code never fetches content from WordPress.

## Deployment

Deploy the Rust binary, `target/site`, and a durable data directory on a US-hosted server. This SQLite implementation is for one application instance with a local persistent disk, not multiple replicas or ephemeral serverless storage. Run behind an HTTPS reverse proxy/service manager and set:

```
TRAIL_CMS_ENABLED=1
TRAIL_DATA_DIR=/var/lib/living-trail
TRAIL_ORIGIN=https://your-domain.example
LEPTOS_OUTPUT_NAME=living-trail
LEPTOS_SITE_ROOT=/opt/living-trail/site
LEPTOS_SITE_ADDR=127.0.0.1:3107
```

`TRAIL_ORIGIN` must exactly match the browser's origin. Mutations require that origin and a per-session CSRF token. Sessions use HttpOnly, SameSite=Strict cookies, with Secure enabled for HTTPS. Login attempts are limited by client address. Behind the local Caddy proxy, set `TRAIL_TRUST_PROXY=1`; forwarded addresses are accepted only from loopback peers. Bootstrap administrator environment values are required only for a new empty database. Limit filesystem access to the service account and administrators.

The review instance runs on [Azure](https://iditarod-review-1829.eastus2.cloudapp.azure.com/admin), with HTTPS, persistent SQLite storage, and nightly local backups. See [Azure operations](../../deploy/README.md). A custom production domain and actual Square/CRM/email integrations remain separate work. The GitHub Pages proposal is a standalone static demo.

## Verification

```powershell
cargo test --features ssr --lib studio_store::tests
./scripts/start-native-test.ps1
# In another terminal:
npx playwright test -c native.playwright.config.js
```

The browser suite uses an isolated `.cms/native-test` database on port 3110. Preserve or move that test directory before a fresh suite run because fixtures deliberately use stable names. The main content database is untouched by tests. Original public-demo tests use `npm test` against a server without `TRAIL_CMS_ENABLED` (set `DEMO_URL` to its address).

## Optional public service links

An administrator can open **Connections** and add approved HTTPS destinations for the store, donations, memberships, newsletter signup, and volunteering. Saving updates the matching public buttons. Leave a destination empty to keep that service inactive; the site never simulates a completed purchase, gift, or signup. No payment, membership, or mailing-list account is created by the CMS.

The separate **Service shortcuts** section holds staff dashboard links. Those addresses are excluded from the public content payload. Passwords and API keys do not belong in these fields. Payment processing and subscriber consent remain with the destination provider. The Azure review deployment deliberately leaves public destinations unconfigured until approval.
