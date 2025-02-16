# aishell

`aishell` is a smart shell assistant that understands commands sent to the shell. If a command fails or is typed incorrectly (e.g., "gerp" instead of "grep"), `aishell` suggests a fix for the command. The fix is placed into the shell history, so the user can press the up arrow and hit enter to execute the corrected command.

## Installation

To install `aishell` on Ubuntu using bash, follow these steps:

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/aishell.git
    cd aishell
    ```

2. Build the project:
    ```sh
    cargo build --release
    ```

3. Move the binary to `/usr/local/bin`:
    ```sh
    sudo mv target/release/aishell /usr/local/bin
    ```

4. Run the installation script:
    ```sh
    ./install.sh
    ```

## Usage

To initialize `aishell` in your shell, add the following to your `~/.bashrc` or `~/.zshrc` file:

```sh
eval "$(aishell init bash)"  # For bash
# or
eval "$(aishell init zsh)"   # For zsh
```

For fish shell, add the following to your `~/.config/fish/config.fish` file:

```sh
aishell init fish | source
```

## Old Notes

Add to shell in `~/.bashrc` or `~/.zshrc`:

```sh
eval "$(aishell init bash)"
```

Build and move the binary:

```sh
cargo build --release
sudo mv target/release/aishell /usr/local/bin
eval "$(aishell init bash)"  # Or zsh/fish
```

## Function to initialize aishell error trap

```sh
init_aishell() {
    if [ -n "$BASH_VERSION" ]; then
        trap 'aishell "$BASH_COMMAND" "$?"' ERR
    elif [ -n "$ZSH_VERSION" ]; then
        trap 'aishell "$ZSH_COMMAND" "$?"' ERR
    fi
}

# Initialize aishell
init_aishell