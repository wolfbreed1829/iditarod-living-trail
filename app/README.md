# The Living Trail

Independent, responsive design preview for the Iditarod Historic Trail Alliance. Built in Rust with Leptos 0.8, Axum server rendering, and WebAssembly hydration. No Sites dependency.

## Live preview

https://wolfbreed1829.github.io/iditarod-living-trail/

Guided walkthrough: https://wolfbreed1829.github.io/iditarod-living-trail/preview/

The editable Rust source is tracked in the repository under `app/`. The repository root also retains the earlier static GitHub Pages export.

## Working CMS

The project includes **Trail Studio**, a native Rust/Leptos CMS backed by SQLite with persistent content, private drafts, live previews, scheduling, document replacement, media, roles, and revision recovery. See [CMS setup and operations](cms/README.md) for local startup, staff login, backups, and integration boundaries. The CMS is local; the existing GitHub Pages link remains the static proposal demo.

## Motion and interactive journey

The homepage uses a gently moving landscape, drifting winter snow, a staggered headline entrance, and an animated Winter/Summer selector with image crossfades. Scroll reveals introduce the sections and draw the illustrated route. Four keyboard-accessible chapter buttons update its progress, place markers, story, and destination link through Leptos signals.

The hero's Pause motion button stops ambient motion and removes animated transitions while keeping every control functional. The operating system's reduced-motion setting disables animation automatically. Server-rendered content remains readable without JavaScript. Intersection observers are owned and disconnected by their Leptos components, including when navigating away. No animation framework or video download is required; see `src/motion.rs` and the motion section of `style/main.css`.

## Local development

Requirements: Rust stable, the `wasm32-unknown-unknown` target, and `cargo-leptos` 0.3.9.

```powershell
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked --version 0.3.9
cargo leptos watch
```

Open http://localhost:3107. Content and layouts live in `src/`; styling lives in `style/main.css`; credited, optimized assets live in `public/`.

## Build and host

```powershell
cargo leptos build --release
```

The server executable is `target/release/living-trail.exe` on Windows. Deploy the corresponding Linux binary on a Linux host, with `target/site`, and set `LEPTOS_OUTPUT_NAME=living-trail`, `LEPTOS_SITE_ROOT=target/site`, and the required `LEPTOS_SITE_ADDR`. `PORT` is also supported. Production host selection must account for the RFP's US hosting and data requirements.

The bid preview can be exported to ordinary static hosting. Pages are rendered by Leptos before export and remain interactive through the same compiled Rust/WASM application. This avoids a sleeping server or a temporary tunnel. The demo has no server-dependent business transactions.

For a deployment subdirectory, set `TRAIL_BASE_PATH` before building both server and browser bundles:

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
npm ci
npm test
```

Tests use installed Google Chrome and cover all 15 routes at 360, 768, and 1440 pixels; photo loading; browser errors; the mobile menu; seasonal crossfades; motion pause; reduced motion; keyboard chapter selection; readable content without JavaScript; combined filters; resource search; the timeline; gift and membership previews; cart totals and route persistence; checkout; and editor behavior. Automated axe checks cover core pages. Automated checks do not certify full WCAG conformance.

For a different preview origin, set `DEMO_URL`. For a subdirectory deployment, also set `TRAIL_BASE_PATH` for the tests.

## Preview boundaries

- Forms remain in the browser. They do not save or send personal data.
- Giving, memberships, merchandise, prices, and checkout are clearly labeled concepts. No payments or real orders occur.
- The small editor on `/preview` remains illustrative. The native Trail Studio dashboard at `/admin` uses SQLite in the same Rust process when `TRAIL_CMS_ENABLED=1`.
- Historical copy and brand treatments are proposed editorial content pending Alliance review.
- Visitor cards link to original official resources; they are not live conditions or route instructions.
- The route illustration is decorative and clearly labeled, not GIS or a navigation map.
- Photo credits and links are at `/credits`; font licenses are included in `public/fonts/`.

Production work still includes final CMS/platform selection, approved content migration, actual Square/CRM/email integrations, privacy decisions, manual accessibility review, training, and support arrangements. The RFP and FAQ require these to be scoped separately from this demonstration.
