// src/bin/coach.rs
use std::sync::{Arc, Mutex};
use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use colored::Colorize;
use std::{collections::HashMap, thread};
use std::time::Duration;
use chrono::Local;
use std::fs;
use std::io::Write;

// Root game state structure
#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameState {
    provider: Option<Provider>,
    map: Option<Map>,
    player: Option<Player>,
    hero: Option<Hero>,
    abilities: Option<HashMap<String, Ability>>,
    items: Option<HashMap<String, Item>>,
    buildings: Option<HashMap<String, HashMap<String, Building>>>,
    minimap: Option<HashMap<String, MinimapObject>>,
    wearables: Option<Value>,
    draft: Option<Value>,
    
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
    clock_time: Option<i32>,
    daytime: Option<bool>,
    nightstalker_night: Option<bool>,
    game_state: Option<String>,
    paused: Option<bool>,
    win_team: Option<String>,
    customgamename: Option<String>,
    ward_purchase_cooldown: Option<i32>,
    radiant_score: Option<i32>,
    dire_score: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    steamid: Option<String>,
    name: Option<String>,
    activity: Option<String>,
    kills: Option<i32>,
    deaths: Option<i32>,
    assists: Option<i32>,
    last_hits: Option<i32>,
    denies: Option<i32>,
    kill_streak: Option<i32>,
    commands_issued: Option<i32>,
    team_name: Option<String>,
    gold: Option<i32>,
    gold_reliable: Option<i32>,
    gold_unreliable: Option<i32>,
    gold_from_hero_kills: Option<i32>,
    gold_from_creep_kills: Option<i32>,
    gold_from_income: Option<i32>,
    gold_from_shared: Option<i32>,
    net_worth: Option<i32>,
    gpm: Option<i32>,
    xpm: Option<i32>,
    
    // Kill list as a map of victim IDs to kill counts
    kill_list: Option<HashMap<String, i32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Hero {
    id: Option<i32>,
    name: Option<String>,
    level: Option<i32>,
    xp: Option<i32>,
    alive: Option<bool>,
    respawn_seconds: Option<i32>,
    buyback_cost: Option<i32>,
    buyback_cooldown: Option<i32>,
    health: Option<i32>,
    max_health: Option<i32>,
    health_percent: Option<i32>,
    mana: Option<i32>,
    max_mana: Option<i32>,
    mana_percent: Option<i32>,
    silenced: Option<bool>,
    stunned: Option<bool>,
    disarmed: Option<bool>,
    magicimmune: Option<bool>,
    hexed: Option<bool>,
    muted: Option<bool>,
    r#break: Option<bool>,  // Fixed: Use raw identifier for reserved keyword
    aghanims_scepter: Option<bool>,
    aghanims_shard: Option<bool>,
    smoked: Option<bool>,
    has_debuff: Option<bool>,
    
    // Talent selections
    talent_1: Option<bool>,
    talent_2: Option<bool>,
    talent_3: Option<bool>,
    talent_4: Option<bool>,
    talent_5: Option<bool>,
    talent_6: Option<bool>,
    talent_7: Option<bool>,
    talent_8: Option<bool>,
    
    // Position on map
    xpos: Option<i32>,
    ypos: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Ability {
    name: Option<String>,
    level: Option<i32>,
    can_cast: Option<bool>,
    passive: Option<bool>,
    ability_active: Option<bool>,
    cooldown: Option<i32>,
    ultimate: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Item {
    name: Option<String>,
    purchaser: Option<i32>,
    item_level: Option<i32>,
    contains_rune: Option<String>,
    can_cast: Option<bool>,
    cooldown: Option<i32>,
    passive: Option<bool>,
    charges: Option<i32>,
    item_charges: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Building {
    health: i32,
    max_health: i32,
}

// Update the MinimapObject struct to make fields optional
#[derive(Clone, Debug, Deserialize, Serialize)]
struct MinimapObject {
    #[serde(default)]
    image: String,
    name: Option<String>,
    #[serde(default)]
    team: i32,
    unitname: Option<String>,  // Changed from String to Option<String>
    #[serde(default)]
    visionrange: i32,
    xpos: i32,
    ypos: i32,
    yaw: Option<i32>,
}

// Coach-specific structures

// Structure to track enemy hero information with more details
#[derive(Clone, Debug)]
struct EnemyHero {
    name: String,
    position: (i32, i32),
    last_seen: i32,
    estimated_level: i32,
    items: Vec<String>,
}

// Structure to track important game events
#[derive(Clone, Debug)]
struct GameEvent {
    time: i32,
    event_type: String,
    description: String,
}

// Structure to track hero performance metrics
struct HeroPerformanceTracker {
    gpm_samples: Vec<(i32, i32)>,    // (game_time, gpm)
    xpm_samples: Vec<(i32, i32)>,    // (game_time, xpm)
    last_hits_samples: Vec<(i32, i32)>,  // (game_time, last_hits)
    last_death_time: i32,            // Game time of last death
    death_count: i32,                // Number of deaths
}

impl HeroPerformanceTracker {
    fn new() -> Self {
        Self {
            gpm_samples: Vec::new(),
            xpm_samples: Vec::new(),
            last_hits_samples: Vec::new(),
            last_death_time: 0,
            death_count: 0,
        }
    }
    
    fn update(&mut self, state: &GameState, last_state: &Option<GameState>) {
        // Get current game time
        let game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Update GPM and XPM
        if let Some(player) = &state.player {
            if let Some(gpm) = player.gpm {
                self.gpm_samples.push((game_time, gpm));
                // Keep only recent samples
                if self.gpm_samples.len() > 20 {
                    self.gpm_samples.remove(0);
                }
            }
            
            if let Some(xpm) = player.xpm {
                self.xpm_samples.push((game_time, xpm));
                if self.xpm_samples.len() > 20 {
                    self.xpm_samples.remove(0);
                }
            }
            
            if let Some(last_hits) = player.last_hits {
                self.last_hits_samples.push((game_time, last_hits));
                if self.last_hits_samples.len() > 20 {
                    self.last_hits_samples.remove(0);
                }
            }
        }
        
        // Detect deaths
        if let (Some(current_hero), Some(last_state_unwrapped)) = (state.hero.as_ref(), last_state.as_ref()) {
            if let Some(last_hero) = &last_state_unwrapped.hero {
                let was_alive = last_hero.alive.unwrap_or(true);
                let is_alive = current_hero.alive.unwrap_or(true);
                
                if was_alive && !is_alive {
                    self.last_death_time = game_time;
                    self.death_count += 1;
                }
            }
        }
    }
    
    fn display_metrics(&self, game_time: i32) -> Vec<String> {
        let mut result = Vec::new();
        
        // GPM trend
        if self.gpm_samples.len() >= 2 {
            let current_gpm = self.gpm_samples.last().unwrap().1;
            let avg_gpm = self.gpm_samples.iter()
                .map(|(_, gpm)| gpm)
                .sum::<i32>() / self.gpm_samples.len() as i32;
            
            result.push(format!("  GPM: {} (Avg: {})", current_gpm, avg_gpm));
            
            if current_gpm > avg_gpm + 100 {
                result.push("    ✅ GPM trending up significantly!".to_string());
            } else if current_gpm > avg_gpm + 20 {
                result.push("    ✅ GPM trending up".to_string());
            } else if current_gpm < avg_gpm - 100 {
                result.push("    ⚠️ GPM trending down significantly".to_string());
            } else if current_gpm < avg_gpm - 20 {
                result.push("    ⚠️ GPM trending down".to_string());
            }
        }
        
        // XPM trend
        if self.xpm_samples.len() >= 2 {
            let current_xpm = self.xpm_samples.last().unwrap().1;
            result.push(format!("  XPM: {}", current_xpm));
        }
        
        // Last hit efficiency
        if self.last_hits_samples.len() >= 2 {
            let current_last_hits = self.last_hits_samples.last().unwrap().1;
            let minutes = game_time / 60;
            
            if minutes > 0 {
                let cs_per_min = current_last_hits as f32 / minutes as f32;
                result.push(format!("  CS/min: {:.1}", cs_per_min));
                
                // Benchmark CS/min (very rough benchmarks)
                if minutes < 10 {
                    if cs_per_min >= 7.0 {
                        result.push("    ✅ Excellent early game CS".to_string());
                    } else if cs_per_min >= 5.0 {
                        result.push("    ✅ Good early game CS".to_string());
                    } else if cs_per_min < 3.0 {
                        result.push("    ⚠️ Early game CS needs improvement".to_string());
                    }
                } else {
                    if cs_per_min >= 8.0 {
                        result.push("    ✅ Excellent CS".to_string());
                    } else if cs_per_min >= 6.0 {
                        result.push("    ✅ Good CS".to_string());
                    } else if cs_per_min < 4.0 {
                        result.push("    ⚠️ CS needs improvement".to_string());
                    }
                }
            }
        }
        
        // Death analysis
        if self.death_count > 0 {
            result.push(format!("  Deaths: {}", self.death_count));
            
            let minutes = game_time / 60;
            if minutes > 0 {
                let deaths_per_min = self.death_count as f32 / minutes as f32;
                
                if deaths_per_min > 0.2 {
                    result.push("    ⚠️ High death rate, play more cautiously".to_string());
                }
            }
            
            let time_since_last_death = game_time - self.last_death_time;
            if time_since_last_death > 300 {  // 5 minutes
                result.push(format!("    ✅ Good survival streak: {} minutes without dying", 
                    time_since_last_death / 60));
            }
        } else {
            result.push("  Deaths: 0 - Excellent survival!".to_string());
        }
        
        result
    }
}

// Structure to track enemy positions and movements
struct EnemyPositionTracker {
    positions: HashMap<String, Vec<(i32, (i32, i32))>>,  // Hero name -> [(game_time, (x, y))]
    last_game_time: i32,
}

impl EnemyPositionTracker {
    fn new() -> Self {
        Self {
            positions: HashMap::new(),
            last_game_time: 0,
        }
    }
    
    fn update(&mut self, state: &GameState) {
        // Get current game time
        let current_game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Don't update if we've already processed this time or if we went back in time
        if current_game_time <= self.last_game_time {
            return;
        }
        
        // Determine player's team
        let player_team = state.player.as_ref()
            .and_then(|p| p.team_name.as_ref())
            .map(|t| t.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        let enemy_team_id = if player_team == "radiant" { 3 } else { 2 };
        
        // If minimap data is available, extract enemy positions
        if let Some(minimap) = &state.minimap {
            for (_, obj) in minimap {
                if obj.image == "minimap_enemyicon" && obj.team == enemy_team_id {
                    if let Some(name) = &obj.name {
                        let hero_name = format_hero_name(name);
                        
                        // Add position to history - Fixed: Clone hero_name before entry
                        self.positions
                            .entry(hero_name.clone())
                            .or_insert_with(Vec::new)
                            .push((current_game_time, (obj.xpos, obj.ypos)));
                        
                        // Limit history size
                        let positions = self.positions.get_mut(&hero_name).unwrap();
                        if positions.len() > 100 {
                            positions.remove(0);
                        }
                    }
                }
            }
        }
        
        self.last_game_time = current_game_time;
    }
    
    fn predict_movements(&self, game_time: i32) -> Vec<(String, (i32, i32))> {
        let mut predictions = Vec::new();
        
        for (hero_name, positions) in &self.positions {
            if positions.len() < 2 {
                continue;
            }
            
            // Get the two most recent positions
            let latest = positions.last().unwrap();
            let second_last = positions[positions.len() - 2];
            
            // Calculate time since last seen
            let time_since_seen = game_time - latest.0;
            
            // Only predict if seen recently (within 30 seconds)
            if time_since_seen <= 30 {
                // Calculate movement vector
                let dx = latest.1.0 - second_last.1.0;
                let dy = latest.1.1 - second_last.1.1;
                
                // Calculate time between observations
                let dt = latest.0 - second_last.0;
                
                if dt > 0 {
                    // Calculate predicted position (simple linear extrapolation)
                    let time_factor = time_since_seen as f32 / dt as f32;
                    let pred_x = latest.1.0 + (dx as f32 * time_factor) as i32;
                    let pred_y = latest.1.1 + (dy as f32 * time_factor) as i32;
                    
                    predictions.push((hero_name.clone(), (pred_x, pred_y)));
                }
            }
        }
        
        predictions
    }
    
    fn get_movement_descriptions(&self, game_time: i32) -> Vec<String> {
        let mut result = Vec::new();
        
        for (hero_name, positions) in &self.positions {
            if positions.is_empty() {
                continue;
            }
            
            // Get the most recent position
            let latest = positions.last().unwrap();
            
            // Calculate time since last seen
            let time_since_seen = game_time - latest.0;
            
            // Only show if seen in the last 60 seconds
            if time_since_seen <= 60 {
                result.push(format!("  {}: Last seen {} seconds ago at ({}, {})", 
                    hero_name,
                    time_since_seen,
                    latest.1.0,
                    latest.1.1));
                
                // If we have multiple positions, analyze movement
                if positions.len() >= 2 {
                    let second_last = positions[positions.len() - 2];
                    
                    // Direction of movement (very simple approximation)
                    let dx = latest.1.0 - second_last.1.0;
                    let dy = latest.1.1 - second_last.1.1;
                    
                    let direction = if dx.abs() > dy.abs() {
                        if dx > 0 { "East" } else { "West" }
                    } else {
                        if dy > 0 { "North" } else { "South" }
                    };
                    
                    result.push(format!("    Moving {}", direction));
                }
            }
        }
        
        result
    }
}

// Structure to track team fight detection and analysis
struct TeamFightAnalyzer {
    player_deaths: Vec<i32>,          // Game times when player died
    enemy_deaths: HashMap<String, Vec<i32>>,  // Hero name -> [death times]
    last_kill_time: i32,              // Last time a kill happened
    team_fight_detected: bool,        // Whether a team fight is happening
    team_fight_start: i32,            // When the team fight started
}

impl TeamFightAnalyzer {
    fn new() -> Self {
        Self {
            player_deaths: Vec::new(),
            enemy_deaths: HashMap::new(),
            last_kill_time: 0,
            team_fight_detected: false,
            team_fight_start: 0,
        }
    }
    
    fn update(&mut self, state: &GameState, last_state: &Option<GameState>) {
        // Get current game time
        let game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Detect player death
        if let (Some(current_hero), Some(last_state_unwrapped)) = (state.hero.as_ref(), last_state.as_ref()) {
            if let Some(last_hero) = &last_state_unwrapped.hero {
                let was_alive = last_hero.alive.unwrap_or(true);
                let is_alive = current_hero.alive.unwrap_or(true);
                
                if was_alive && !is_alive {
                    self.player_deaths.push(game_time);
                    self.last_kill_time = game_time;
                }
            }
        }
        
        // Detect enemy deaths by looking at kill list changes
        if let (Some(current_player), Some(last_state_unwrapped)) = (state.player.as_ref(), last_state.as_ref()) {
            if let Some(last_player) = &last_state_unwrapped.player {
                if let (Some(current_kills), Some(last_kills)) = (
                    current_player.kill_list.as_ref(),
                    last_player.kill_list.as_ref()
                ) {
                    for (victim_id, kill_count) in current_kills {
                        let last_count = last_kills.get(victim_id).unwrap_or(&0);
                        
                        if kill_count > last_count {
                            // A new kill happened
                            let enemy_name = format!("Enemy{}", victim_id.replace("victimid_", ""));
                            self.enemy_deaths
                                .entry(enemy_name)
                                .or_insert_with(Vec::new)
                                .push(game_time);
                            
                            self.last_kill_time = game_time;
                        }
                    }
                }
            }
        }
        
        // Detect team fights (multiple kills in short succession)
        let time_since_last_kill = game_time - self.last_kill_time;
        
        if !self.team_fight_detected && time_since_last_kill < 15 {
            // Check if there were multiple kills in the last 30 seconds
            let kills_in_window = self.count_kills_in_window(game_time, 30);
            
            if kills_in_window >= 3 {
                self.team_fight_detected = true;
                self.team_fight_start = game_time;
            }
        } else if self.team_fight_detected && time_since_last_kill >= 15 {
            // Team fight has ended
            self.team_fight_detected = false;
        }
    }
    
    fn count_kills_in_window(&self, game_time: i32, window: i32) -> i32 {
        let window_start = game_time - window;
        
        // Count player deaths in window
        let player_deaths = self.player_deaths.iter()
            .filter(|time| **time >= window_start)
            .count() as i32;
        
        // Count enemy deaths in window
        let mut enemy_deaths = 0;
        for deaths in self.enemy_deaths.values() {
            enemy_deaths += deaths.iter()
                .filter(|time| **time >= window_start)
                .count() as i32;
        }
        
        player_deaths + enemy_deaths
    }
    
    fn get_team_fight_status(&self, game_time: i32) -> Vec<String> {
        let mut result = Vec::new();
        
        if self.team_fight_detected {
            result.push("⚔️ TEAM FIGHT IN PROGRESS!".to_string());
            result.push(format!("  Started {} seconds ago", game_time - self.team_fight_start));
        } else {
            // Check if a team fight might be coming soon
            let kills_in_window = self.count_kills_in_window(game_time, 60);
            
            if kills_in_window >= 2 {
                result.push("⚠️ Skirmishes detected - team fight may be developing!".to_string());
            }
        }
        
        result
    }
}

// Debug mode for saving game state data
struct DebugMode {
    enabled: bool,
    last_save_time: std::time::Instant,
    save_interval: std::time::Duration,
    debug_dir: String,
}

impl DebugMode {
    fn new(enabled: bool) -> Self {
        // Create debug directory if it doesn't exist
        if enabled {
            let debug_dir = "dota2_coach_debug";
            let _ = fs::create_dir_all(debug_dir);
            
            Self {
                enabled,
                last_save_time: std::time::Instant::now(),
                save_interval: std::time::Duration::from_secs(30), // Save every 30 seconds
                debug_dir: debug_dir.to_string(),
            }
        } else {
            Self {
                enabled,
                last_save_time: std::time::Instant::now(),
                save_interval: std::time::Duration::from_secs(60),
                debug_dir: String::new(),
            }
        }
    }
    
    fn save_game_state(&mut self, state: &GameState) {
        if !self.enabled {
            return;
        }
        
        let now = std::time::Instant::now();
        if now.duration_since(self.last_save_time) >= self.save_interval {
            // Get game time for filename
            let game_time = state.map.as_ref()
                .and_then(|m| m.game_time)
                .unwrap_or(0);
            
            // Create timestamp for filename
            let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
            let filename = format!("{}/gsi_data_{}_{}.json", self.debug_dir, timestamp, game_time);
            
            // Serialize and save game state
            if let Ok(json) = serde_json::to_string_pretty(state) {
                if let Ok(mut file) = fs::File::create(&filename) {
                    let _ = file.write_all(json.as_bytes());
                }
            }
            
            self.last_save_time = now;
        }
    }
}

// Main function with improved coaching features
#[tokio::main]
async fn main() {
    println!("{}", "Dota 2 Coach - Advanced Game Analysis".green().bold());
    println!("{}", "======================================".green());
    println!("Starting GSI server on port 3000...");
    
    // Enable debug mode (set to true to log game state data)
    let debug_mode = Arc::new(Mutex::new(DebugMode::new(false)));
    
    // Create shared state
    let game_state = Arc::new(Mutex::new(None::<GameState>));
    let last_game_state = Arc::new(Mutex::new(None::<GameState>));
    let enemy_tracker = Arc::new(Mutex::new(EnemyPositionTracker::new()));
    let perf_tracker = Arc::new(Mutex::new(HeroPerformanceTracker::new()));
    let team_fight_analyzer = Arc::new(Mutex::new(TeamFightAnalyzer::new()));
    let _game_events = Arc::new(Mutex::new(Vec::<GameEvent>::new())); // Fixed: Added underscore to silence warning
    
    // Clones for the server endpoint
    let game_state_clone = game_state.clone();
    let last_game_state_clone = last_game_state.clone();
    let enemy_tracker_clone = enemy_tracker.clone();
    let perf_tracker_clone = perf_tracker.clone();
    let team_fight_analyzer_clone = team_fight_analyzer.clone();
    let debug_mode_clone = debug_mode.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::content_length_limit(1024 * 1024 * 10))
        .and(warp::body::json())
        .map(move |data: Value| {
            // Convert the incoming JSON to our GameState struct
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Get the current game state for comparison
                    let mut last_gs = last_game_state_clone.lock().unwrap();
                    
                    // Store the game state
                    let mut gs = game_state_clone.lock().unwrap();
                    
                    // Save current state to debug mode
                    let mut debug = debug_mode_clone.lock().unwrap();
                    debug.save_game_state(&state);
                    
                    // Update trackers
                    {
                        let mut tracker = enemy_tracker_clone.lock().unwrap();
                        tracker.update(&state);
                    }
                    
                    {
                        let mut perf = perf_tracker_clone.lock().unwrap();
                        perf.update(&state, &last_gs);
                    }
                    
                    {
                        let mut team_fight = team_fight_analyzer_clone.lock().unwrap();
                        team_fight.update(&state, &last_gs);
                    }
                    
                    // Update last game state before setting current
                    *last_gs = gs.clone();
                    
                    // Set current game state
                    *gs = Some(state);
                    
                    // Log timestamp of update
                    //let time = Local::now().format("%H:%M:%S").to_string();
                    //println!("[{}] Received game state update", time);
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
    
    // Create a thread for the display/coaching logic
    let display_thread = thread::spawn(move || {
        let mut last_game_time = -1;
        
        loop {
            // Try to get the current game state
            let state_option = {
                let state = game_state.lock().unwrap();
                state.clone()
            };
            
            if let Some(state) = state_option {
                // Only update display if game time has changed
                let game_time = state.map.as_ref().and_then(|m| m.game_time).unwrap_or(-1);
                
                if game_time != last_game_time {
                    // Clear screen and display updated info
                    print!("\x1B[2J\x1B[1;1H");  // Clear screen and move cursor to home
                    
                    // Display game information and coaching insights
                    display_coach_interface(&state, &game_time, &enemy_tracker, &perf_tracker, &team_fight_analyzer);
                    
                    // Update last game time
                    last_game_time = game_time;
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

// Function to display the main coach interface
fn display_coach_interface(
    state: &GameState, 
    game_time: &i32, 
    enemy_tracker: &Arc<Mutex<EnemyPositionTracker>>,
    perf_tracker: &Arc<Mutex<HeroPerformanceTracker>>,
    team_fight_analyzer: &Arc<Mutex<TeamFightAnalyzer>>
) {
    println!("{}", "DOTA 2 COACH - LIVE GAME ANALYSIS".green().bold());
    println!("{}", "====================================".green());
    
    // Display game state section
    display_game_state_section(state);
    
    // Display player section
    display_player_section(state);
    
    // Display hero section
    display_hero_section(state);
    
    // Display coaching insights based on game phase
    display_coaching_insights(state, *game_time, enemy_tracker, perf_tracker, team_fight_analyzer);
    
    println!("\nPress Ctrl+C to exit");
}

// Function to display game state information
fn display_game_state_section(state: &GameState) {
    println!("{}", "GAME STATE".yellow().bold());
    println!("{}", "----------".yellow());
    
    if let Some(map) = &state.map {
        // Game time
        println!("⏱️  Game Time: {}", format_game_time(map.game_time));
        
        // Game state
        println!("🎮 Game State: {}", map.game_state.as_deref().unwrap_or("Unknown"));
        
        // Daytime indicator
        if let Some(is_day) = map.daytime {
            println!("☀️  Time: {}", if is_day { "Day" } else { "Night" });
        }
        
        // Show team scores if available
        println!("📊 Score: {} Radiant - {} Dire", 
            map.radiant_score.unwrap_or(0).to_string().green(), 
            map.dire_score.unwrap_or(0).to_string().red());
    } else {
        println!("No game state information available");
    }
    
    println!();
}

// Function to display player information
fn display_player_section(state: &GameState) {
    println!("{}", "PLAYER INFO".yellow().bold());
    println!("{}", "-----------".yellow());
    
    if let Some(player) = &state.player {
        println!("👤 Player: {}", player.name.as_deref().unwrap_or("Unknown"));
        
        // Team
        println!("🏷️  Team: {}", match player.team_name.as_deref() {
            Some("radiant") => "Radiant".green(),
            Some("dire") => "Dire".red(),
            _ => "Unknown".normal()
        });
        
        // KDA
        println!("⚔️  KDA: {}/{}/{}", 
            player.kills.unwrap_or(0),
            player.deaths.unwrap_or(0),
            player.assists.unwrap_or(0));
        
        // Gold and net worth
        if let Some(gold) = player.gold {
            println!("💰 Gold: {}", gold);
        }
        
        if let Some(net_worth) = player.net_worth {
            println!("💎 Net Worth: {}", net_worth);
        }
        
        // GPM/XPM
        if let (Some(gpm), Some(xpm)) = (player.gpm, player.xpm) {
            println!("📈 GPM/XPM: {}/{}", gpm, xpm);
        }
        
        // Last hits/denies
        if let (Some(last_hits), Some(denies)) = (player.last_hits, player.denies) {
            println!("🌾 CS: {} last hits, {} denies", last_hits, denies);
        }
    } else {
        println!("No player information available");
    }
    
    println!();
}

// Function to display hero information
fn display_hero_section(state: &GameState) {
    println!("{}", "HERO INFO".yellow().bold());
    println!("{}", "---------".yellow());
    
    if let Some(hero) = &state.hero {
        println!("👑 Hero: {}, Level {}", 
            format_hero_name(&hero.name.as_deref().unwrap_or("Unknown").to_string()),
            hero.level.unwrap_or(0));
        
        // Health and mana bars
        if let (Some(health_pct), Some(mana_pct)) = (hero.health_percent, hero.mana_percent) {
            println!("❤️  Health: {}% {}", health_pct, create_bar(health_pct, 20));
            println!("🔷 Mana: {}% {}", mana_pct, create_bar(mana_pct, 20));
        }
        
        // Status effects
        let mut status_effects = Vec::new();
        if hero.silenced.unwrap_or(false) { status_effects.push("Silenced"); }
        if hero.stunned.unwrap_or(false) { status_effects.push("Stunned"); }
        if hero.hexed.unwrap_or(false) { status_effects.push("Hexed"); }
        if hero.disarmed.unwrap_or(false) { status_effects.push("Disarmed"); }
        if hero.magicimmune.unwrap_or(false) { status_effects.push("Magic Immune"); }
        if hero.muted.unwrap_or(false) { status_effects.push("Muted"); }
        if hero.r#break.unwrap_or(false) { status_effects.push("Break"); } // Fixed: Use r#break
        if hero.smoked.unwrap_or(false) { status_effects.push("Smoked"); }
        
        if !status_effects.is_empty() {
            println!("⚡ Status: {}", status_effects.join(", "));
        }
        
        // Buyback status
        if let (Some(buyback_cost), Some(buyback_cooldown)) = (hero.buyback_cost, hero.buyback_cooldown) {
            println!("💸 Buyback: Cost {}, {} cooldown", 
                buyback_cost,
                if buyback_cooldown > 0 { 
                    format!("{} seconds", buyback_cooldown) 
                } else { 
                    "No".to_string() 
                });
        }
        
        // Abilities
        if let Some(abilities) = &state.abilities {
            println!("\n🧙 Abilities:");
            for (id, ability) in abilities {
                if let Some(name) = &ability.name {
                    // Skip passive abilities for brevity
                    if name.starts_with("plus_") || ability.passive.unwrap_or(false) {
                        continue;
                    }
                    
                    let ability_name = name.replace("item_", "")
                        .replace("_", " ")
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            chars.next().map_or(String::new(), |c| 
                                c.to_uppercase().collect::<String>() + chars.as_str())
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    
                    let level_info = ability.level.map_or(String::new(), |lvl| format!("(Lvl {})", lvl));
                    let status = if ability.can_cast.unwrap_or(false) {
                        "✅ Ready".green()
                    } else if let Some(cd) = ability.cooldown {
                        if cd > 0 {
                            format!("⏳ CD: {}s", cd).red()
                        } else {
                            "✅ Ready".green()
                        }
                    } else {
                        "".normal()
                    };
                    
                    println!("  {} {} {}", ability_name, level_info, status);
                }
            }
        }
        
        // Items
        if let Some(items) = &state.items {
            println!("\n🎒 Items:");
            let mut item_count = 0;
            
            for (slot, item) in items {
                if let Some(name) = &item.name {
                    if name == "empty" {
                        continue;
                    }
                    
                    let item_name = name.replace("item_", "")
                        .replace("_", " ")
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            chars.next().map_or(String::new(), |c| 
                                c.to_uppercase().collect::<String>() + chars.as_str())
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    
                    let charges_info = item.charges.map_or(String::new(), |charges| 
                        if charges > 1 { format!("({})", charges) } else { String::new() }
                    );
                    
                    let status = if item.can_cast.unwrap_or(false) {
                        "✅".green()
                    } else if let Some(cd) = item.cooldown {
                        if cd > 0 {
                            format!("⏳ {}s", cd).red()
                        } else {
                            "✅".green()
                        }
                    } else {
                        "".normal()
                    };
                    
                    println!("  {} {} {}", item_name, charges_info, status);
                    item_count += 1;
                }
            }
            
            if item_count == 0 {
                println!("  No items");
            }
        }
    } else {
        println!("No hero information available");
    }
    
    println!();
}

// Function to display coaching insights
fn display_coaching_insights(
    state: &GameState, 
    game_time: i32, 
    enemy_tracker: &Arc<Mutex<EnemyPositionTracker>>,
    perf_tracker: &Arc<Mutex<HeroPerformanceTracker>>,
    team_fight_analyzer: &Arc<Mutex<TeamFightAnalyzer>>
) {
    println!("{}", "COACH INSIGHTS".cyan().bold());
    println!("{}", "--------------".cyan());
    
    let minutes = game_time / 60;
    let seconds = game_time % 60;
    
    // Team fight status
    let team_fight_status = {
        let analyzer = team_fight_analyzer.lock().unwrap();
        analyzer.get_team_fight_status(game_time)
    };
    
    if !team_fight_status.is_empty() {
        println!("{}", "⚔️  TEAM FIGHT STATUS".red().bold());
        for line in team_fight_status {
            println!("  {}", line);
        }
        println!();
    }
    
    // Enemy position tracking
    let enemy_movements = {
        let tracker = enemy_tracker.lock().unwrap();
        tracker.get_movement_descriptions(game_time)
    };
    
    if !enemy_movements.is_empty() {
        println!("{}", "👁️  ENEMY TRACKING".red().bold());
        for line in enemy_movements {
            println!("{}", line);
        }
        
        // Show predictions
        let predictions = {
            let tracker = enemy_tracker.lock().unwrap();
            tracker.predict_movements(game_time)
        };
        
        if !predictions.is_empty() {
            println!("\n  Predicted movements:");
            for (hero_name, (x, y)) in predictions {
                println!("    {} likely at ({}, {})", hero_name, x, y);
                
                // Check if the predicted position is near player
                if let (Some(hero), Some(player_x), Some(player_y)) = 
                    (state.hero.as_ref(), state.hero.as_ref().and_then(|h| h.xpos), state.hero.as_ref().and_then(|h| h.ypos)) {
                    
                    let dx = x - player_x;
                    let dy = y - player_y;
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    if distance < 2000.0 {
                        println!("      ⚠️ WARNING: This enemy may be very close to you!");
                    }
                }
            }
        }
        println!();
    }
    
    // Performance metrics
    let performance_metrics = {
        let tracker = perf_tracker.lock().unwrap();
        tracker.display_metrics(game_time)
    };
    
    if !performance_metrics.is_empty() {
        println!("{}", "📊 PERFORMANCE METRICS".yellow().bold());
        for line in performance_metrics {
            println!("{}", line);
        }
        println!();
    }
    
    // Phase-specific insights
    if minutes < 10 {
        // Early game insights
        println!("{}", "🌱 EARLY GAME COACHING".green().bold());
        
        // Stack timing reminder
        if seconds >= 45 && seconds <= 48 {
            println!("  ⏰ Stack camps now! Pull at X:53.");
        }
        
        // Rune spawning reminder
        if minutes > 0 && minutes % 2 == 0 && seconds >= 55 {
            println!("  ⏰ Water runes spawning in a few seconds!");
        } else if minutes >= 4 && minutes % 5 == 4 && seconds >= 55 {
            println!("  ⏰ Power runes spawning in a few seconds!");
        }
        
        // Check last hit efficiency
        if let Some(player) = &state.player {
            if let Some(last_hits) = player.last_hits {
                let expected_cs = minutes * 10; // Rough benchmark: 10 CS per minute
                if last_hits < expected_cs / 2 {
                    println!("  ⚠️ Your last hits are low ({}). Focus more on last hitting.", last_hits);
                } else if last_hits >= expected_cs {
                    println!("  ✅ Good job on last hitting! You have {} CS.", last_hits);
                }
            }
        }
    } else if minutes < 25 {
        // Mid game insights
        println!("{}", "🌳 MID GAME COACHING".yellow().bold());
        
        // Item suggestions based on gold
        if let Some(player) = &state.player {
            if let Some(gold) = player.gold {
                if gold >= 4000 {
                    println!("  💰 You have sufficient gold for major items (BKB, Blink, etc.)");
                } else if gold >= 2000 {
                    println!("  💰 You have gold for mid-tier items (Force Staff, Eul's, etc.)");
                } else if gold >= 1000 {
                    println!("  💰 Consider purchasing support/utility items");
                }
            }
        }
        
        // Ward advice
        if minutes % 7 == 0 && seconds < 10 {
            println!("  🔍 New observer wards are available in shop");
        }
        
        // Roshan timing
        if minutes >= 10 {
            println!("  🔶 Roshan is available. Consider checking with team coordination.");
        }
    } else {
        // Late game insights
        println!("{}", "🌲 LATE GAME COACHING".red().bold());
        
        // Buyback reminder
        if let (Some(hero), Some(player)) = (state.hero.as_ref(), state.player.as_ref()) {
            if let (Some(buyback_cost), Some(gold)) = (hero.buyback_cost, player.gold) {
                if gold < buyback_cost {
                    println!("  ⚠️ You don't have buyback gold! Need {} more gold.", buyback_cost - gold);
                } else {
                    println!("  ✅ You have buyback available ({} gold).", buyback_cost);
                }
            }
        }
        
        // Team fight readiness
        let readiness_score = assess_team_fight_readiness(state);
        match readiness_score {
            score if score >= 4 => println!("  ✅ Team fight readiness: Excellent! All systems ready."),
            score if score >= 2 => println!("  ⚖️ Team fight readiness: Good. Most resources available."),
            score if score >= 0 => println!("  ⚠️ Team fight readiness: Caution advised. Limited resources."),
            _ => println!("  ❌ Team fight readiness: Not ready. Consider retreating."),
        }
    }
    
    println!();
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

// Helper function to create a visual bar
fn create_bar(percent: i32, length: usize) -> String {
    let filled_length = (percent as f32 / 100.0 * length as f32).round() as usize;
    let empty_length = length - filled_length;
    
    let filled = "█".repeat(filled_length);
    let empty = "░".repeat(empty_length);
    
    format!("{}{}", filled, empty)
}

// Helper function to assess team fight readiness
fn assess_team_fight_readiness(state: &GameState) -> i32 {
    let mut score = 0;
    
    // Check hero health and mana
    if let Some(hero) = &state.hero {
        if let Some(health_pct) = hero.health_percent {
            if health_pct > 80 {
                score += 2;
            } else if health_pct > 50 {
                score += 1;
            } else {
                score -= 1;
            }
        }
        
        if let Some(mana_pct) = hero.mana_percent {
            if mana_pct > 70 {
                score += 2;
            } else if mana_pct > 40 {
                score += 1;
            } else {
                score -= 1;
            }
        }
    }
    
    // Check ability cooldowns
    if let Some(abilities) = &state.abilities {
        let has_ultimate_ready = abilities.values().any(|ability| {
            ability.ultimate.unwrap_or(false) && 
            ability.can_cast.unwrap_or(false)
        });
        
        if has_ultimate_ready {
            score += 2;
        }
        
        // Check if key abilities are ready
        let all_key_abilities_ready = abilities.values()
            .filter(|ability| !ability.passive.unwrap_or(true))
            .all(|ability| ability.can_cast.unwrap_or(false));
        
        if all_key_abilities_ready {
            score += 1;
        }
    }
    
    score
}

// Helper function to estimate hero level based on game time
fn estimate_hero_level(game_time: i32) -> i32 {
    let minutes = game_time / 60;
    
    // Very simple approximation
    if minutes < 10 {
        (minutes / 2) + 1
    } else {
        (minutes / 3) + 5
    }
}