#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Determine script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Default to user installation
INSTALL_MODE="user"
if [[ "$1" == "--system" ]]; then
    INSTALL_MODE="system"
fi

echo -e "${GREEN}Redshift Rebooted Installation Script${NC}"
echo "======================================"
echo ""

# Check if running as root for system install
if [[ "$INSTALL_MODE" == "system" ]] && [[ $EUID -ne 0 ]]; then
    echo -e "${RED}Error: System installation requires root privileges${NC}"
    echo "Please run: sudo $0 --system"
    exit 1
fi

# Build the project
echo -e "${YELLOW}Building redshift-rebooted...${NC}"
cd "$PROJECT_DIR"
cargo build --release

if [[ ! -f "$PROJECT_DIR/target/release/redshift-rebooted" ]]; then
    echo -e "${RED}Error: Build failed - binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Set installation paths based on mode
if [[ "$INSTALL_MODE" == "user" ]]; then
    BIN_DIR="$HOME/.local/bin"
    SYSTEMD_DIR="$HOME/.config/systemd/user"
    SYSTEMCTL_CMD="systemctl --user"
    INSTALL_TYPE="user"
else
    BIN_DIR="/usr/bin"
    SYSTEMD_DIR="/usr/lib/systemd/user"
    SYSTEMCTL_CMD="systemctl --user"
    INSTALL_TYPE="system-wide (user service)"
fi

echo -e "${YELLOW}Installing binary...${NC}"
mkdir -p "$BIN_DIR"
cp "$PROJECT_DIR/target/release/redshift-rebooted" "$BIN_DIR/redshift-rebooted"
chmod +x "$BIN_DIR/redshift-rebooted"
echo -e "${GREEN}✓ Binary installed to $BIN_DIR/redshift-rebooted${NC}"
echo ""

echo -e "${YELLOW}Installing systemd service...${NC}"
mkdir -p "$SYSTEMD_DIR"
cp "$PROJECT_DIR/systemd/redshift-rebooted.service" "$SYSTEMD_DIR/redshift-rebooted.service"

# Update ExecStart path in service file for system install
if [[ "$INSTALL_MODE" == "system" ]]; then
    sed -i "s|%h/.local/bin/redshift-rebooted|/usr/bin/redshift-rebooted|g" "$SYSTEMD_DIR/redshift-rebooted.service"
fi

echo -e "${GREEN}✓ Service file installed to $SYSTEMD_DIR/redshift-rebooted.service${NC}"
echo ""

# Reload systemd
echo -e "${YELLOW}Reloading systemd daemon...${NC}"
$SYSTEMCTL_CMD daemon-reload
echo -e "${GREEN}✓ Systemd daemon reloaded${NC}"
echo ""

# Print success message and instructions
echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Next steps:"
echo "==========="
echo ""
echo "1. Enable the service to start at login:"
echo -e "   ${YELLOW}systemctl --user enable redshift-rebooted${NC}"
echo ""
echo "2. Start the service now:"
echo -e "   ${YELLOW}systemctl --user start redshift-rebooted${NC}"
echo ""
echo "3. Check service status:"
echo -e "   ${YELLOW}systemctl --user status redshift-rebooted${NC}"
echo ""
echo "4. View service logs:"
echo -e "   ${YELLOW}journalctl --user -u redshift-rebooted -f${NC}"
echo ""
echo "Notes:"
echo "------"
echo "- Configure your location and preferences in ~/.config/redshift/redshift.conf"
echo "- Sample config: $PROJECT_DIR/redshift.conf.sample"
echo "- Restart the service after config changes: systemctl --user restart redshift-rebooted"
echo ""
