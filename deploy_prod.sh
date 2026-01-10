#!/bin/bash
set -euo pipefail

# Configuration
SSH_KEY="/Users/shafeeq/Documents/01-New-Job/Prep/ai-serv/lul-mul-tul.pem"
EC2_USER="ubuntu"
EC2_HOST="ec2-3-7-70-60.ap-south-1.compute.amazonaws.com"
EC2_DEST="/var/www/axiom" 
FRONTEND_DEST="/var/www/axiom-ccp.shafeeq.dev"
SSH_CMD="ssh -i \"$SSH_KEY\""

echo "=========================================================="
echo "🚀 Syncing Axiom System Source to EC2 (Local Task) 🚀"
echo "=========================================================="

# 1. Transfer Source Code via Rsync + SSH
echo "[1/2] Transferring project source files to EC2 ($EC2_HOST)..."

# Ensure remote directories exist
eval "$SSH_CMD ${EC2_USER}@${EC2_HOST} 'sudo mkdir -p ${EC2_DEST}/src && sudo mkdir -p ${EC2_DEST}/bin && sudo chown -R ${EC2_USER}:${EC2_USER} ${EC2_DEST}'"

# Sync deploy_cloud.sh
echo "  -> Syncing deploy_cloud.sh..."
rsync -vhz --progress -e "$SSH_CMD" ./deploy_cloud.sh ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/src/deploy_cloud.sh

# Sync Full Source Code (Excluding targets, node_modules, dist, and logs)
echo "  -> Syncing source code..."
rsync -avhz --progress --exclude 'target/' --exclude '.logs/' --exclude 'node_modules/' --exclude 'dist/' --exclude '.git/' -e "$SSH_CMD" ./axiom-ccp ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/src/
rsync -avhz --progress --exclude 'target/' --exclude '.logs/' --exclude '.git/' -e "$SSH_CMD" ./axiom-shell ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/src/
rsync -avhz --progress --exclude 'target/' --exclude '.logs/' --exclude '.git/' -e "$SSH_CMD" ./axiom-cli ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/src/
rsync -avhz --progress --exclude 'target/' --exclude '.logs/' --exclude '.git/' -e "$SSH_CMD" ./axiom-sdk ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/src/

# Sync Service Templates
echo "  -> Syncing docs/ec2-services..."
rsync -avhz --progress -e "$SSH_CMD" ./docs/ec2-services ${EC2_USER}@${EC2_HOST}:${EC2_DEST}/docs/

# 2. Trigger remote deployment script
echo "=========================================================="
echo "🚀 Triggering Remote Cloud Build and Restart... 🚀"
echo "=========================================================="
eval "$SSH_CMD ${EC2_USER}@${EC2_HOST} 'chmod +x ${EC2_DEST}/src/deploy_cloud.sh && ${EC2_DEST}/src/deploy_cloud.sh'"

echo "=========================================================="
echo "✅ Local Sync Complete! "
echo "=========================================================="
