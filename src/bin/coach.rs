// src/bin/coach.rs
use std::sync::{Arc, Mutex};
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
    status: EnemyStatus,
}

// Status tracking for enemy heroes
#[derive(Clone, Debug, PartialEq)]
enum EnemyStatus {
    NewlySpotted,
    Tracking,
    MovedSignificantly,
    Lost,
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

// Describe a position relative to player
fn describe_position_relative_to_player(player_pos: (i32, i32), enemy_pos: (i32, i32)) -> String {
    let distance = calculate_distance(player_pos, enemy_pos);
    
    // Determine the direction
    let dx = enemy_pos.0 - player_pos.0;
    let dy = enemy_pos.1 - player_pos.1;
    
    let direction = if dx.abs() > dy.abs() * 2.0 as i32 {
        if dx > 0 { "east" } else { "west" }
    } else if dy.abs() > dx.abs() * 2.0 as i32 {
        if dy > 0 { "north" } else { "south" }
    } else if dx > 0 && dy > 0 {
        "northeast"
    } else if dx > 0 && dy < 0 {
        "southeast"
    } else if dx < 0 && dy > 0 {
        "northwest"
    } else {
        "southwest"
    };
    
    // Determine distance description
    let distance_desc = if distance < 1000.0 {
        "very close to you".red().bold().to_string()
    } else if distance < 2000.0 {
        "nearby".yellow().to_string()
    } else if distance < 4000.0 {
        "at medium distance".to_string()
    } else {
        "far away".green().to_string()
    };
    
    format!("{} to the {}", distance_desc, direction)
}

// Convert map position to a named location (approximate)
fn describe_map_location(position: (i32, i32)) -> String {
    // These are very approximate location markers
    let x = position.0;
    let y = position.1;

    // Simple map quadrants
    if x > 5000 && y > 5000 {
        "Radiant jungle".to_string()
    } else if x > 3000 && y < -3000 {
        "Dire jungle".to_string()
    } else if x.abs() < 3000 && y.abs() < 3000 {
        "mid lane area".to_string()
    } else if x > 0 && y > 0 {
        "Radiant top lane".to_string()
    } else if x < 0 && y > 0 {
        "Radiant bottom lane".to_string()
    } else if x > 0 && y < 0 {
        "Dire top lane".to_string()
    } else {
        "Dire bottom lane".to_string()
    }
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

// Check if enemy has moved significantly
fn has_moved_significantly(old_pos: (i32, i32), new_pos: (i32, i32)) -> bool {
    calculate_distance(old_pos, new_pos) > 1000.0
}

#[tokio::main]
async fn main() {
    println!("{}", "Dota 2 Coach - Enemy Tracking".green().bold());
    println!("{}", "============================".green());
    println!("Starting server on port 3000...");
    
    // Create shared state
    let game_state = Arc::new(Mutex::new(None::<GameState>));
    let enemy_states = Arc::new(Mutex::new(HashMap::<String, EnemyHeroState>::new()));
    let last_game_time = Arc::new(Mutex::new(-1));
    let enemy_team_heroes = Arc::new(Mutex::new(Vec::<String>::new()));
    
    // Clones for the server endpoint
    let game_state_clone = game_state.clone();
    let enemy_states_clone = enemy_states.clone();
    let last_game_time_clone = last_game_time.clone();
    let enemy_team_heroes_clone = enemy_team_heroes.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::content_length_limit(1024 * 1024 * 10))
        .and(warp::body::json())
        .map(move |data: Value| {
            // Parse the incoming JSON
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Get current game time
                    let current_game_time = state.map.as_ref()
                        .and_then(|m| m.game_time)
                        .unwrap_or(0);
                    
                    // Check if this is a new game time to avoid processing duplicates
                    {
                        let mut last_time = last_game_time_clone.lock().unwrap();
                        if *last_time == current_game_time {
                            return "OK";
                        }
                        *last_time = current_game_time;
                    }
                    
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
                    
                    // Get player position for relative directions
                    let player_position = if let Some(hero) = &state.hero {
                        match (hero.xpos, hero.ypos) {
                            (Some(x), Some(y)) => Some((x, y)),
                            _ => None
                        }
                    } else {
                        None
                    };
                    
                    // Update enemy states with the collected data
                    {
                        let mut enemy_map = enemy_states_clone.lock().unwrap();
                        
                        // First mark all enemies as potentially lost
                        for (_, enemy) in enemy_map.iter_mut() {
                            if enemy.status != EnemyStatus::Lost && current_game_time - enemy.last_seen_time > 10 {
                                enemy.status = EnemyStatus::Lost;
                            }
                        }
                        
                        // Then update with current sightings
                        for (name, position) in visible_enemies {
                            let was_already_tracked = enemy_map.contains_key(&name);
                            let mut status = EnemyStatus::Tracking;
                            
                            if !was_already_tracked {
                                status = EnemyStatus::NewlySpotted;
                            } else if let Some(existing) = enemy_map.get(&name) {
                                if has_moved_significantly(existing.last_seen_position, position) {
                                    status = EnemyStatus::MovedSignificantly;
                                }
                            }
                            
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
                                status,
                            });
                            
                            // Add to enemy team heroes list if not already there
                            let mut enemy_heroes = enemy_team_heroes_clone.lock().unwrap();
                            if !enemy_heroes.contains(&name) {
                                enemy_heroes.push(name.clone());
                                
                                // Print updated enemy team list whenever we discover a new hero
                                println!("\n[{}] {}: {} spotted for the first time. Now tracking {} enemies:", 
                                    format_game_time(Some(current_game_time)),
                                    "ENEMY HERO DISCOVERED".magenta().bold(),
                                    name.yellow().bold(),
                                    enemy_heroes.len());
                                    
                                for (i, hero_name) in enemy_heroes.iter().enumerate() {
                                    println!("  {}. {}", i+1, hero_name.yellow());
                                }
                                println!();
                            }
                        }
                        
                        // Process enemy states to generate text updates
                        if player_position.is_some() {
                            for (name, enemy) in enemy_map.iter() {
                                let time_str = format_game_time(Some(current_game_time));
                                
                                match enemy.status {
                                    EnemyStatus::NewlySpotted => {
                                        let location = if let Some(pos) = player_position {
                                            describe_position_relative_to_player(pos, enemy.last_seen_position)
                                        } else {
                                            describe_map_location(enemy.last_seen_position)
                                        };
                                        
                                        println!("[{}] {}: {} {} (Level {}) spotted {}", 
                                            time_str,
                                            "ENEMY SPOTTED".red().bold(),
                                            name.yellow().bold(),
                                            if enemy.times_spotted > 1 { "reappeared" } else { "appeared" },
                                            enemy.estimated_level,
                                            location);
                                    },
                                    EnemyStatus::MovedSignificantly => {
                                        if let Some(pos) = player_position {
                                            let location = describe_position_relative_to_player(pos, enemy.last_seen_position);
                                            println!("[{}] {}: {} is moving, now {}", 
                                                time_str,
                                                "ENEMY MOVEMENT".yellow(),
                                                name.yellow(),
                                                location);
                                        }
                                    },
                                    EnemyStatus::Lost => {
                                        println!("[{}] {}: Lost track of {}, last seen {} seconds ago", 
                                            time_str,
                                            "ENEMY MISSING".blue(),
                                            name,
                                            current_game_time - enemy.last_seen_time);
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                    // Check for low health buildings
                    if let Some(buildings) = &state.buildings {
                        let enemy_team_key = if player_team == "radiant" { "dire" } else { "radiant" };
                        
                        if let Some(enemy_buildings) = buildings.get(enemy_team_key) {
                            let time_str = format_game_time(Some(current_game_time));
                            
                            for (name, building) in enemy_buildings {
                                let health_percent = (building.health as f32 / building.max_health as f32 * 100.0) as i32;
                                
                                // Only alert for low health buildings
                                if health_percent <= 30 {
                                    // Format building name for better readability
                                    let building_name = name.replace("dota_goodguys_", "")
                                        .replace("dota_badguys_", "")
                                        .replace("_", " ");
                                    
                                    println!("[{}] {}: Enemy {} at {}% health", 
                                        time_str,
                                        "OBJECTIVE".green().bold(),
                                        building_name.green(),
                                        health_percent);
                                }
                            }
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
    let _server_thread = tokio::spawn(async move {
        warp::serve(gsi_endpoint)
            .run(([127, 0, 0, 1], 3000))
            .await;
    });
    
    println!("{}", "Server running! Waiting for Dota 2 data...".yellow());
    println!("{}", "Make sure you have configured the GSI config file in Dota 2.".yellow());
    println!("{}", "Add -gamestateintegration to Dota 2 launch options".yellow());
    println!();
    println!("{}", "Enemy activity will stream below as it happens...".green());
    println!("{}", "======================================================".green());
    
    // Print the current enemy team composition command
    // Periodically display enemy team composition
    let enemy_team_heroes_display = enemy_team_heroes.clone();
    let last_time_clone = last_game_time.clone();
    tokio::spawn(async move {
        let mut last_display_time = 0;
        
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await; // Display every minute
            
            // Get current game time
            let current_time = *last_time_clone.lock().unwrap();
            
            // Only display if game time has progressed and it's been at least a minute since last display
            if current_time > 0 && current_time > last_display_time + 60 {
                let heroes = enemy_team_heroes_display.lock().unwrap();
                if !heroes.is_empty() {
                    println!("\n[{}] {}: ", 
                        format_game_time(Some(current_time)),
                        "ENEMY TEAM SUMMARY".cyan().bold());
                    
                    for (i, hero) in heroes.iter().enumerate() {
                        println!("  {}. {}", i+1, hero.yellow());
                    }
                    println!();
                    
                    last_display_time = current_time;
                }
            }
        }
    });
    
    // Keep main thread alive
    println!("Press Ctrl+C to exit");
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("Shutting down server..."),
        Err(err) => eprintln!("Error listening for Ctrl+C: {}", err),
    }
}