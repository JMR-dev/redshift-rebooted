#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default to user installation
INSTALL_MODE="user"
if [[ "$1" == "--system" ]]; then
    INSTALL_MODE="system"
fi

echo -e "${YELLOW}Redshift Rebooted Uninstallation Script${NC}"
echo "========================================"
echo ""

# Check if running as root for system uninstall
if [[ "$INSTALL_MODE" == "system" ]] && [[ $EUID -ne 0 ]]; then
    echo -e "${RED}Error: System uninstallation requires root privileges${NC}"
    echo "Please run: sudo $0 --system"
    exit 1
fi

# Set paths based on mode
if [[ "$INSTALL_MODE" == "user" ]]; then
    BIN_PATH="$HOME/.local/bin/redshift-rebooted"
    SERVICE_PATH="$HOME/.config/systemd/user/redshift-rebooted.service"
    SYSTEMCTL_CMD="systemctl --user"
else
    BIN_PATH="/usr/bin/redshift-rebooted"
    SERVICE_PATH="/usr/lib/systemd/user/redshift-rebooted.service"
    SYSTEMCTL_CMD="systemctl --user"
fi

# Stop and disable service if running
echo -e "${YELLOW}Stopping and disabling service...${NC}"
if $SYSTEMCTL_CMD is-active --quiet redshift-rebooted 2>/dev/null; then
    $SYSTEMCTL_CMD stop redshift-rebooted
    echo -e "${GREEN}✓ Service stopped${NC}"
fi

if $SYSTEMCTL_CMD is-enabled --quiet redshift-rebooted 2>/dev/null; then
    $SYSTEMCTL_CMD disable redshift-rebooted
    echo -e "${GREEN}✓ Service disabled${NC}"
fi
echo ""

# Remove service file
if [[ -f "$SERVICE_PATH" ]]; then
    echo -e "${YELLOW}Removing service file...${NC}"
    rm -f "$SERVICE_PATH"
    echo -e "${GREEN}✓ Service file removed: $SERVICE_PATH${NC}"
else
    echo -e "${YELLOW}Service file not found: $SERVICE_PATH${NC}"
fi
echo ""

# Remove binary
if [[ -f "$BIN_PATH" ]]; then
    echo -e "${YELLOW}Removing binary...${NC}"
    rm -f "$BIN_PATH"
    echo -e "${GREEN}✓ Binary removed: $BIN_PATH${NC}"
else
    echo -e "${YELLOW}Binary not found: $BIN_PATH${NC}"
fi
echo ""

# Reload systemd
echo -e "${YELLOW}Reloading systemd daemon...${NC}"
$SYSTEMCTL_CMD daemon-reload
echo -e "${GREEN}✓ Systemd daemon reloaded${NC}"
echo ""

echo -e "${GREEN}Uninstallation complete!${NC}"
echo ""
echo "Note: Configuration files in ~/.config/redshift/ were not removed."
echo "You can manually delete them if desired."
echo ""
