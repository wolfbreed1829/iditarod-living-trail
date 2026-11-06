#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_meta::provide_meta_context;
    use leptos_router::components::Router as BrowserRouter;
    use living_trail::{app::*, components::Cart, paths::BASE_PATH};
    let conf = get_configuration(None).expect("Leptos configuration");
    let mut options = conf.leptos_options;
    if let Ok(port) = std::env::var("PORT") {
        options
            .site_addr
            .set_port(port.parse().expect("Valid PORT"));
    }
    let addr = options.site_addr;
    if std::env::var("TRAIL_CMS_ENABLED").as_deref() == Ok("1") {
        let dir = std::env::var("TRAIL_DATA_DIR").unwrap_or(".cms/rust".into());
        let origin =
            std::env::var("TRAIL_ORIGIN").unwrap_or(format!("http://localhost:{}", addr.port()));
        let store = std::sync::Arc::new(
            living_trail::studio_store::Store::open(dir.into(), origin)
                .expect("Open Trail Studio database"),
        );
        let _ = living_trail::studio_store::STORE.set(store.clone());
        tokio::spawn(async move {
            loop {
                if let Err(e) = store.tick() {
                    eprintln!("Scheduled publishing: {e}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }
    let app = Router::new()
        // Explicit assets take precedence over the CMS's /:slug page route.
        .route(
            "/cms.css",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../public/cms.css"),
                )
            }),
        )
        .route(
            "/studio.css",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../public/studio.css"),
                )
            }),
        )
        .route(
            "/studio-native.css",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../public/studio-native.css"),
                )
            }),
        )
        .route(
            "/favicon.svg",
            axum::routing::get(|| async {
                (
                    [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("../public/favicon.svg"),
                )
            }),
        )
        .merge(living_trail::studio_api::routes())
        .leptos_routes(
            &options,
            generate_route_list(|| {
                // Route enumeration must be synchronous; content loading is asynchronous.
                provide_meta_context();
                provide_context(Cart(RwSignal::new([0u32; 3])));
                view! {<BrowserRouter base=BASE_PATH><SiteContent data=None/></BrowserRouter>}
            }),
            {
                let options = options.clone();
                move || shell(options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(options)
        .layer(axum::middleware::from_fn(
            |request: axum::extract::Request, next: axum::middleware::Next| async move {
                let mut response = next.run(request).await;
                response
                    .headers_mut()
                    .insert("X-Content-Type-Options", "nosniff".parse().unwrap());
                if std::env::var("TRAIL_CMS_ENABLED").as_deref() == Ok("1") {
                    response
                        .headers_mut()
                        .insert("Cache-Control", "no-store".parse().unwrap());
                    response
                        .headers_mut()
                        .insert("Referrer-Policy", "no-referrer".parse().unwrap());
                }
                response
            },
        ));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Bind server");
    println!(
        "The Living Trail is ready at http://localhost:{}",
        addr.port()
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Serve app");
}
#[cfg(not(feature = "ssr"))]
fn main() {}
