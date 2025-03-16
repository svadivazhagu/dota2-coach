// src/bin/coach.rs
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use colored::Colorize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use chrono::Local;

// Root game state structure
#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameState {
    provider: Option<Provider>,
    map: Option<Map>,
    player: Option<Player>,
    hero: Option<Hero>,
    minimap: Option<HashMap<String, MinimapObject>>,
    buildings: Option<HashMap<String, HashMap<String, Building>>>,
    
    // Fallback for any other fields
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Provider {
    name: Option<String>,
    appid: Option<i32>,
    version: Option<i32>,
    timestamp: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Map {
    name: Option<String>,
    matchid: Option<String>,
    game_time: Option<i32>,
    game_state: Option<String>,
    paused: Option<bool>,
    daytime: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    team_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Hero {
    name: Option<String>,
    level: Option<i32>,
    xpos: Option<i32>,
    ypos: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MinimapObject {
    image: String,
    name: Option<String>,
    team: i32,
    xpos: i32,
    ypos: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Building {
    health: i32,
    max_health: i32,
}

// Persistent state for enemy heroes
#[derive(Clone, Debug)]
struct EnemyHeroState {
    name: String,
    last_seen_position: (i32, i32),
    last_seen_time: i32,
    estimated_level: i32,
    times_spotted: i32,
}

// Format game time from seconds to MM:SS format
fn format_game_time(seconds: Option<i32>) -> String {
    if let Some(secs) = seconds {
        let minutes = secs / 60;
        let remaining_seconds = secs % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    } else {
        "Unknown".to_string()
    }
}

// Format hero names from "npc_dota_hero_xxx" to a readable format
fn format_hero_name(name: &str) -> String {
    let name = name.replace("npc_dota_hero_", "");
    
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Calculate distance between two points
fn calculate_distance(pos1: (i32, i32), pos2: (i32, i32)) -> f32 {
    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    ((dx * dx + dy * dy) as f32).sqrt()
}

// Estimate hero level based on game time
fn estimate_hero_level(game_time: i32) -> i32 {
    let minutes = game_time / 60;
    
    if minutes < 10 {
        (minutes / 2) + 1
    } else if minutes < 20 {
        (minutes / 3) + 5
    } else {
        (minutes / 5) + 10
    }
}

// Save game state to file for later analysis
fn save_game_state(state: &GameState, enemy_states: &HashMap<String, EnemyHeroState>) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("dota_state_{}.json", timestamp);
    
    // Create a combined state object
    let mut combined_state = serde_json::to_value(state).unwrap_or(Value::Null);
    
    // Add enemy tracking data
    let enemy_data: HashMap<String, serde_json::Value> = enemy_states.iter()
        .map(|(k, v)| (k.clone(), serde_json::json!({
            "name": v.name,
            "last_seen_position": [v.last_seen_position.0, v.last_seen_position.1],
            "last_seen_time": v.last_seen_time,
            "estimated_level": v.estimated_level,
            "times_spotted": v.times_spotted
        })))
        .collect();
    
    if let Value::Object(ref mut map) = combined_state {
        map.insert("enemy_tracking".to_string(), serde_json::to_value(enemy_data).unwrap());
    }
    
    if let Ok(mut file) = File::create(filename) {
        let _ = file.write_all(serde_json::to_string_pretty(&combined_state).unwrap().as_bytes());
    }
}

#[tokio::main]
async fn main() {
    println!("{}", "Dota 2 Coach - Enemy Tracking".green().bold());
    println!("{}", "============================".green());
    println!("Starting server on port 3000...");
    
    // Create shared state
    let game_state = Arc::new(Mutex::new(None::<GameState>));
    let enemy_states = Arc::new(Mutex::new(HashMap::<String, EnemyHeroState>::new()));
    
    // Clones for the server endpoint
    let game_state_clone = game_state.clone();
    let enemy_states_clone = enemy_states.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::content_length_limit(1024 * 1024 * 10))
        .and(warp::body::json())
        .map(move |data: Value| {
            // Parse the incoming JSON
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Update enemy information
                    let current_game_time = state.map.as_ref()
                        .and_then(|m| m.game_time)
                        .unwrap_or(0);
                    
                    // Determine player's team
                    let player_team = state.player.as_ref()
                        .and_then(|p| p.team_name.as_ref())
                        .map(|t| t.to_lowercase())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let enemy_team_id = if player_team == "radiant" { 3 } else { 2 };
                    
                    // Track currently visible enemies
                    let mut visible_enemies = Vec::new();
                    
                    // Extract currently visible enemies from minimap
                    if let Some(minimap) = &state.minimap {
                        for (_, obj) in minimap {
                            if obj.image == "minimap_enemyicon" && obj.team == enemy_team_id {
                                if let Some(name) = &obj.name {
                                    let hero_name = format_hero_name(name);
                                    visible_enemies.push((hero_name, (obj.xpos, obj.ypos)));
                                }
                            }
                        }
                    }
                    
                    // Update enemy states with the collected data
                    {
                        let mut enemy_map = enemy_states_clone.lock().unwrap();
                        
                        for (name, position) in visible_enemies {
                            let times_spotted = enemy_map.get(&name)
                                .map(|existing| existing.times_spotted + 1)
                                .unwrap_or(1);
                            
                            // Update or create entry
                            enemy_map.insert(name.clone(), EnemyHeroState {
                                name: name.clone(),
                                last_seen_position: position,
                                last_seen_time: current_game_time,
                                estimated_level: estimate_hero_level(current_game_time),
                                times_spotted,
                            });
                        }
                    }
                    
                    // Store the game state
                    let mut gs = game_state_clone.lock().unwrap();
                    *gs = Some(state);
                },
                Err(e) => {
                    eprintln!("Error parsing game state: {}", e);
                }
            }
            
            "OK"
        });
    
    // Start the webserver in a separate thread
    let server_thread = tokio::spawn(async move {
        warp::serve(gsi_endpoint)
            .run(([127, 0, 0, 1], 3000))
            .await;
    });
    
    println!("{}", "Server running! Waiting for Dota 2 data...".yellow());
    println!("{}", "Make sure you have configured the GSI config file in Dota 2.".yellow());
    println!("{}", "Add -gamestateintegration to Dota 2 launch options".yellow());
    
    // Create a thread for displaying information
    let display_thread = thread::spawn(move || {
        let mut last_update_time = std::time::Instant::now();
        let mut last_save_time = std::time::Instant::now();
        
        loop {
            // Only update display every 1 second
            if last_update_time.elapsed() >= Duration::from_secs(1) {
                // Try to get the current game state
                let state_option = {
                    let state = game_state.lock().unwrap();
                    state.clone()
                };
                
                // Get enemy hero states
                let enemy_state_data = {
                    let enemy_map = enemy_states.lock().unwrap();
                    enemy_map.clone()
                };
                
                // Display information
                if let Some(state) = state_option.clone() {
                    display_game_information(&state, &enemy_state_data);
                    last_update_time = std::time::Instant::now();
                    
                    // Save state every 5 minutes
                    if last_save_time.elapsed() >= Duration::from_secs(300) {
                        if let Some(ref state) = state_option {
                            save_game_state(state, &enemy_state_data);
                        }
                        last_save_time = std::time::Instant::now();
                    }
                }
            }
            
            // Sleep to avoid excessive CPU usage
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Wait for the threads to complete (they won't in normal operation)
    display_thread.join().unwrap();
    server_thread.abort();
}

// Display game information
fn display_game_information(state: &GameState, enemy_states: &HashMap<String, EnemyHeroState>) {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");
    
    println!("{}", "DOTA 2 GAME STATUS".green().bold());
    println!("{}", "=================".green());
    
    // Display game time
    if let Some(map) = &state.map {
        println!("\n{}", "GAME TIME".yellow().bold());
        println!("Time: {}", format_game_time(map.game_time));
        println!("State: {}", map.game_state.as_deref().unwrap_or("Unknown"));
        println!("Environment: {}", 
            if map.daytime.unwrap_or(true) { "Day".bright_blue() } else { "Night".blue() });
    }
    
    // Get player team
    let player_team = state.player.as_ref()
        .and_then(|p| p.team_name.as_ref())
        .map(|t| t.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    
    let enemy_team_name = if player_team == "radiant" { "Dire" } else { "Radiant" };
    
    // Get player position
    let player_position = if let Some(hero) = &state.hero {
        match (hero.xpos, hero.ypos) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None
        }
    } else {
        None
    };
    
    // Display enemy heroes with persistent tracking
    println!("\n{}", "ENEMY HEROES".red().bold());
    println!("{}", "============".red());
    
    if enemy_states.is_empty() {
        println!("No enemy heroes detected yet");
    } else {
        // Get current game time
        let current_game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Sort enemies by last seen time (most recent first)
        let mut sorted_enemies: Vec<_> = enemy_states.values().collect();
        sorted_enemies.sort_by(|a, b| b.last_seen_time.cmp(&a.last_seen_time));
        
        for hero in sorted_enemies {
            // Calculate time since last seen
            let seconds_since_seen = current_game_time - hero.last_seen_time;
            
            let name_display = if seconds_since_seen < 5 {
                format!("{} (Level {})", hero.name, hero.estimated_level)
            } else if seconds_since_seen < 30 {
                format!("{} (Level {})", hero.name, hero.estimated_level)
            } else {
                format!("{} (Est. Level {})", hero.name, hero.estimated_level)
            };
            
            // Use proper colored formatting
            if seconds_since_seen < 5 {
                println!("Hero: {}", name_display.red().bold());
            } else if seconds_since_seen < 30 {
                println!("Hero: {}", name_display.yellow());
            } else {
                println!("Hero: {}", name_display);
            }
            
            println!("  Last seen: {} ({} seconds ago)", 
                format_game_time(Some(hero.last_seen_time)),
                seconds_since_seen);
            
            println!("  Position: ({}, {})", hero.last_seen_position.0, hero.last_seen_position.1);
            
            // Show relative distance if player position is available
            if let Some(player_pos) = player_position {
                let distance = calculate_distance(player_pos, hero.last_seen_position);
                
                if distance < 1000.0 {
                    println!("  Distance from you: {} ({:.0} units)", "VERY CLOSE!".red().bold(), distance);
                } else if distance < 2000.0 {
                    println!("  Distance from you: {} ({:.0} units)", "Nearby".yellow(), distance);
                } else if distance < 4000.0 {
                    println!("  Distance from you: {} ({:.0} units)", "Medium distance", distance);
                } else {
                    println!("  Distance from you: {} ({:.0} units)", "Far away".green(), distance);
                }
            }
            
            println!("  Times spotted: {}", hero.times_spotted);
            println!();
        }
    }
    
    // Display building status
    println!("\n{}", "BUILDING STATUS".yellow().bold());
    println!("{}", "===============".yellow());
    
    if let Some(buildings) = &state.buildings {
        // Display player team buildings
        let player_team_key = if player_team == "radiant" { "radiant" } else { "dire" };
        println!("{}", format!("YOUR TEAM ({})", player_team.to_uppercase()).green().bold());
        
        if let Some(team_buildings) = buildings.get(player_team_key) {
            for (name, building) in team_buildings {
                let health_percent = (building.health as f32 / building.max_health as f32 * 100.0) as i32;
                
                let health_display = if health_percent > 70 {
                    format!("{}%", health_percent).green()
                } else if health_percent > 30 {
                    format!("{}%", health_percent).yellow()
                } else {
                    format!("{}%", health_percent).red()
                };
                
                // Format building name for better readability
                let building_name = name.replace("dota_goodguys_", "")
                    .replace("dota_badguys_", "")
                    .replace("_", " ");
                
                println!("• {}: {}", building_name, health_display);
            }
        } else {
            println!("No building data available");
        }
        
        // Display enemy team buildings
        println!();
        println!("{}", format!("ENEMY TEAM ({})", enemy_team_name.to_uppercase()).red().bold());
        
        let enemy_team_key = if player_team == "radiant" { "dire" } else { "radiant" };
        if let Some(team_buildings) = buildings.get(enemy_team_key) {
            for (name, building) in team_buildings {
                let health_percent = (building.health as f32 / building.max_health as f32 * 100.0) as i32;
                
                let health_display = if health_percent > 70 {
                    format!("{}%", health_percent).green()
                } else if health_percent > 30 {
                    format!("{}%", health_percent).yellow()
                } else {
                    format!("{}%", health_percent).red()
                };
                
                // Format building name for better readability
                let building_name = name.replace("dota_goodguys_", "")
                    .replace("dota_badguys_", "")
                    .replace("_", " ");
                
                println!("• {}: {}", building_name, health_display);
            }
        } else {
            println!("No building data available");
        }
    } else {
        println!("No building data available");
    }
    
    println!("\nTracking enemy data for analysis. Press Ctrl+C to exit.");
}