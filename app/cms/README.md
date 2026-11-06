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

An administrator can open **Connections** and add approved HTTPS destinations for the store, donations, memberships, newsletter signup, and volunteering. Saving updates the matching public journey. Without a destination, support journeys save a giving enquiry, membership enquiry, volunteer offer, or newsletter interest in the private inbox. The shop stays closed without its destination. A linked checkout or signup service handles its own final confirmation; opening a link never counts as a completed payment or subscription.

The separate **Service shortcuts** section holds staff dashboard links. Those addresses are excluded from the public content payload. Passwords and API keys do not belong in these fields. Payment processing and final subscription confirmation remain with the destination provider. The Azure review deployment leaves public destinations unconfigured until approval.

## Support requests and projects

**Support requests** is a private inbox for editors and administrators. Requests include contact permission and its text version, a reference, timestamp, area of interest and optional details. Change progress to **In progress** or **Closed**, and save a private follow-up note. These states never assert that a payment was made, a membership created, a volunteer booked or a subscription activated. No automatic emails are sent; staff must check the inbox and arrange any follow-up through an approved process.

Administrators can export a selected request type as CSV or permanently delete a request. Exports include every non-withdrawn request of that type, irrespective of search/status filters. Newsletter interest is not a verified subscriber list. Formula-like CSV values are escaped. Contributors cannot access the inbox or exports.

Visitors receive a private confirmation link only after a successful database write. Reloading that link retrieves the saved record; uncertain network responses can be retried without duplicate records. The receipt contains no contact details. Its secret link can withdraw the request and erase the active name, email, message, amount and staff note. Withdrawal cannot be reversed or exported. Staff deletion invalidates the confirmation. Daily database backups include requests and rotate to the latest 14 copies; exported files need their own handling. Public privacy details are at `/support/privacy`.

Requests live in `support_requests` in the existing SQLite database. Public POSTs require the configured exact Origin, explicit consent and validated fields. A honeypot, body-size limits and a persisted limit of 10 new requests per client per hour reduce abuse. Raw client addresses are not stored in the request record. Private mutations use the normal CMS session, CSRF and version checks. The database migration adds tables without changing existing content or accounts.

Under **Projects & giving**, fill **Project status**, **Project date or reporting period**, **Project source address**, **What support covered**, **How people could help next**, and **Support interest key**. Publishing a project status includes it in **Work supported**. Use one of `trail-care`, `history`, `education`, or `where-needed` for the interest key. Keep completed work, funding commitments and current openings distinct. The four initial records are sourced in `cms/support-projects.json`; they do not publish budgets or restricted fundraising commitments.

Consent version `support-2026-11-06` records permission for the site team to store details and contact the requester about the specific request; newsletter interest separately permits contact about future newsletter updates. Other request types do not opt people into mailing lists. Update the version when the consent wording or purpose changes.

## Photo presentation

The editor offers 35 licensed trail photographs, searchable by place, subject or photographer and filterable into six categories. Choosing one fills its description, dated caption and credit, with links to the original record and license. The public collection at `/credits` shares those records. Uploaded photos are also available; add their descriptions, captions and credits before publishing. Use **Horizontal focus** and **Vertical focus** to keep the important part in the crop. These controls change presentation only and leave the source image untouched. Stories, places, and the Alliance page also offer a taller feature photograph. Archival photographs retain their full composition. The homepage has independent summer focus controls.

Stories include **Photo sequence & map** controls for two supporting photographs and a geographic locator. Leave a selection at its editorial default, choose another collection image, or remove it. The collection supplies verified captions and credits. Even a one-paragraph story displays both selected images. All choices remain private until publication.

Place guides have optional **Visit notes**, **Season notes** and **Access notes**. Existing guides include sourced activities, planning information and supporting photography. New places use the same destination layout with their own CMS content and optional map connection. The map opens the interactive trail explorer.

### Editing a mapped place

In a place editor, **Trail explorer → Place on the map** connects that entry to one of six verified locations. Publishing uses the entry's introduction and photo in the homepage and Explore map. Add **Explorer history**, its source, a dated **Photo caption**, and separate **Summer ideas** / **Winter ideas** (one activity per line, up to six). **Visitor information address** provides the planning link. Blank history and activity fields retain the researched defaults. Coordinates and seasonal planning notes remain fixed.

Only one published or scheduled entry can own a location. Draft changes stay private. To return to the original field guide, clear the map connection and publish. Existing entries are not connected or rewritten automatically during an upgrade.

The same saved focus is used in the public feature image and content cards. The editor crop is a composition aid; **Preview** shows the actual responsive page. Changes autosave privately and only become public when published. The homepage featured story selection controls the lead story in the trail journal.
