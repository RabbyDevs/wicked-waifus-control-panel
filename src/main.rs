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
    release: Option<bool>
}

struct ServerState {
    running: bool,
}

fn get_config() -> Config {
    let config_path = "config-panel.toml";
    if !PathBuf::from(config_path).exists() {
        eprintln!("Error: config-panel.toml not found. Please create it and define 'project_path'.");
        std::process::exit(1);
    }

    let config_content = fs::read_to_string(config_path).unwrap_or_else(|_| {
        eprintln!("Error: Failed to read config-panel.toml.");
        std::process::exit(1);
    });

    toml::from_str(&config_content).unwrap_or_else(|_| {
        eprintln!("Error: Failed to parse config-panel.toml. Please put actual toml in it.");
        std::process::exit(1);
    })
}

fn main() {
    let config = get_config();
    let project_path = config.project_path;
    
    let mut server_state = ServerState {
        running: false,
    };

    loop {
        // print!("\x1B[2J\x1B[1;1H");
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
            "1" => start_servers(&project_path, &mut server_state),
            "2" => restart_servers(&project_path, &mut server_state),
            "3" => stop_servers(&mut server_state),
            "4" => {
                exit_control_panel(&mut server_state);
                break;
            }
            "5" => start_servers_and_launch_launcher(&project_path, &mut server_state),
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

// Generate a unique window title prefix for our server terminals
fn get_window_title_prefix() -> String {
    "WW_".to_string()
}

fn start_servers(project_path: &str, server_state: &mut ServerState) {
    let config = get_config();
    let release = config.release.unwrap_or(false);
    
    if server_state.running {
        println!("Servers appear to be already running.");
        println!("Stop them first or restart them.");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    let servers = [
        "wicked-waifus-config-server",
        "wicked-waifus-login-server",
        "wicked-waifus-gateway-server",
        "wicked-waifus-game-server",
    ];

    let window_prefix = get_window_title_prefix();
    let mut wt_command_string = String::new();
    for (i, server) in servers.iter().enumerate() {
        let window_title = format!("{}{}", window_prefix, server);
        let cmd = format!(
            "cd /d {} && cargo run {} --bin {} && pause",
            project_path, if release {"--release"} else {""}, server
        );

        if i == 0 {
            wt_command_string.push_str(&format!(
                "--title \"{}\" cmd /c \"{}\"",
                window_title, cmd
            ));
        } else {
            wt_command_string.push_str(&format!(
                " ; new-tab --title \"{}\" cmd /c \"{}\"",
                window_title, cmd
            ));
        }
    }

    // Start Windows Terminal with unique window titles
    match Command::new("wt")
        .args(wt_command_string.split_whitespace())
        .spawn() {
            Ok(_) => {
                server_state.running = true;
                println!("All servers started successfully.");
                // Give the servers time to start up
                thread::sleep(Duration::from_secs(2));
            }
            Err(err) => {
                eprintln!("Error: Failed to launch Windows Terminal: {}", err);
            }
        }
}

fn stop_servers(server_state: &mut ServerState) {
    if !server_state.running {
        println!("No server terminals are running.");
        return;
    }

    let window_prefix = get_window_title_prefix();
    
    let ps_command = format!(
        "Get-Process | Where-Object {{$_.MainWindowTitle -like '*{}*'}} | ForEach-Object {{$_.Id}}",
        window_prefix
    );

    let output = Command::new("powershell")
        .args(["-Command", &ps_command])
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Error finding server processes. Assuming they're not running.");
                server_state.running = false;
                return;
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<&str> = stdout.split_whitespace().collect();
            
            if pids.is_empty() {
                println!("No server processes found. They may have been closed already.");
                server_state.running = false;
                return;
            }

            let mut all_killed = true;
            for pid in pids {
                if pid.trim().is_empty() {
                    continue;
                }
                
                println!("Stopping server with PID: {}", pid);
                
                match Command::new("taskkill")
                    .args(["/F", "/PID", pid])
                    .output() {
                        Ok(kill_output) => {
                            if !kill_output.status.success() {
                                eprintln!(
                                    "Warning: Failed to kill process {}. Output: {:?}",
                                    pid,
                                    String::from_utf8_lossy(&kill_output.stderr)
                                );
                                all_killed = false;
                            }
                        },
                        Err(e) => {
                            eprintln!("Error killing process {}: {}", pid, e);
                            all_killed = false;
                        }
                    }
            }

            if all_killed {
                println!("All server processes stopped successfully.");
                server_state.running = false;
            } else {
                println!("Some server processes could not be stopped.");
                server_state.running = false;
            }
        },
        Err(e) => {
            eprintln!("Error running PowerShell command: {}", e);
            println!("Unable to find server processes.");
            server_state.running = false;
        }
    }
}

fn restart_servers(project_path: &str, server_state: &mut ServerState) {
    stop_servers(server_state);
    start_servers(project_path, server_state);
}

fn exit_control_panel(server_state: &mut ServerState) {
    if server_state.running {
        println!("WARNING: Servers are still running!");
        let choice = get_user_input("Stop servers before exiting? (Y/N): ");
        if choice.trim().to_lowercase() == "y" {
            stop_servers(server_state);
        }
    }
    println!("Exiting control panel...");
}

fn start_servers_and_launch_launcher(project_path: &str, server_state: &mut ServerState) {
    if server_state.running {
        println!("Servers appear to be already running.");
        println!("No need to start them again.");
        thread::sleep(Duration::from_secs(3));
    } else {
        start_servers(project_path, server_state);
    }

    let path = PathBuf::new();
    let launcher_path = env::current_exe().unwrap_or_else(|_| PathBuf::new()).parent().unwrap_or(path.as_path()).join("launcher.exe");
    
    if launcher_path.exists() {
        println!("Launching launcher.exe with administrator privileges...");
        
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