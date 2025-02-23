use std::env;
use std::io::{self, Write};
#[allow(unused)]
use std::process::{Command, Stdio};
use tokio::runtime::Runtime;
use nix::unistd::{fork, ForkResult, setsid};
use std::fs::{self, OpenOptions};
use std::path::Path;
mod ai;

/// This program receives all commands sent to the shell, if that command returns an error,
/// or the command was typed wrong like "gerp" instead of "grep" it will be able to suggest a fix for the command.

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: aishell <failed_command> <error_output>");
        return;
    }

    // The 'add-to-shell' command adds aishell to the user's shell config
    // It should be called once, and then the user should restart their shell
    if args[0] == "add-to-shell" {
        if let None = env::var_os("OPENAI_API_KEY") {
            eprintln!("Warning: OPENAI_API_KEY environment variable is not set.");
        }
        let user_home = std::env::var("HOME").expect("Failed to get HOME directory");
        if let Err(e) = add_to_shell_config(&user_home) {
            eprintln!("Error: {}", e);
        }
    }

    // The 'init' command tells the shell how to call aishell
    // It should be called in the shell's init file, e.g. ~/.bashrc or ~/.zshrc
    if args[0] == "init" {
        if args.len() < 2 {
            eprintln!("Usage: aishell init <shell>");
            return;
        }
        init_shell(&args[1]);
        if let None = env::var_os("OPENAI_API_KEY") {
            eprintln!("OPENAI_API_KEY environment variable is not set.");
        }
        return;
    }

    let command = args[0].clone();
    let error_msg = args[1..].join(" "); // Preserve full error message

    println!("{}", error_msg.replace("\\n", "\n")); // Print the exact error message from the shell

    print!("🤖 loading...");
    io::stdout().flush().unwrap();

    unsafe {
        match fork() {
            Ok(ForkResult::Child) => {
                // detach from the parent’s session so it keeps running
                setsid().ok();
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    // Do async work here, e.g. calling your get_ai_suggestion
                    let fix = get_ai_suggestion(&command, &error_msg).await;
                    // print!("\x1B[F\x1B[2K"); // ANSI: Move cursor up + Clear line
                    print!("\x1B[s"); // Save cursor position
                    print!("\x1B[F\x1B[F\x1B[2K"); // ANSI: Move cursor up twice + Clear line
                    io::stdout().flush().unwrap();
                    if let Some(fix) = fix {
                        print!("🤖 {} \x1B[32m(Ctrl+T)\x1B[0m", fix);
                        io::stdout().flush().unwrap();
                        insert_into_history(&fix);
                    } else {
                        println!("🤖 No suggestion available.");
                    }
                    // print!("\x1B[2E\x1B[999C"); // Move down two lines, then move cursor far right
                    // print!("\x1B[2E"); // Move down two lines, but DO NOT reset cursor position
                    // print!("\x1B[2E\x1B[998C"); // Move down 2 lines, then move to the far right minus one
                    print!("\x1B[u"); // Restore cursor position
                    io::stdout().flush().unwrap();
                });
                std::process::exit(0);
            }
            Ok(ForkResult::Parent { .. }) => {
                // Return immediately, letting the child continue
            }
            Err(e) => eprintln!("fork failed: {}", e),
        }
    }
}

/// Inserts the AI-generated fix into shell history **without running it**
fn insert_into_history(command: &str) {
    // These are other methods I tried.
    // if let Some(history_file) = env::var_os("HISTFILE") {
    //     let mut file = std::fs::OpenOptions::new()
    //         .append(true)
    //         .open(history_file)
    //         .unwrap();
    //     writeln!(file, "{}", command).unwrap();
    // } else {
    //     eprintln!("HISTFILE environment variable is not set.");
    // }
    // let _ = std::process::Command::new("bash")
    //     .arg("-c")
    //     .arg(format!("history -s '{}'", command))
    //     .status();
    // let _ = std::process::Command::new("history")
    //     .args(&["-s", command])
    //     .status();
    // Overwrite to a file /tmp/aishell_suggestion
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/aishell_suggestion")
        .unwrap();
    writeln!(file, "{}", command).unwrap();
}

/// Simulates an AI-generated fix (Replace this with real API logic)
async fn get_ai_suggestion(command: &str, error: &str) -> Option<String> {
    let response = ai::get_ai_response(command, error).await; // Await the async response
    // Handle the response and extract the suggestion now!
    if let Ok(suggestion) = response {
        if let Some(suggested_command) = suggestion.get("suggested_exact_command").and_then(|v| v.as_str()) {
            return Some(suggested_command.to_string());
        }
    }
    None
}

#[allow(unused)]
fn init_shell(shell: &str) {
    match shell {
        "bash" | "zsh" => {
            println!(
                "trap 'aishell \"$BASH_COMMAND\" \"$?\"' ERR"
                // "trap 'fix=\"$(aishell \"$BASH_COMMAND\" \"$?\")\"; history -s \"$fix\"' ERR"
                // "trap 'fix=\"$(aishell \"$BASH_COMMAND\" \"$?\" 2>&1 1>/dev/null)\"; history -s \"$fix\"' ERR"
            );
        }
        "fish" => {
            println!(
                "function fish_postexec --on-event fish_postexec; if test $status -ne 0; aishell \"$argv\"; end; end"
            );
        }
        _ => eprintln!("Shell not supported"),
    }
}

fn add_to_shell_config(user_home: &str) -> io::Result<()> {
    println!("Initializing aishell for user...");
    
    let shell_configs = vec![
        format!("{}/.bashrc", user_home),
        format!("{}/.zshrc", user_home),
    ];

    let aishell_init = "eval \"$(aishell init bash)\"";
    let aishell_function = r#"
aishell_suggestion() {
  local suggestion_file="/tmp/aishell_suggestion"
  if [ -f "$suggestion_file" ]; then
    fix=$(< "$suggestion_file")
    rm -f "$suggestion_file"
    READLINE_LINE="$fix"
    READLINE_POINT=${#fix}
  else
    echo "No suggestion available."
  fi
}
bind -x '\"\\C-t\":aishell_suggestion'
"#;

    for shell_config in shell_configs {
        let path = Path::new(&shell_config);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            if !content.contains("eval \"$(aishell init") {
                println!("Adding aishell init to {}", shell_config);
                let mut file = OpenOptions::new().append(true).open(path)?;
                writeln!(file, "{}", aishell_init)?;
                writeln!(file, "{}", aishell_function)?;
            } else {
                println!("aishell already initialized in {}", shell_config);
            }
        }
    }
    
    Ok(())
}
