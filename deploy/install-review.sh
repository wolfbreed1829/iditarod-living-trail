#!/usr/bin/env bash
# Install a prebuilt release and an optional private SQLite/media snapshot.
# Usage: sudo bash install-review.sh /tmp/release.tar.gz /tmp/data.tar.gz hostname
set -euo pipefail
release_archive=$(realpath "$1")
data_archive=$(realpath "$2")
hostname="$3"
[[ "$hostname" =~ ^[a-z0-9.-]+$ ]] || { echo 'Invalid hostname'; exit 1; }
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq ca-certificates python3
id livingtrail >/dev/null 2>&1 || useradd --system --home /var/lib/living-trail --shell /usr/sbin/nologin livingtrail
id caddy >/dev/null 2>&1 || useradd --system --home /var/lib/caddy --shell /usr/sbin/nologin caddy
install -d -m 0755 /opt/living-trail/releases /etc/living-trail /etc/caddy
install -d -m 0700 -o livingtrail -g livingtrail /var/lib/living-trail /var/backups/living-trail
install -d -m 0700 -o caddy -g caddy /var/lib/caddy
release_dir="/opt/living-trail/releases/$(date -u +%Y%m%dT%H%M%SZ)"
install -d "$release_dir"
tar -xzf "$release_archive" -C "$release_dir" --no-same-owner
chmod 0755 "$release_dir/living-trail" "$release_dir/caddy"
if [[ ! -f /var/lib/living-trail/trail.sqlite3 ]]; then
  tar -xzf "$data_archive" -C /var/lib/living-trail --no-same-owner
  chown -R livingtrail:livingtrail /var/lib/living-trail
  chmod 0700 /var/lib/living-trail
  chmod 0600 /var/lib/living-trail/trail.sqlite3
fi
ln -sfn "$release_dir" /opt/living-trail/current
cat > /etc/living-trail/server.env <<EOF
TRAIL_CMS_ENABLED=1
TRAIL_TRUST_PROXY=1
TRAIL_DATA_DIR=/var/lib/living-trail
TRAIL_ORIGIN=https://$hostname
LEPTOS_OUTPUT_NAME=living-trail
LEPTOS_SITE_ROOT=/opt/living-trail/current/site
LEPTOS_SITE_ADDR=127.0.0.1:3107
PORT=3107
EOF
chmod 0600 /etc/living-trail/server.env
cat > /etc/systemd/system/living-trail.service <<'EOF'
[Unit]
Description=The Living Trail website and CMS
After=network-online.target
Wants=network-online.target
[Service]
User=livingtrail
Group=livingtrail
WorkingDirectory=/opt/living-trail/current
EnvironmentFile=/etc/living-trail/server.env
ExecStart=/opt/living-trail/current/living-trail
Restart=on-failure
RestartSec=3
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/living-trail
UMask=0077
[Install]
WantedBy=multi-user.target
EOF
cat > /etc/caddy/Caddyfile <<EOF
$hostname {
  encode zstd gzip
  header X-Robots-Tag "noindex, nofollow"
  header Referrer-Policy "no-referrer"
  reverse_proxy 127.0.0.1:3107
}
EOF
cat > /etc/systemd/system/caddy.service <<'EOF'
[Unit]
Description=HTTPS for the Living Trail review site
After=network-online.target living-trail.service
Wants=network-online.target
[Service]
User=caddy
Group=caddy
Environment=HOME=/var/lib/caddy
ExecStart=/opt/living-trail/current/caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
ExecReload=/opt/living-trail/current/caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
Restart=on-failure
RestartSec=3
AmbientCapabilities=CAP_NET_BIND_SERVICE
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/caddy
UMask=0077
[Install]
WantedBy=multi-user.target
EOF
install -m 0755 "$release_dir/backup-review.py" /usr/local/bin/backup-living-trail
cat > /etc/systemd/system/living-trail-backup.service <<'EOF'
[Unit]
Description=Consistent SQLite and media backup
[Service]
Type=oneshot
User=livingtrail
Group=livingtrail
ExecStart=/usr/local/bin/backup-living-trail
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/backups/living-trail
EOF
cat > /etc/systemd/system/living-trail-backup.timer <<'EOF'
[Unit]
Description=Nightly Living Trail review backup
[Timer]
OnCalendar=*-*-* 03:00:00 UTC
Persistent=true
RandomizedDelaySec=300
[Install]
WantedBy=timers.target
EOF
systemctl daemon-reload
systemctl enable living-trail caddy living-trail-backup.timer
systemctl restart living-trail caddy
systemctl start living-trail-backup.timer living-trail-backup.service
systemctl is-active living-trail caddy
echo "Review site: https://$hostname"
