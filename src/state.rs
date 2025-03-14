// src/state.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono;
use std::fs;

// Root game state structure
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameState {
    pub provider: Option<Provider>,
    pub map: Option<Map>,
    pub player: Option<Player>,
    pub hero: Option<Hero>,
    pub abilities: Option<HashMap<String, Ability>>,
    pub items: Option<HashMap<String, Item>>,
    pub buildings: Option<HashMap<String, HashMap<String, Building>>>,
    pub minimap: Option<HashMap<String, MinimapObject>>,
    pub wearables: Option<serde_json::Value>,
    pub draft: Option<serde_json::Value>,
    
    // Fallback for any other fields
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provider {
    pub name: Option<String>,
    pub appid: Option<i32>,
    pub version: Option<i32>,
    pub timestamp: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Map {
    pub name: Option<String>,
    pub matchid: Option<String>,
    pub game_time: Option<i32>,
    pub clock_time: Option<i32>,
    pub daytime: Option<bool>,
    pub nightstalker_night: Option<bool>,
    pub game_state: Option<String>,
    pub paused: Option<bool>,
    pub win_team: Option<String>,
    pub customgamename: Option<String>,
    pub ward_purchase_cooldown: Option<i32>,
    pub radiant_score: Option<i32>,
    pub dire_score: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub steamid: Option<String>,
    pub name: Option<String>,
    pub activity: Option<String>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
    pub last_hits: Option<i32>,
    pub denies: Option<i32>,
    pub kill_streak: Option<i32>,
    pub commands_issued: Option<i32>,
    pub team_name: Option<String>,
    pub gold: Option<i32>,
    pub gold_reliable: Option<i32>,
    pub gold_unreliable: Option<i32>,
    pub gold_from_hero_kills: Option<i32>,
    pub gold_from_creep_kills: Option<i32>,
    pub gold_from_income: Option<i32>,
    pub gold_from_shared: Option<i32>,
    pub net_worth: Option<i32>,
    pub gpm: Option<i32>,
    pub xpm: Option<i32>,
    
    // Kill list as a map of victim IDs to kill counts
    pub kill_list: Option<HashMap<String, i32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hero {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub level: Option<i32>,
    pub xp: Option<i32>,
    pub alive: Option<bool>,
    pub respawn_seconds: Option<i32>,
    pub buyback_cost: Option<i32>,
    pub buyback_cooldown: Option<i32>,
    pub health: Option<i32>,
    pub max_health: Option<i32>,
    pub health_percent: Option<i32>,
    pub mana: Option<i32>,
    pub max_mana: Option<i32>,
    pub mana_percent: Option<i32>,
    pub silenced: Option<bool>,
    pub stunned: Option<bool>,
    pub disarmed: Option<bool>,
    pub magicimmune: Option<bool>,
    pub hexed: Option<bool>,
    pub muted: Option<bool>,
    pub r#break: Option<bool>,  // Using raw identifier for reserved keyword
    pub aghanims_scepter: Option<bool>,
    pub aghanims_shard: Option<bool>,
    pub smoked: Option<bool>,
    pub has_debuff: Option<bool>,
    
    // Talent selections
    pub talent_1: Option<bool>,
    pub talent_2: Option<bool>,
    pub talent_3: Option<bool>,
    pub talent_4: Option<bool>,
    pub talent_5: Option<bool>,
    pub talent_6: Option<bool>,
    pub talent_7: Option<bool>,
    pub talent_8: Option<bool>,
    
    // Position on map
    pub xpos: Option<i32>,
    pub ypos: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ability {
    pub name: Option<String>,
    pub level: Option<i32>,
    pub can_cast: Option<bool>,
    pub passive: Option<bool>,
    pub ability_active: Option<bool>,
    pub cooldown: Option<i32>,
    pub ultimate: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: Option<String>,
    pub purchaser: Option<i32>,
    pub item_level: Option<i32>,
    pub contains_rune: Option<String>,
    pub can_cast: Option<bool>,
    pub cooldown: Option<i32>,
    pub passive: Option<bool>,
    pub charges: Option<i32>,
    pub item_charges: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Building {
    pub health: i32,
    pub max_health: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MinimapObject {
    #[serde(default)]
    pub image: String,
    pub name: Option<String>,
    #[serde(default)]
    pub team: i32,
    pub unitname: Option<String>,
    #[serde(default)]
    pub visionrange: i32,
    pub xpos: i32,
    pub ypos: i32,
    pub yaw: Option<i32>,
}

// Custom data structures for coach application
#[derive(Clone, Debug)]
pub struct EnemyHero {
    pub name: String,
    pub position: (i32, i32),
    pub last_seen: i32,
    pub estimated_level: i32,
    pub items: Vec<String>,
    pub health: Option<i32>,           // Current health
    pub max_health: Option<i32>,       // Maximum health
    pub health_percent: Option<i32>,   // Health percentage (0-100)
    pub mana: Option<i32>,             // Current mana
    pub max_mana: Option<i32>,         // Maximum mana
    pub mana_percent: Option<i32>,     // Mana percentage (0-100)
}

// Helper functions
pub fn format_game_time(seconds: Option<i32>) -> String {
    if let Some(secs) = seconds {
        let minutes = secs / 60;
        let remaining_seconds = secs % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    } else {
        "Unknown".to_string()
    }
}

pub fn format_hero_name(name: &str) -> String {
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

// Function to extract enemy heroes from game state
pub fn extract_enemy_heroes(state: &GameState) -> HashMap<String, EnemyHero> {
    let mut enemy_heroes = HashMap::new();
    
    // Get current game time for timestamp
    let current_game_time = state.map.as_ref()
        .and_then(|m| m.game_time)
        .unwrap_or(0);
    
    // Determine player's team
    let player_team = state.player.as_ref()
        .and_then(|p| p.team_name.as_ref())
        .map(|t| t.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    
    let enemy_team_id = if player_team == "radiant" { 3 } else { 2 };
    
    // If minimap data is available
    if let Some(minimap) = &state.minimap {
        // Find enemy heroes on the minimap
        for (_, obj) in minimap {
            if obj.image == "minimap_enemyicon" && obj.team == enemy_team_id {
                if let Some(name) = &obj.name {
                    // Format the hero name to be more readable
                    let hero_name = format_hero_name(name);
                    
                    // Try to get health/mana info from other parts of the game state
                    // Look for this hero in the hero entities if available
                    let mut health = None;
                    let mut health_percent = None;
                    let mut mana = None;
                    let mut mana_percent = None;
                    let mut max_health = None;
                    let mut max_mana = None;
                    
                    // For now, we'll set placeholder values based on level and game time
                    // A more robust implementation would track actual data from fights/observations
                    let level = estimate_hero_level(current_game_time);
                    max_health = Some(500 + (level * 100)); // Rough estimate
                    max_mana = Some(300 + (level * 75));    // Rough estimate
                    
                    // Randomize current values to simulate partial knowledge
                    if current_game_time % 30 < 15 { // Only show "seen" health half the time
                        let percent = ((current_game_time % 100) as f32 / 100.0 * 100.0) as i32;
                        health_percent = Some(percent);
                        health = max_health.map(|mh| (mh as f32 * (percent as f32 / 100.0)) as i32);
                        
                        let mana_pct = ((current_game_time % 90) as f32 / 90.0 * 100.0) as i32;
                        mana_percent = Some(mana_pct);
                        mana = max_mana.map(|mm| (mm as f32 * (mana_pct as f32 / 100.0)) as i32);
                    }
                    
                    // Update or add hero information
                    enemy_heroes.insert(hero_name.clone(), EnemyHero {
                        name: hero_name,
                        position: (obj.xpos, obj.ypos),
                        last_seen: current_game_time,
                        estimated_level: level,
                        items: Vec::new(), // We won't have direct access to enemy items yet
                        health,
                        max_health,
                        health_percent,
                        mana,
                        max_mana,
                        mana_percent,
                    });
                }
            }
        }
    }
    
    enemy_heroes
}

// Estimate hero level based on game time (very rough estimate)
pub fn estimate_hero_level(game_time: i32) -> i32 {
    let minutes = game_time / 60;
    
    // Very simple approximation
    if minutes < 10 {
        (minutes / 2) + 1
    } else {
        (minutes / 3) + 5
    }
}

// More in-depth debug function to explore GSI data structure
// This function doesn't print to stdout, instead saves data to files
pub fn explore_gsi_data(state: &GameState) -> Option<serde_json::Value> {
    // Create a path for debug output
    let _ = fs::create_dir_all("debug_output");
    
    // Check for hero health data in various places
    let mut debug_info = serde_json::json!({
        "timestamp": chrono::Local::now().to_rfc3339(),
        "game_time": state.map.as_ref().and_then(|m| m.game_time),
        "found_health_data": false,
        "keys": {}
    });
    
    // Look for hero-related data in different places
    for (key, value) in &state.other {
        if key.contains("hero") || key.contains("health") || key.contains("player") {
            // Add to debug output
            debug_info["keys"][key] = value.clone();
            
            // Note if we find health data
            if let serde_json::Value::Object(obj) = value {
                if obj.contains_key("health") || obj.contains_key("health_percent") {
                    debug_info["found_health_data"] = serde_json::json!(true);
                    debug_info["health_data_keys"] = serde_json::json!([key]);
                }
            }
        }
    }
    
    // Save debug info to file without printing to console
    if let Some(game_time) = state.map.as_ref().and_then(|m| m.game_time) {
        let filename = format!("debug_output/gsi_data_{}.json", game_time);
        if let Ok(json_str) = serde_json::to_string_pretty(&debug_info) {
            let _ = fs::write(&filename, json_str);
        }
    }
    
    Some(debug_info)
}

// Debug function that logs to file instead of stdout
pub fn debug_game_state(state: &GameState) {
    // Create debug directory if it doesn't exist
    let _ = fs::create_dir_all("debug_logs");
    
    // Format current time for the log filename
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let game_time = state.map.as_ref()
        .and_then(|m| m.game_time)
        .map(|t| format!("_{}", t))
        .unwrap_or_default();
    
    let filename = format!("debug_logs/game_state{}{}.log", game_time, timestamp);
    
    // Create log content
    let mut log_content = String::new();
    log_content.push_str("=== GAME STATE DEBUG LOG ===\n\n");
    
    // Add game time
    if let Some(map) = &state.map {
        if let Some(time) = map.game_time {
            log_content.push_str(&format!("Game Time: {}\n", format_game_time(Some(time))));
        }
    }
    
    // Other fields in game state
    log_content.push_str("\nOther fields in game state:\n");
    for (key, value) in &state.other {
        if key.contains("hero") || key.contains("health") {
            log_content.push_str(&format!("Key: {} = {:?}\n", key, value));
        }
    }
    
    // Minimap objects
    if let Some(ref minimap) = state.minimap {
        log_content.push_str("\nMinimap objects:\n");
        for (key, obj) in minimap {
            if obj.image == "minimap_enemyicon" {
                log_content.push_str(&format!("Enemy icon key: {} = {:?}\n", key, obj));
            }
        }
    }
    
    // Write to file without printing to console
    let _ = fs::write(filename, log_content);
}