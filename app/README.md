# The Living Trail

The proposed website and Trail Studio CMS for the Iditarod Historic Trail Alliance. Built in Rust with Leptos 0.8, Axum server rendering, and WebAssembly hydration. No Sites dependency.

## Live preview

https://iditarod-review-1829.eastus2.cloudapp.azure.com/

Staff CMS: https://iditarod-review-1829.eastus2.cloudapp.azure.com/admin

Guided walkthrough: https://iditarod-review-1829.eastus2.cloudapp.azure.com/preview/

The editable Rust source is tracked in the repository under `app/`. The repository root also retains the earlier static GitHub Pages export.

## Working CMS

The project includes **Trail Studio**, a native Rust/Leptos CMS backed by SQLite with persistent content, private drafts, live previews, scheduling, document replacement, media, roles, and revision recovery. See [CMS setup and operations](cms/README.md) for local startup, staff login, backups, and integration boundaries. The full review system runs on Azure; [deployment operations](../deploy/README.md) describe hosting, backups, and costs. The earlier GitHub Pages link remains a static proposal demo.

## Motion and interactive journey

The homepage leads into a six-place geographic field guide: Seward, Crow Pass, Knik, Iditarod, Unalakleet and Nome. The custom Alaska Albers map combines the BLM main historic alignment with Natural Earth coastlines, rivers and graticule. Nearby Southcentral places expand into a regional view. Keyboard controls, a mobile list/map switch, authentic photography, sourced history and seasonal ideas make every place browsable. This is an orientation map, not navigation or current conditions. Source notes and photograph attribution are available at `/credits`.

The hero and explorer share a Winter/Summer selection, including the published experience filters on `/explore`. Links such as `/explore?place=crow-pass&season=summer#trail-atlas` restore the selected place and season. Default place content lives in `public/maps/explorer-places.json`; editors can override it through the place editor's **Trail explorer** section. Geographic coordinates and planning guidance stay fixed. The reproducible basemap builder and provenance are alongside the JSON.

The hero's Pause motion button stops ambient motion and removes animated transitions while keeping every control functional. The operating system's reduced-motion setting disables animation automatically. Server-rendered content remains readable without JavaScript. Intersection observers are owned and disconnected by their Leptos components, including when navigating away. No animation framework or video download is required; see `src/motion.rs` and the motion section of `style/main.css`.

## Local development

Requirements: Rust stable, the `wasm32-unknown-unknown` target, and `cargo-leptos` 0.3.9.

```powershell
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked --version 0.3.9
cargo leptos watch
```

Open http://localhost:3107. Enable the full CMS with `./scripts/start-cms.ps1 -Build`. Public compositions live in `src/editorial.rs` and `src/editorial_pages.rs`; CMS editing lives in `src/studio.rs` and `src/studio_photos.rs`. Editorial styles in `public/design/` extend `style/main.css`. Credited assets and source manifests live in `public/`.

## Build and host

```powershell
cargo leptos build --release
```

The server executable is `target/release/living-trail.exe` on Windows. Deploy the corresponding Linux binary on a Linux host, with `target/site`, and set `LEPTOS_OUTPUT_NAME=living-trail`, `LEPTOS_SITE_ROOT=target/site`, and the required `LEPTOS_SITE_ADDR`. `PORT` is also supported. Production host selection must account for the RFP's US hosting and data requirements.

Use `deploy/upgrade-review.sh` for the existing Azure instance. It backs up SQLite, preserves content, uploads and accounts, activates a matching server/browser release, and rolls back a failed application health check. Never deploy a development database over the review database. Azure is the current full-system review environment.

The earlier static export workflow is retained for reference. For a deployment subdirectory, set `TRAIL_BASE_PATH` before building both server and browser bundles:

```powershell
$env:TRAIL_BASE_PATH='/iditarod-living-trail'
cargo leptos build --release
# Run the resulting server on port 3109, then:
python scripts/export.py --origin http://127.0.0.1:3109 --base /iditarod-living-trail
python scripts/serve_preview.py
```

Upload the contents of `dist/` to that deployment path. Test the exported build before publishing. Clear `TRAIL_BASE_PATH` before a root-domain build. `Cargo.lock` and `package-lock.json` pin the tested dependencies.

## Quality checks

```powershell
cargo test --locked --features ssr --lib
cargo check --locked --target wasm32-unknown-unknown --no-default-features --features hydrate
npm ci
$env:EDITORIAL_URL='http://localhost:3107'
npm test
```

The public browser check uses Chrome and covers 11 routes at desktop, tablet, and phone widths, image loading, overflow, axe audits, resource search, map chapters, seasonal controls, and mobile navigation. It can inspect the Azure URL without changing content. Reports go to `artifacts/editorial`, or `EDITORIAL_ARTIFACT_DIR` if set. Automated checks do not certify full WCAG conformance.

Run `node tests/trail-explorer.cjs` with the same `EDITORIAL_URL` to check all six places, shared seasonal state, regional pins, deep links, keyboard controls and mobile list/map browsing. This suite is read-only and writes screenshots and its report to `artifacts/explorer` unless `EDITORIAL_ARTIFACT_DIR` is set.

For editor checks, start an **isolated** instance with `scripts/start-native-test.ps1`. Set `EDITORIAL_URL` to `http://localhost:3110`, `EDITORIAL_WRITE_TESTS` to `1`, and provide `TRAIL_TEST_LOGIN` and `TRAIL_TEST_PASSWORD` privately for that test account. Run `npm run test:editorial:cms`. The check refuses non-local hosts. It creates and archives a test story, verifies photo metadata, crops, preview, publication, private drafts and revisions, and restores the homepage draft after checking featured-story selection. The older proposal suite remains available as `test:legacy`; `test:cms` retains the original native API and role suite.

## Review boundaries

- Public store, donation, membership, volunteer and newsletter destinations are optional HTTPS links in **Connections**. They remain inactive on Azure pending approval. The site does not simulate payments, orders, or signup success.
- Staff service shortcuts remain inside the authenticated workspace. No payment provider, donor system, or mailing account has been activated.
- Trail Studio at `/admin` uses persistent SQLite. The `/preview` page introduces the actual site and publishing workflow.
- Historical copy and brand treatments are proposed editorial content pending Alliance review.
- Visitor cards link to original official resources; they are not live conditions or route instructions.
- The trail explorer uses sourced geographic data and is labeled as an orientation map.
- Photo credits and links are at `/credits`; font licenses are included in `public/fonts/`.

Approved content migration, production service destinations, privacy decisions, manual accessibility review, staff training, support arrangements, and launch acceptance remain part of the handoff. The review domain remains excluded from search indexing.
