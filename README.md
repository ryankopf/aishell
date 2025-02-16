Add to shell in ~/.bashrc or ~/.zshrc

eval "$(aishell init bash)"


cargo build --release
mv target/release/aishell /usr/local/bin
eval "$(aishell init bash)"  # Or zsh/fish



# Function to initialize aishell error trap
init_aishell() {
    if [ -n "$BASH_VERSION" ]; then
        trap 'aishell "$BASH_COMMAND" "$?"' ERR
    elif [ -n "$ZSH_VERSION" ]; then
        trap 'aishell "$ZSH_COMMAND" "$?"' ERR
    fi
}

# Initialize aishell
init_aishell