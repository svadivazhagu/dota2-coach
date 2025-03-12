// src/bin/coach.rs
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
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
    name: Option<String>,
    level: Option<i32>,
    health: Option<i32>,
    max_health: Option<i32>,
    mana: Option<i32>,
    max_mana: Option<i32>,
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

#[tokio::main]
async fn main() {
    println!("{}", "Dota 2 Coach - Real-time GSI Data Feed".green().bold());
    println!("{}", "-------------------------------------".green());
    println!("Starting GSI server on port 3000...");
    
    // Create a shared game state that can be accessed from different threads
    let game_state = Arc::new(Mutex::new(None::<GameState>));
    let game_state_clone = game_state.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::json())
        .map(move |data: Value| {
            // Convert the incoming JSON to our GameState struct
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Store the game state
                    let mut gs = game_state_clone.lock().unwrap();
                    *gs = Some(state);
                    
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
    
    // Create a thread for the display logic
    let display_thread = thread::spawn(move || {
        let game_state_ref = game_state;
        let mut last_update_time = 0;
        
        loop {
            // Try to get the current game state
            let state_option = {
                let state = game_state_ref.lock().unwrap();
                state.clone()
            };
            
            if let Some(state) = state_option {
                // Get current timestamp to check if we have a new update
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();
                    
                // Only print if we have a new update (prevents spamming the console)
                if current_time > last_update_time {
                    last_update_time = current_time;
                    
                    // Clear the console
                    print!("\x1B[2J\x1B[1;1H");
                    
                    println!("{}", "Dota 2 Coach - Live Game Data".green().bold());
                    println!("{}", "-------------------------------------".green());
                    
                    // Display basic game info
                    if let Some(map) = &state.map {
                        println!("Game Time: {}", format_game_time(map.game_time));
                        println!("Game State: {}", map.game_state.as_deref().unwrap_or("Unknown"));
                        
                        // Show team scores if available
                        println!("Score: {} Radiant - {} Dire", 
                            map.radiant_score.unwrap_or(0).to_string().green(), 
                            map.dire_score.unwrap_or(0).to_string().red());
                        
                        println!("-------------------------------------");
                    }
                    
                    // Display player info if available
                    if let Some(player) = &state.player {
                        println!("{}", "Player Information:".yellow().bold());
                        
                        println!("Name: {}", player.name.as_deref().unwrap_or("Unknown"));
                        
                        let team_name = player.team_name.as_deref().unwrap_or("Unknown");
                        let colored_team = match team_name.to_lowercase().as_str() {
                            "radiant" => team_name.green(),
                            "dire" => team_name.red(),
                            _ => team_name.normal(),
                        };
                        println!("Team: {}", colored_team);
                        
                        println!("Gold: {}", player.gold.unwrap_or(0));
                        println!("GPM: {}", player.gpm.unwrap_or(0));
                        println!("XPM: {}", player.xpm.unwrap_or(0));
                        
                        println!("-------------------------------------");
                    }
                    
                    // Display hero information
                    if let Some(hero) = &state.hero {
                        println!("{}", "Hero Information:".yellow().bold());
                        
                        println!("Hero: {}", hero.name.as_deref().unwrap_or("Unknown"));
                        println!("Level: {}", hero.level.unwrap_or(0));
                        
                        let health = hero.health.unwrap_or(0);
                        let max_health = hero.max_health.unwrap_or(1);
                        let health_percent = (health as f32 / max_health as f32) * 100.0;
                        println!("Health: {}/{} ({:.1}%)", health, max_health, health_percent);
                        
                        let mana = hero.mana.unwrap_or(0);
                        let max_mana = hero.max_mana.unwrap_or(1);
                        let mana_percent = (mana as f32 / max_mana as f32) * 100.0;
                        println!("Mana: {}/{} ({:.1}%)", mana, max_mana, mana_percent);
                        
                        println!("-------------------------------------");
                    }
                    
                    // Display items if available
                    if let Some(items) = &state.items {
                        println!("{}", "Inventory:".yellow().bold());
                        
                        // Display main inventory items
                        for i in 0..6 {
                            let slot_name = format!("slot{}", i);
                            if let Some(item) = items.get(&slot_name) {
                                println!("Slot {}: {}", i, item.name.as_deref().unwrap_or("Unknown Item"));
                            } else {
                                println!("Slot {}: Empty", i);
                            }
                        }
                        
                        // Display backpack items
                        println!("\n{}", "Backpack:".yellow());
                        for i in 0..3 {
                            let slot_name = format!("backpack{}", i);
                            if let Some(item) = items.get(&slot_name) {
                                println!("Backpack {}: {}", i, item.name.as_deref().unwrap_or("Unknown Item"));
                            } else {
                                println!("Backpack {}: Empty", i);
                            }
                        }
                        
                        println!("-------------------------------------");
                    }
                    
                    // Display abilities if available
                    if let Some(abilities) = &state.abilities {
                        println!("{}", "Abilities:".yellow().bold());
                        
                        for (key, ability) in abilities.iter() {
                            if key.starts_with("ability") {
                                let status = if ability.can_cast.unwrap_or(false) { 
                                    "Ready".green() 
                                } else if ability.cooldown.is_some() && ability.cooldown.unwrap() > 0 { 
                                    format!("CD: {}s", ability.cooldown.unwrap()).red() 
                                } else { 
                                    "Not Ready".red() 
                                };
                                
                                println!("{}: Level {}, {}", 
                                    ability.name.as_deref().unwrap_or("Unknown"),
                                    ability.level.unwrap_or(0),
                                    status
                                );
                            }
                        }
                        
                        println!("-------------------------------------");
                    }
                    
                    println!("\nPress Ctrl+C to exit");
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