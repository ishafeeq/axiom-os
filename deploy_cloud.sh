#!/bin/bash
set -euo pipefail

echo "=========================================================="
echo "☁️ Executing Cloud Task: Compiling and Restarting Axiom ☁️"
echo "=========================================================="

export PATH="$HOME/.cargo/bin:$PATH"

# 1. Compile Frontend SPA
echo "[1/4] Compiling frontend SPA on Linux..."
cd /var/www/axiom/src/axiom-ccp/axiom-ccp-frontend
npm install
npm run build
rsync -avhz --delete dist/ /var/www/axiom-ccp.shafeeq.dev/dist/

# 1. Stop Services for Update
echo "[1/4] Stopping services for update..."
sudo systemctl stop axiom-ccp-backend || true
sudo systemctl stop axiom-shell || true

# 2. Compile Backend natively on Linux
echo "[2/4] Compiling axiom-ccp-backend..."
cd /var/www/axiom/src/axiom-ccp/axiom-ccp-backend
cargo build --release

# 3. Compile Shell natively on Linux
echo "[3/4] Compiling axiom-shell..."
cd /var/www/axiom/src/axiom-shell
cargo build --release

# 4. Compile Axiom CLI natively on Linux
echo "[4/4] Compiling axiom-cli (ax)..."
cd /var/www/axiom/src/axiom-cli
cargo build --release
sudo rm -f /usr/local/bin/ax
sudo cp target/release/ax /usr/local/bin/ax
sudo chmod +x /usr/local/bin/ax

# 5. Update systemd files and Deploy
echo "[5/4] Updating systemd files and deploying binaries..."
sudo cp /var/www/axiom/docs/ec2-services/axiom-ccp-backend.service /etc/systemd/system/
sudo cp /var/www/axiom/docs/ec2-services/axiom-shell.service /etc/systemd/system/
sudo systemctl daemon-reload

# 4. Deploy Binaries, Permissions and Restart
echo "[4/4] Deploying binaries and restarting..."
cp /var/www/axiom/src/axiom-ccp/axiom-ccp-backend/target/release/axiom-ccp-backend /var/www/axiom/bin/
cp /var/www/axiom/src/axiom-shell/target/release/axiom-shell /var/www/axiom/bin/

sudo chown ubuntu:ubuntu /var/www/axiom/bin/axiom-ccp-backend
sudo chown ubuntu:ubuntu /var/www/axiom/bin/axiom-shell
sudo chmod +x /var/www/axiom/bin/axiom-ccp-backend
sudo chmod +x /var/www/axiom/bin/axiom-shell

sudo systemctl start axiom-ccp-backend
sudo systemctl start axiom-shell

# Check status of services
echo "-> Checking service status..."
sudo systemctl is-active --quiet axiom-ccp-backend && echo "✅ axiom-ccp-backend is ONLINE" || echo "❌ axiom-ccp-backend failed to start"
sudo systemctl is-active --quiet axiom-shell && echo "✅ axiom-shell is ONLINE" || echo "❌ axiom-shell failed to start"

echo "Cloud deployment steps finished successfully."
