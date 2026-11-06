# The Living Trail

Rust + Leptos website and Trail Studio CMS for the Iditarod Historic Trail Alliance review.

The complete application source is in [`app/`](app/).

- [Live website](https://iditarod-review-1829.eastus2.cloudapp.azure.com/)
- [Trail Studio CMS](https://iditarod-review-1829.eastus2.cloudapp.azure.com/admin)

The Azure review instance runs the website and authenticated CMS together, with HTTPS, persistent SQLite storage, and nightly local backups. Request reviewer credentials from the project administrator; passwords are not stored in GitHub. The earlier [GitHub Pages export](https://wolfbreed1829.github.io/iditarod-living-trail/) remains a static design preview.

## Run locally

```powershell
cd app
./scripts/start-cms.ps1 -Build
```

Open http://localhost:3107 and http://localhost:3107/admin. A clean installation generates its initial administrator credentials in the ignored `app/.cms/rust/credentials.env` file. No credentials or live database are stored in this repository.

## What is included

- Native Leptos staff dashboard, autosave, private drafts, live previews and scheduled publishing.
- Pages, stories, places, documents, events and giving content.
- Revision recovery, permanent document download links, image/PDF uploads and staff roles.
- Axum authentication, Argon2id password storage, CSRF/origin checks and version-conflict detection.
- SQLite storage and backup scripts.
- Destination guides with planning notes, photo sequences and geographic locator maps; magazine stories and a sourced Alliance history.
- A searchable collection of 35 licensed photographs with captions, credits, source records and responsive image sizes.
- Azure Linux installation, HTTPS proxy and nightly review backups in [`deploy/`](deploy/).

See [CMS operations](app/cms/README.md), [application development](app/README.md), and [Azure deployment](deploy/README.md).

## Checks

```sh
cd app
cargo test --features ssr --lib
cargo check --target wasm32-unknown-unknown --no-default-features --features hydrate
```

The browser suites cover the public site and native CMS (`npm test`, `npm run test:cms` with their corresponding local servers). CI checks the Rust storage/authentication code and compiles the WASM application.

This is an independent review environment with proposed editorial content for Alliance review. Payments, donor/member records and email services are not connected; public destinations remain configurable in the CMS. The RFP/FAQ, local credentials, private data, backups and development logs are excluded from the repository.
