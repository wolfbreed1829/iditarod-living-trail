# Azure review server

The review system runs the Rust/Leptos executable and Caddy on one small Ubuntu VM in a US Azure region. The website and `/admin` use the same HTTPS origin. SQLite and uploads live on its persistent managed OS disk, outside application releases.

## Review instance

- Website: https://iditarod-review-1829.eastus2.cloudapp.azure.com/
- CMS: https://iditarod-review-1829.eastus2.cloudapp.azure.com/admin
- Resource group: `iditarod-review-rg`
- VM: `iditarod-review`, East US 2, `Standard_F1als_v7` (1 vCPU, 2 GB RAM), Ubuntu 24.04.
- Storage: 32 GB Standard SSD managed OS disk, including SQLite and media.
- Network: HTTPS and HTTP public; SSH restricted to the deployment administrator's address; application port 3107 is loopback only.

The November 5, 2026 retail compute rate is $0.0605/hour (about $44.17 for 730 hours), plus managed disk, public IPv4, and any billable traffic. Actual billing depends on the subscription and runtime. The subscription could not allocate the cheaper burstable VM types at deployment time. Reviewer credentials are delivered privately and are absent from this repository.

## Release contents

Create a gzip tar archive containing `living-trail` (Linux x86-64), `caddy` (Linux x86-64), `site/` (the matching Leptos browser build), and `backup-review.py`. Build the server from `app/` with `cargo build --locked --release --bin living-trail --no-default-features --features ssr`. Build the browser assets with `cargo leptos build --release --frontend-only`. Leave `TRAIL_BASE_PATH` unset for this root-domain deployment.

The initial private data archive contains `trail.sqlite3`, `uploads/`, and `legacy-uploads/`. Use the SQLite backup API, remove session/preview rows from the snapshot, and transfer it over SSH. Never commit this archive, SSH keys, passwords, or runtime data. For a fresh review instance, the regular application setup can initialize demo content instead.

Copy both archives and `install-review.sh` to the VM, then run:

```sh
sudo bash install-review.sh /tmp/release.tar.gz /tmp/data.tar.gz review-hostname
```

The installer supplies a `_bg.wasm` alias for plain Cargo SSR builds so their hydration URL matches the cargo-leptos browser output. It does not overwrite an existing database. Later releases update the application symlink while retaining content. It expects a fresh, dedicated Ubuntu VM. Only ports 80 and 443 need public inbound access. Restrict SSH to the administrator's IP. The Rust application listens on loopback port 3107; it trusts forwarded client addresses only from the explicitly enabled local proxy.

## Operations

```sh
sudo systemctl status living-trail caddy
sudo journalctl -u living-trail --since '15 minutes ago'
sudo systemctl start living-trail-backup.service
sudo systemctl list-timers living-trail-backup.timer
```

Backups run nightly at approximately 03:00 UTC and keep the latest 14 archives in `/var/backups/living-trail`. These are on the same persistent VM disk; periodically download them to another location. They are not an off-site backup service. The deployment workstation also retains the initial snapshot.

To restore, stop `living-trail`, retain the current data as a rollback copy, extract a chosen backup into `/var/lib/living-trail`, restore ownership to `livingtrail:livingtrail`, and start the service. Accounts and passwords are those in the snapshot. Sessions and preview links are deliberately excluded.

Reviewer changes are separate from the local development database. The environment uses sample content; Square, donor/member and email services remain external shortcuts, with no transaction processing or automatic synchronization. Nothing sends invitation emails.

The VM, managed disk and public IP incur Azure charges until removed; deallocating the VM stops compute billing but retains disk and public-IP charges. The resource group is tagged `project=iditarod` and `purpose=review` so it can be identified later. Do not remove it while reviews are underway.
