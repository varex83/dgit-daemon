#!/bin/bash

# This script sets up the necessary environment variables for IPFS integration
# It configures both local IPFS daemon and gateway settings

# Check if .env file exists and create it if not
if [ ! -f .env ]; then
    echo "Creating .env file..."
    touch .env
else
    echo ".env file already exists, appending to it..."
fi

# Set IPFS API URL
echo -e "\n====================== IPFS CONFIGURATION ======================"
echo "Setting up IPFS API and Gateway configuration"

# Default IPFS API URL
DEFAULT_IPFS_API="http://127.0.0.1:5001"
read -p "Enter IPFS API URL (default: $DEFAULT_IPFS_API): " ipfs_api_url
if [ -z "$ipfs_api_url" ]; then
    ipfs_api_url="$DEFAULT_IPFS_API"
    echo "Using default IPFS API URL: $DEFAULT_IPFS_API"
fi

# Check if IPFS daemon is running
echo "Checking if IPFS daemon is running at $ipfs_api_url..."
if curl -s "$ipfs_api_url/api/v0/version" > /dev/null; then
    echo "✅ IPFS daemon is running and accessible"
    
    # Get node ID to verify connection
    NODE_ID=$(curl -s "$ipfs_api_url/api/v0/id" | grep -o '"ID": "[^"]*"' | cut -d'"' -f4)
    if [ ! -z "$NODE_ID" ]; then
        echo "Connected to IPFS node: $NODE_ID"
    fi
else
    echo "⚠️ Could not connect to IPFS daemon at $ipfs_api_url"
    echo "Make sure your IPFS daemon is running with the command:"
    echo "    ipfs daemon"
    echo ""
    echo "Installation instructions:"
    echo "    https://docs.ipfs.tech/install/command-line/"
    echo ""
    echo "Continuing setup anyway..."
fi

# Set IPFS Gateway URL
echo -e "\nChoose an IPFS Gateway for content access:"
echo "1. Local Gateway (if running local daemon, fastest)"
echo "2. IPFS.io Gateway (public, rate limited)"
echo "3. Cloudflare Gateway (public, rate limited)"
echo "4. Custom Gateway"
read -p "Enter your choice (1-4, default: 1): " gateway_choice

case $gateway_choice in
    2)
        ipfs_prefix="https://ipfs.io/ipfs/"
        echo "Using IPFS.io Gateway"
        ;;
    3)
        ipfs_prefix="https://cloudflare-ipfs.com/ipfs/"
        echo "Using Cloudflare IPFS Gateway"
        ;;
    4)
        read -p "Enter your custom IPFS Gateway URL (including /ipfs/ suffix): " ipfs_prefix
        ;;
    *)
        ipfs_prefix="http://127.0.0.1:8080/ipfs/"
        echo "Using Local IPFS Gateway"
        ;;
esac

# Try to verify the gateway works
echo "Verifying gateway access..."
if curl -s "${ipfs_prefix}QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme" > /dev/null; then
    echo "✅ Successfully connected to gateway at $ipfs_prefix"
else
    echo "⚠️ Could not connect to gateway at $ipfs_prefix"
    echo "This might be due to the gateway being unavailable or network issues."
    echo "Gateway verification is not critical, continuing setup..."
fi

# Update .env file with IPFS configuration
echo -e "\n# IPFS Configuration" >> .env
echo "IPFS_API_URL=$ipfs_api_url" >> .env
echo "IPFS_PREFIX=$ipfs_prefix" >> .env

echo -e "\nIPFS configuration has been set up successfully in .env file"
echo -e "\nIf you encounter issues with IPFS connectivity:"
echo "1. Verify your IPFS daemon is running with 'ipfs daemon'"
echo "2. Make sure your API URL is correct (currently: $ipfs_api_url)"
echo "3. Make sure your gateway URL is correct (currently: $ipfs_prefix)"
echo "4. If using local gateway, ensure the daemon is running with public gateway enabled"
echo "5. For remote gateways, check your network connection and firewall settings" 