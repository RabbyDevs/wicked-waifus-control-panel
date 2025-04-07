use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::env;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    project_path: String,
}

fn main() {
    let config_path = "config-panel.toml";
    if !PathBuf::from(config_path).exists() {
        eprintln!("Error: config-panel.toml not found. Please create it and define 'project_path'.");
        return;
    }

    let config_content = fs::read_to_string(config_path).unwrap_or_else(|_| {
        eprintln!("Error: Failed to read config-panel.toml.");
        String::new()
    });

    let config: Config = toml::from_str(&config_content).unwrap_or_else(|_| {
        eprintln!("Error: Failed to parse config-panel.toml. Ensure it contains 'project_path'.");
        std::process::exit(1);
    });

    let project_path = config.project_path;

    let marker_file = env::var("TEMP").unwrap_or_else(|_| String::from(".")) + "\\ww-server-terminals.txt";

    loop {
        println!("===================================================");
        println!("       WICKED WAIFUS SERVERS CONTROL PANEL");
        println!("===================================================");
        println!();
        println!(" [1] Start All Servers");
        println!(" [2] Restart All Servers");
        println!(" [3] Stop All Servers");
        println!(" [4] Exit");
        println!(" [5] Start Servers and Launch Wuthering Waves");
        println!();
        println!("===================================================");
        println!();

        let choice = get_user_input("Enter your choice (1-5): ");
        match choice.trim() {
            "1" => start_servers(&project_path, &marker_file),
            "2" => restart_servers(&project_path, &marker_file),
            "3" => stop_servers(&marker_file),
            "4" => {
                exit_control_panel(&marker_file);
                break;
            }
            "5" => start_servers_and_launch_launcher(&project_path, &marker_file),
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn start_servers(project_path: &str, marker_file: &str) {
    if PathBuf::from(marker_file).exists() {
        println!("Servers appear to be already running.");
        println!("Stop them first or restart them.");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    let servers = [
        "wicked-waifus-config-server",
        "wicked-waifus-hotpatch-server",
        "wicked-waifus-login-server",
        "wicked-waifus-gateway-server",
        "wicked-waifus-game-server",
    ];

    let mut wt_command_string = String::new();
    for (i, server) in servers.iter().enumerate() {
        let cmd = format!(
            "cd /d {} && cargo run --bin {} && exit",
            project_path, server
        );

        if i == 0 {
            wt_command_string.push_str(&format!(
                "--title \"{}\" cmd /c \"{}\"",
                server, cmd
            ));
        } else {
            wt_command_string.push_str(&format!(
                " ; new-tab --title \"{}\" cmd /c \"{}\"",
                server, cmd
            ));
        }
    }

    let status = Command::new("wt")
        .args(wt_command_string.split_whitespace())
        .status()
        .unwrap_or_else(|err| {
            eprintln!("Error: Failed to launch Windows Terminal: {}", err);
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("Error: Failed to start servers.");
    } else {
        fs::write(marker_file, "WindowsTerminal").unwrap_or_else(|_| {
            eprintln!("Error: Failed to create marker file.");
        });
        println!("All servers started successfully.");
    }
}

fn stop_servers(marker_file: &str) {
    if !PathBuf::from(marker_file).exists() {
        println!("No server terminals are running.");
        return;
    }

    let servers = [
        "wicked-waifus-config-server",
        "wicked-waifus-hotpatch-server",
        "wicked-waifus-login-server",
        "wicked-waifus-gateway-server",
        "wicked-waifus-game-server",
    ];

    for server in servers {
        match Command::new("taskkill")
            .args(["/f", "/im", &format!("{}.exe", server)])
            .output() {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!(
                            "Warning: Failed to kill process '{}'. Output: {:?}",
                            server,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    } else {
                        println!("Successfully stopped {}", server);
                    }
                },
                Err(err) => {
                    eprintln!("Warning: Failed to execute taskkill for '{}': {}", server, err);
                }
            };
    }

    match Command::new("taskkill")
        .args(["/F", "/IM", "WindowsTerminal.exe"])
        .output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!(
                        "Warning: Failed to close Windows Terminal. Output: {:?}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                } else {
                    println!("Successfully stopped Terminal");
                }
            },
            Err(_) => {
                eprintln!("Error: Failed to close Windows Terminal.")
            }
        };

    fs::remove_file(marker_file).unwrap_or_else(|_| {
        eprintln!("Error: Failed to remove marker file.");
    });

    println!("All servers stopped and terminal closed.");
}

fn restart_servers(project_path: &str, marker_file: &str) {
    stop_servers(marker_file);
    thread::sleep(Duration::from_secs(1));
    start_servers(project_path, marker_file);
}

fn exit_control_panel(marker_file: &str) {
    if PathBuf::from(marker_file).exists() {
        println!("WARNING: Servers are still running!");
        let choice = get_user_input("Stop servers before exiting? (Y/N): ");
        if choice.trim().to_lowercase() == "y" {
            stop_servers(marker_file);
        }
    }
    println!("Exiting control panel...");
}

fn start_servers_and_launch_launcher(project_path: &str, marker_file: &str) {
    if PathBuf::from(marker_file).exists() {
        println!("Servers appear to be already running.");
        println!("No need to start them again.");
        thread::sleep(Duration::from_secs(3));
    } else {
        start_servers(project_path, marker_file);
    }

    let path = PathBuf::new();
    let launcher_path = env::current_exe().unwrap_or_else(|_| PathBuf::new()).parent().unwrap_or(path.as_path()).join("launcher.exe");
    
    if launcher_path.exists() {
        // Launch launcher.exe with administrator elevation using powershell
        println!("Launching launcher.exe with administrator privileges...");
        
        // Create a PowerShell command to start launcher.exe with elevation
        let powershell_command = format!(
            "Start-Process -FilePath \"{}\" -Verb RunAs",
            launcher_path.to_string_lossy()
        );
        
        let status = Command::new("powershell")
            .args(["-Command", &powershell_command])
            .status()
            .unwrap_or_else(|err| {
                eprintln!("Error: Failed to launch launcher with elevation: {}", err);
                std::process::exit(1);
            });
            
        if status.success() {
            println!("launcher.exe started successfully with administrator privileges.");
        } else {
            println!("Failed to start launcher.exe with administrator privileges.");
            println!("This might happen if you declined the UAC prompt.");
        }
    } else {
        println!("launcher.exe not found. Skipping this step.");
    }
}