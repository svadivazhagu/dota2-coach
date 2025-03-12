// src/bin/coach.rs
use std::sync::{Arc, Mutex};
use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use colored::Colorize;
use std::{collections::HashMap, thread};
use std::time::Duration;
use chrono::Local;

// Basic game state structure
#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameState {
    provider: Option<Provider>,
    map: Option<Map>,
    player: Option<Player>,
    hero: Option<Hero>,
    abilities: Option<HashMap<String, Ability>>,
    items: Option<HashMap<String, Item>>,
    minimap: Option<HashMap<String, MinimapObject>>,
    
    // Fallback for any other fields
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Provider {
    name: Option<String>,
    timestamp: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Map {
    name: Option<String>,
    game_time: Option<i32>,
    game_state: Option<String>,
    radiant_score: Option<i32>,
    dire_score: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    name: Option<String>,
    team_name: Option<String>,
    gold: Option<i32>,
    gpm: Option<i32>,
    xpm: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Hero {
    id: Option<i32>,
    name: Option<String>,
    level: Option<i32>,
    health: Option<i32>,
    max_health: Option<i32>,
    health_percent: Option<i32>,
    mana: Option<i32>,
    max_mana: Option<i32>,
    mana_percent: Option<i32>,
    xpos: Option<i32>,
    ypos: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Ability {
    name: Option<String>,
    level: Option<i32>,
    can_cast: Option<bool>,
    cooldown: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Item {
    name: Option<String>,
    purchaser: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MinimapObject {
    image: String,
    name: Option<String>,
    team: i32,
    unitname: String,
    visionrange: i32,
    xpos: i32,
    ypos: i32,
    yaw: Option<i32>,
}

// Structure to track enemy hero information
#[derive(Clone, Debug)]
struct EnemyHero {
    name: String,
    position: (i32, i32),
    last_seen: i32, // game time when last seen
}

#[tokio::main]
async fn main() {
    println!("{}", "Dota 2 Coach - Real-time GSI Data Feed".green().bold());
    println!("{}", "-------------------------------------".green());
    println!("Starting GSI server on port 3000...");
    
    // Create a shared game state that can be accessed from different threads
    let game_state = Arc::new(Mutex::new(None::<GameState>));
    let enemy_heroes = Arc::new(Mutex::new(HashMap::<String, EnemyHero>::new()));
    
    let game_state_clone = game_state.clone();
    let enemy_heroes_clone = enemy_heroes.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::content_length_limit(1024 * 1024 * 10))
        .and(warp::body::json())
        .map(move |data: Value| {
            // Convert the incoming JSON to our GameState struct
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Store the game state
                    let mut gs = game_state_clone.lock().unwrap();
                    *gs = Some(state.clone());
                    
                    // Update enemy hero information
                    if let Some(minimap) = &state.minimap {
                        let current_game_time = state.map.as_ref()
                            .and_then(|m| m.game_time)
                            .unwrap_or(0);
                        
                        let mut heroes = enemy_heroes_clone.lock().unwrap();
                        
                        // Determine player's team
                        let player_team = state.player.as_ref()
                            .and_then(|p| p.team_name.as_ref())
                            .map(|t| t.to_lowercase())
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        let enemy_team_id = if player_team == "radiant" { 3 } else { 2 };
                        
                        // Find enemy heroes on the minimap
                        for (id, obj) in minimap {
                            if obj.image == "minimap_enemyicon" && obj.team == enemy_team_id {
                                if let Some(name) = &obj.name {
                                    // Format the hero name to be more readable
                                    let hero_name = format_hero_name(name);
                                    
                                    // Update or add hero information
                                    heroes.insert(hero_name.clone(), EnemyHero {
                                        name: hero_name,
                                        position: (obj.xpos, obj.ypos),
                                        last_seen: current_game_time,
                                    });
                                }
                            }
                        }
                    }
                    
                    // Log timestamp of update
                    let time = Local::now().format("%H:%M:%S").to_string();
                    println!("[{}] Received game state update", time);
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
    
    println!("{}", "Server running! Waiting for Dota 2 game data...".yellow());
    println!("{}", "Make sure you have configured the GSI config file in Dota 2.".yellow());
    println!("{}", "Add -gamestateintegration to Dota 2 launch options".yellow());
    
    // Create a thread for the display logic
    let display_thread = thread::spawn(move || {
        let game_state_ref = game_state;
        let enemy_heroes_ref = enemy_heroes;
        let mut last_game_time = -1;
        
        loop {
            // Try to get the current game state
            let state_option = {
                let state = game_state_ref.lock().unwrap();
                state.clone()
            };
            
            let enemy_heroes_info = {
                let heroes = enemy_heroes_ref.lock().unwrap();
                heroes.clone()
            };
            
            if let Some(state) = state_option {
                // Only update display if game time has changed
                let game_time = state.map.as_ref().and_then(|m| m.game_time).unwrap_or(-1);
                
                if game_time != last_game_time {
                    // Clear screen and display updated info
                    print!("\x1B[2J\x1B[1;1H");  // Clear screen and move cursor to home
                    
                    println!("{}", "Dota 2 Coach - Enemy Team Monitor".green().bold());
                    println!("{}", "-------------------------------------".green());
                    
                    if let Some(map) = &state.map {
                        println!("Game Time: {}", format_game_time(map.game_time));
                        println!("Game State: {}", map.game_state.as_deref().unwrap_or("Unknown"));
                        
                        // Show team scores if available
                        println!("Score: {} Radiant - {} Dire", 
                            map.radiant_score.unwrap_or(0).to_string().green(), 
                            map.dire_score.unwrap_or(0).to_string().red());
                        
                        println!("-------------------------------------");
                    }
                    
                    // Show player info
                    if let Some(player) = &state.player {
                        println!("Player: {}", player.name.as_deref().unwrap_or("Unknown"));
                        println!("Team: {}", match player.team_name.as_deref() {
                            Some("radiant") => "Radiant".green(),
                            Some("dire") => "Dire".red(),
                            _ => "Unknown".normal()
                        });
                        println!("-------------------------------------");
                    }
                    
                    // Display enemy hero information
                    println!("{}", "Enemy Heroes:".red().bold());
                    
                    if enemy_heroes_info.is_empty() {
                        println!("No enemy heroes detected yet");
                    } else {
                        for (_, hero) in enemy_heroes_info {
                            println!("Hero: {}", hero.name);
                            println!("  Position: ({}, {})", hero.position.0, hero.position.1);
                            println!("  Last seen: {}", format_game_time(Some(hero.last_seen)));
                            println!();
                        }
                    }
                    
                    println!("-------------------------------------");
                    
                    // Display helpful information about your hero
                    println!("{}", "Your Hero Information:".yellow().bold());
                    
                    // Show your hero information
                    if let Some(hero) = &state.hero {
                        println!("Your hero: {}, Level: {}", 
                            format_hero_name(&hero.name.as_deref().unwrap_or("Unknown").to_string()),
                            hero.level.unwrap_or(0));
                        
                        // Show HP percentage
                        if let Some(health_pct) = hero.health_percent {
                            println!("Health: {}%", health_pct);
                        }
                        
                        // Show mana percentage
                        if let Some(mana_pct) = hero.mana_percent {
                            println!("Mana: {}%", mana_pct);
                        }
                    }
                    
                    println!("\nPress Ctrl+C to exit");
                    
                    // Update last game time
                    last_game_time = game_time;
                }
            }
            
            // Sleep for a short time to avoid excessive CPU usage
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    // Wait for the threads to complete (they won't in normal operation)
    display_thread.join().unwrap();
    server_thread.abort();
}

// Helper function to format game time
fn format_game_time(seconds: Option<i32>) -> String {
    if let Some(secs) = seconds {
        let minutes = secs / 60;
        let remaining_seconds = secs % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    } else {
        "Unknown".to_string()
    }
}

// Helper function to format hero names
fn format_hero_name(name: &str) -> String {
    // Convert "npc_dota_hero_bounty_hunter" to "Bounty Hunter"
    let name = name.replace("npc_dota_hero_", "");
    
    // Split by underscore and capitalize each word
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