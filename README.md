# The Living Trail

Rust + Leptos website and Trail Studio CMS for the Iditarod Historic Trail Alliance review.

The complete application source is in [`app/`](app/). The existing GitHub Pages export remains available at https://wolfbreed1829.github.io/iditarod-living-trail/. A full Azure review deployment is being prepared; it runs the website and authenticated CMS together, with persistent SQLite storage.

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
- Azure Linux installation, HTTPS proxy and nightly review backups in [`deploy/`](deploy/).

See [CMS operations](app/cms/README.md), [application development](app/README.md), and [Azure deployment](deploy/README.md).

## Checks

```sh
cd app
cargo test --features ssr --lib
cargo check --target wasm32-unknown-unknown --no-default-features --features hydrate
```

The browser suites cover the public site and native CMS (`npm test`, `npm run test:cms` with their corresponding local servers). CI checks the Rust storage/authentication code and compiles the WASM application.

This is an independent review environment using demonstration content. Payments, donor/member records and email services are not connected; their links remain external shortcuts. The RFP/FAQ, local credentials, private data, backups and development logs are excluded from the repository.
