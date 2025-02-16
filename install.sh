#!/bin/bash

set -e  # Exit on error

# Ensure the script is run as root
if [ "$(id -u)" -ne 0 ]; then
    echo "This script must be run as root. Try: sudo $0"
    exit 1
fi

# Get the invoking user
USER_HOME=$(eval echo ~$SUDO_USER)
USER_SHELL=$(getent passwd "$SUDO_USER" | cut -d: -f7)

echo "Building aishell as $SUDO_USER..."
sudo -u "$SUDO_USER" env PATH="$USER_HOME/.cargo/bin:$PATH" cargo build --release

echo "Installing aishell..."
sudo rm -f /usr/local/bin/aishell
mv target/release/aishell /usr/local/bin

echo "Initializing aishell for $SUDO_USER..."
for shell_config in "$USER_HOME/.bashrc" "$USER_HOME/.zshrc"; do
    if [ -f "$shell_config" ]; then
        if ! grep -q 'eval "$(aishell init' "$shell_config"; then
            echo "Adding aishell init to $shell_config"
            echo 'eval "$(aishell init bash)"' >> "$shell_config"
        else
            echo "aishell already initialized in $shell_config"
        fi
    fi
done

echo "Reloading shell configuration..."
echo -e "\e[32mTo reload your shell configuration, run:\e[0m"
if [ -f "$USER_HOME/.bashrc" ]; then
    echo -e "\e[32msource $USER_HOME/.bashrc\e[0m"
elif [ -f "$USER_HOME/.zshrc" ]; then
    echo -e "\e[32msource $USER_HOME/.zshrc\e[0m"
fi

echo "Done."
