echo "Initializing aishell for $SUDO_USER..."
for shell_config in "$USER_HOME/.bashrc" "$USER_HOME/.zshrc"; do
    if [ -f "$shell_config" ]; then
        if ! grep -q 'eval "$(aishell init' "$shell_config"; then
            echo "Adding aishell init to $shell_config"
            echo 'eval "$(aishell init bash)"' >> "$shell_config"
            echo 'aishell_suggestion() {' >> "$shell_config"
            echo '  local suggestion_file="/tmp/aishell_suggestion"' >> "$shell_config"
            echo '  if [ -f "$suggestion_file" ]; then' >> "$shell_config"
            echo '    fix=$(< "$suggestion_file")' >> "$shell_config"
            echo '    rm -f "$suggestion_file"' >> "$shell_config"
            echo '    READLINE_LINE="$fix"' >> "$shell_config"
            echo '    READLINE_POINT=${#fix}' >> "$shell_config"
            echo '  else' >> "$shell_config"
            echo '    echo "No suggestion available."' >> "$shell_config"
            echo '  fi' >> "$shell_config"
            echo '}' >> "$shell_config"
            #echo 'bind -x \'"\\C-t":aishell_suggestion\'' >> "$shell_config"
            printf "bind -x '\\\"\\\\C-t\\\":aishell_suggestion'\n" >> "$shell_config"
        else
            echo "aishell already initialized in $shell_config"
        fi
    fi
done