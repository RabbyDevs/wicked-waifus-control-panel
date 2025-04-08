use std::path::PathBuf;
use std::process::{Command, Stdio};
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
    running: bool
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

fn update_terminal_settings() -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join("AppData")
        .join("Local")
        .join("Packages")
        .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
        .join("LocalState")
        .join("settings.json");

    if !settings_path.exists() {
        let settings_path = dirs::home_dir()
            .ok_or("Could not determine home directory")?
            .join("AppData")
            .join("Local")
            .join("Packages")
            .join("Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe")
            .join("LocalState")
            .join("settings.json");
            
        if !settings_path.exists() {
            return Err("Windows Terminal settings.json not found".into());
        }
    }
    
    println!("Updating Windows Terminal settings to close tabs when processes exit...");
    
    let settings_content = fs::read_to_string(&settings_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&settings_content)?;
    
    if let Some(profiles) = settings.get_mut("profiles") {
        if let Some(defaults) = profiles.get_mut("defaults") {
            defaults["closeOnExit"] = serde_json::json!("always");
        } else {
            profiles["defaults"] = serde_json::json!({
                "closeOnExit": "always"
            });
        }
    }
    
    fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;
    
    println!("Windows Terminal settings updated successfully.");
    
    Ok(())
}


fn main() {
    let config = get_config();
    let project_path = config.project_path;
    
    let mut server_state = ServerState {
        running: false
    };

    if let Err(e) = update_terminal_settings() {
        println!("Failed to update Windows Terminal settings: {}", e);
        println!("Terminal tabs may not close automatically when servers stop.");
    }

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
    
    let mut wt_args = Vec::new();
    
    for (i, server) in servers.iter().enumerate() {
        let window_title = format!("{}{}", window_prefix, server);
        let cmd = format!(
            "cd /d {} && cargo run {} --bin {} && pause",
            project_path, if release {"--release"} else {""}, server
        );

        if i == 0 {
            wt_args.push("--title".to_string());
            wt_args.push(window_title);
            wt_args.push("cmd".to_string());
            wt_args.push("/c".to_string());
            wt_args.push(cmd);
        } else {
            wt_args.push(";".to_string());
            wt_args.push("new-tab".to_string());
            wt_args.push("--title".to_string());
            wt_args.push(window_title);
            wt_args.push("cmd".to_string());
            wt_args.push("/c".to_string());
            wt_args.push(cmd);
        }
    }

    match Command::new("wt")
        .args(&wt_args)
        .spawn() {
            Ok(_) => {
                server_state.running = true;
            }
            Err(err) => {
                eprintln!("Error: Failed to launch Windows Terminal: {}", err);
            }
        }
}

fn stop_servers(server_state: &mut ServerState) {
    if !server_state.running {
        println!("Servers don't appear to be running.");
        return;
    }
    
    let kill_cargo = "Get-Process -Name cargo | ForEach-Object { $id = $_.Id; $cmdLine = (Get-CimInstance Win32_Process -Filter \"ProcessId = $id\").CommandLine; if ($cmdLine -like '*wicked-waifus*') { taskkill /F /PID $id } }".to_string();
    
    Command::new("powershell")
        .args(["-Command", &kill_cargo])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| child.wait())
        .ok();
    
    server_state.running = false;
}

fn restart_servers(project_path: &str, server_state: &mut ServerState) {
    stop_servers(server_state);
    thread::sleep(Duration::from_secs(2));
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