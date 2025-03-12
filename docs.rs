/**
 * ==========================================================================
 * DOTA 2 GAME STATE INTEGRATION (GSI) COMPREHENSIVE REFERENCE & IMPLEMENTATION GUIDE
 * ==========================================================================
 * 
 * This document serves both as a reference for all data available via Dota 2's GSI 
 * and as a practical guide for implementing a real-time coach using this data.
 */

/**
 * TABLE OF CONTENTS
 * 
 * 1. SETUP REQUIREMENTS
 * 2. AVAILABLE DATA STRUCTURES
 * 3. IMPLEMENTATION PATTERNS
 * 4. COACH IMPLEMENTATION EXAMPLES
 * 5. ADVANCED FEATURES
 * 6. TROUBLESHOOTING
 */

/**
 * 1. SETUP REQUIREMENTS
 * ====================
 * 
 * To use Dota 2 GSI, you need to complete these steps:
 */

/**
 * 1.1 GSI Configuration File
 * 
 * Create a file named "gamestate_integration_coach.cfg" (or any name prefixed with 
 * "gamestate_integration_") in your Dota 2 configuration directory:
 * 
 * Windows: C:\Program Files (x86)\Steam\steamapps\common\dota 2 beta\game\dota\cfg\
 * Linux: ~/.steam/steam/steamapps/common/dota 2 beta/game/dota/cfg/
 * 
 * File contents:
 * 
 * "Dota 2 Coach Configuration"
 * {
 *     "uri"               "http://127.0.0.1:3000/"
 *     "timeout"           "5.0"
 *     "buffer"            "0.1"
 *     "throttle"          "0.1"
 *     "heartbeat"         "30.0"
 *     "data"
 *     {
 *         "buildings"     "1"
 *         "provider"      "1"
 *         "map"           "1"
 *         "player"        "1"
 *         "hero"          "1"
 *         "abilities"     "1"
 *         "items"         "1"
 *         "draft"         "1"
 *         "wearables"     "1" 
 *         "minimap"       "1"
 *     }
 *     "auth"
 *     {
 *         "token"         "mytoken123"
 *     }
 * }
 */

/**
 * 1.2 Launch Options
 * 
 * Add the launch option "-gamestateintegration" to Dota 2:
 * 
 * 1. Open Steam
 * 2. Right-click on Dota 2 and select "Properties"
 * 3. In the General tab, click "Set Launch Options"
 * 4. Add "-gamestateintegration" (without quotes)
 * 5. Click "OK" and close the properties window
 */

/**
 * 1.3 Basic Application Structure
 * 
 * Your Dota 2 coach application needs these essential components:
 * 
 * 1. HTTP server to receive GSI data
 * 2. Data structures to parse GSI JSON
 * 3. Business logic to process the data
 * 4. User interface to display insights
 * 
 * The following Cargo.toml dependencies are recommended:
 * 
 * [dependencies]
 * tokio = { version = "1", features = ["full"] }
 * warp = "0.3"
 * serde = { version = "1.0", features = ["derive"] }
 * serde_json = "1.0"
 * colored = "2.0"
 * chrono = "0.4"
 */

/**
 * 2. AVAILABLE DATA STRUCTURES
 * ===========================
 * 
 * These are the primary data structures you'll need to parse GSI data.
 * Each section includes the full Rust struct definition for direct use in your code.
 */

/**
 * 2.1 Root GameState Structure
 * 
 * This is the main structure that encompasses all GSI data.
 * Use this as the top-level structure for deserializing the JSON payload.
 */

 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct GameState {
     provider: Option<Provider>,
     map: Option<Map>,
     player: Option<Player>,
     hero: Option<Hero>,
     abilities: Option<HashMap<String, Ability>>,
     items: Option<HashMap<String, Item>>,
     buildings: Option<HashMap<String, HashMap<String, Building>>>,
     draft: Option<Value>,
     minimap: Option<HashMap<String, MinimapObject>>,
     wearables: Option<Value>,
     
     // Fallback for any other fields
     #[serde(flatten)]
     other: HashMap<String, Value>,
 }
 
 /**
  * 2.2 Provider Information
  * 
  * Basic information about the data provider (Dota 2).
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Provider {
     name: Option<String>,         // Always "Dota 2"
     appid: Option<i32>,           // Always 570 (Dota 2's Steam App ID)
     version: Option<i32>,         // Version of the GSI system
     timestamp: Option<i64>,       // Current timestamp when the data was sent
 }
 
 /**
  * 2.3 Map Information
  * 
  * Details about the current game map and state.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Map {
     name: Option<String>,              // Map name (e.g., "dota")
     matchid: Option<String>,           // ID of the current match
     game_time: Option<i32>,            // Current game time in seconds
     clock_time: Option<i32>,           // Time shown on the in-game clock
     daytime: Option<bool>,             // Whether it's currently daytime in-game
     nightstalker_night: Option<bool>,  // Whether it's Night Stalker's night
     game_state: Option<String>,        // Current game state 
     paused: Option<bool>,              // Whether the game is paused
     win_team: Option<String>,          // Which team won (or "none" if game in progress)
     customgamename: Option<String>,    // Name of the custom game if applicable
     ward_purchase_cooldown: Option<i32>, // Cooldown on ward purchases
     radiant_score: Option<i32>,        // Radiant team score (kills)
     dire_score: Option<i32>,           // Dire team score (kills)
 }
 
 /**
  * Game state values for the game_state field:
  * 
  * - "DOTA_GAMERULES_STATE_DISCONNECT" - Disconnected
  * - "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS" - Normal gameplay
  * - "DOTA_GAMERULES_STATE_HERO_SELECTION" - Hero selection phase
  * - "DOTA_GAMERULES_STATE_INIT" - Initializing
  * - "DOTA_GAMERULES_STATE_LAST" - Game ending
  * - "DOTA_GAMERULES_STATE_POST_GAME" - Post-game
  * - "DOTA_GAMERULES_STATE_PRE_GAME" - Pre-game
  * - "DOTA_GAMERULES_STATE_STRATEGY_TIME" - Strategy time
  * - "DOTA_GAMERULES_STATE_WAIT_FOR_MAP_TO_LOAD" - Waiting for map
  * - "DOTA_GAMERULES_STATE_WAIT_FOR_PLAYERS_TO_LOAD" - Waiting for players
  * - "DOTA_GAMERULES_STATE_CUSTOM_GAME_SETUP" - Custom game setup
  */
 
 /**
  * 2.4 Player Information
  * 
  * Information about the current player.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Player {
     steamid: Option<String>,           // Steam ID of the player
     name: Option<String>,              // Player name
     activity: Option<String>,          // Current activity ("playing", "menu")
     kills: Option<i32>,                // Kill count
     deaths: Option<i32>,               // Death count
     assists: Option<i32>,              // Assist count
     last_hits: Option<i32>,            // Last hit count
     denies: Option<i32>,               // Deny count
     kill_streak: Option<i32>,          // Current kill streak
     commands_issued: Option<i32>,      // Number of commands issued
     team_name: Option<String>,         // Team name ("radiant" or "dire")
     gold: Option<i32>,                 // Current gold
     gold_reliable: Option<i32>,        // Current reliable gold
     gold_unreliable: Option<i32>,      // Current unreliable gold
     gold_from_hero_kills: Option<i32>, // Gold from hero kills
     gold_from_creep_kills: Option<i32>, // Gold from creep kills
     gold_from_income: Option<i32>,     // Gold from passive income
     gold_from_shared: Option<i32>,     // Gold from shared sources
     net_worth: Option<i32>,            // Total net worth
     gpm: Option<i32>,                  // Gold per minute
     xpm: Option<i32>,                  // Experience per minute
     
     // Kill list data as a map of victim IDs to kill counts
     kill_list: Option<HashMap<String, i32>>,
 }
 
 /**
  * 2.5 Hero Information
  * 
  * Details about the player's hero.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Hero {
     id: Option<i32>,                   // Hero ID
     name: Option<String>,              // Hero name (e.g., "npc_dota_hero_ursa")
     level: Option<i32>,                // Current hero level
     xp: Option<i32>,                   // Current experience points
     alive: Option<bool>,               // Whether the hero is alive
     respawn_seconds: Option<i32>,      // Seconds until respawn if dead
     buyback_cost: Option<i32>,         // Cost of buyback
     buyback_cooldown: Option<i32>,     // Cooldown on buyback
     health: Option<i32>,               // Current health
     max_health: Option<i32>,           // Maximum health
     health_percent: Option<i32>,       // Health percentage (0-100)
     mana: Option<i32>,                 // Current mana
     max_mana: Option<i32>,             // Maximum mana
     mana_percent: Option<i32>,         // Mana percentage (0-100)
     silenced: Option<bool>,            // Whether the hero is silenced
     stunned: Option<bool>,             // Whether the hero is stunned
     disarmed: Option<bool>,            // Whether the hero is disarmed
     magicimmune: Option<bool>,         // Whether the hero is magic immune
     hexed: Option<bool>,               // Whether the hero is hexed
     muted: Option<bool>,               // Whether the hero is muted
     break: Option<bool>,               // Whether the hero has break status
     aghanims_scepter: Option<bool>,    // Whether the hero has Aghanim's Scepter
     aghanims_shard: Option<bool>,      // Whether the hero has Aghanim's Shard
     smoked: Option<bool>,              // Whether the hero is under Smoke of Deceit effect
     has_debuff: Option<bool>,          // Whether the hero has any debuff
     
     // Talent selections
     talent_1: Option<bool>,            // Whether talent 1 is selected
     talent_2: Option<bool>,            // Whether talent 2 is selected
     talent_3: Option<bool>,            // Whether talent 3 is selected
     talent_4: Option<bool>,            // Whether talent 4 is selected
     talent_5: Option<bool>,            // Whether talent 5 is selected
     talent_6: Option<bool>,            // Whether talent 6 is selected
     talent_7: Option<bool>,            // Whether talent 7 is selected
     talent_8: Option<bool>,            // Whether talent 8 is selected
     
     // Position on map
     xpos: Option<i32>,                 // X position on map
     ypos: Option<i32>,                 // Y position on map
 }
 
 /**
  * 2.6 Ability Information
  * 
  * Details about hero abilities.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Ability {
     name: Option<String>,              // Ability name
     level: Option<i32>,                // Current ability level
     can_cast: Option<bool>,            // Whether the ability can be cast
     passive: Option<bool>,             // Whether the ability is passive
     ability_active: Option<bool>,      // Whether the ability is active/available
     cooldown: Option<i32>,             // Current cooldown in seconds
     ultimate: Option<bool>,            // Whether the ability is an ultimate
 }
 
 /**
  * 2.7 Item Information
  * 
  * Details about items in inventory.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Item {
     name: Option<String>,              // Item name (or "empty" if no item)
     purchaser: Option<i32>,            // ID of the purchaser
     item_level: Option<i32>,           // Item level if applicable
     contains_rune: Option<String>,     // Rune contained if applicable
     can_cast: Option<bool>,            // Whether the item can be cast
     cooldown: Option<i32>,             // Current cooldown in seconds
     passive: Option<bool>,             // Whether the item is passive
     charges: Option<i32>,              // Number of charges
     item_charges: Option<i32>,         // Alternative field for charges
 }
 
 /**
  * 2.8 Building Information
  * 
  * Details about buildings on the map.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct Building {
     health: i32,                       // Current health
     max_health: i32,                   // Maximum health
 }
 
 /**
  * 2.9 Minimap Object Information
  * 
  * Details about entities on the minimap - CRITICAL for enemy tracking.
  */
 
 #[derive(Clone, Debug, Deserialize, Serialize)]
 struct MinimapObject {
     image: String,                     // Image used on minimap (e.g., "minimap_enemyicon")
     name: Option<String>,              // Name of the entity if applicable (e.g., "npc_dota_hero_ursa")
     team: i32,                         // Team ID (2 for Radiant, 3 for Dire, 4 for Neutral, 5 for Other)
     unitname: String,                  // Unit's internal name
     visionrange: i32,                  // Vision range of the entity
     xpos: i32,                         // X position on map
     ypos: i32,                         // Y position on map
     yaw: Option<i32>,                  // Facing direction in degrees
 }
 
 /**
  * 2.10 Coach-Specific Data Structures
  * 
  * These are not part of the GSI data, but useful for your coach application.
  */
 
 // Structure to track enemy hero information
 #[derive(Clone, Debug)]
 struct EnemyHero {
     name: String,                      // Hero name
     position: (i32, i32),              // Position (x, y)
     last_seen: i32,                    // Game time when last seen
     estimated_level: i32,              // Estimated hero level
     items: Vec<String>,                // Known items
 }
 
 // Structure to track important game events
 #[derive(Clone, Debug)]
 struct GameEvent {
     time: i32,                         // Game time when event occurred
     event_type: String,                // Type of event
     description: String,               // Description of event
 }
 
 /**
  * 3. IMPLEMENTATION PATTERNS
  * =========================
  * 
  * Here are the key implementation patterns for building your coach application.
  */
 
 /**
  * 3.1 Basic HTTP Server Setup
  * 
  * This is how to set up a server to receive GSI data:
  */
 
 #[tokio::main]
 async fn main() {
     println!("Starting Dota 2 Coach...");
     
     // Create shared state
     let game_state = Arc::new(Mutex::new(None::<GameState>));
     let game_state_clone = game_state.clone();
     
     // Set up an endpoint to receive GSI data
     let gsi_endpoint = warp::post()
         .and(warp::body::content_length_limit(1024 * 1024 * 10))
         .and(warp::body::json())
         .map(move |data: Value| {
             // Convert the incoming JSON to GameState struct
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
     
     println!("Server running on http://127.0.0.1:3000");
     println!("Waiting for Dota 2 game data...");
     
     // Now create your display/coach logic in a separate thread
     let display_thread = create_coach_thread(game_state);
     
     // Wait for the threads to complete (they won't in normal operation)
     display_thread.join().unwrap();
     server_thread.abort();
 }
 
 /**
  * 3.2 Extracting Enemy Hero Information
  * 
  * This is the key pattern for tracking enemy heroes:
  */
 
 fn extract_enemy_heroes(state: &GameState) -> HashMap<String, EnemyHero> {
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
                     
                     // Update or add hero information
                     enemy_heroes.insert(hero_name.clone(), EnemyHero {
                         name: hero_name,
                         position: (obj.xpos, obj.ypos),
                         last_seen: current_game_time,
                         estimated_level: estimate_hero_level(current_game_time),
                         items: Vec::new(), // We won't have direct access to enemy items
                     });
                 }
             }
         }
     }
     
     enemy_heroes
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
 
 // Estimate hero level based on game time (very rough estimate)
 fn estimate_hero_level(game_time: i32) -> i32 {
     let minutes = game_time / 60;
     
     // Very simple approximation
     if minutes < 10 {
         return (minutes / 2) + 1;
     } else {
         return (minutes / 3) + 5;
     }
 }
 
 /**
  * 3.3 Creating the Coach Thread
  * 
  * This is how to set up a thread for analyzing and displaying GSI data:
  */
 
 fn create_coach_thread(game_state: Arc<Mutex<Option<GameState>>>) -> thread::JoinHandle<()> {
     thread::spawn(move || {
         let mut last_game_time = -1;
         let mut enemy_heroes_history = HashMap::new();
         
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
                     // Process and update game state
                     let enemy_heroes = extract_enemy_heroes(&state);
                     
                     // Update enemy hero history
                     for (name, hero) in &enemy_heroes {
                         enemy_heroes_history.insert(name.clone(), hero.clone());
                     }
                     
                     // Display game information
                     display_game_info(&state, &enemy_heroes_history);
                     
                     // Generate coaching insights
                     generate_coaching_insights(&state, &enemy_heroes_history);
                     
                     // Update last game time
                     last_game_time = game_time;
                 }
             }
             
             // Sleep to avoid excessive CPU usage
             thread::sleep(Duration::from_millis(100));
         }
     })
 }
 
 /**
  * 3.4 Displaying Game Information
  * 
  * How to display the game state to the user:
  */
 
 fn display_game_info(state: &GameState, enemy_heroes: &HashMap<String, EnemyHero>) {
     // Clear screen
     print!("\x1B[2J\x1B[1;1H");
     
     println!("{}", "Dota 2 Coach - Live Game Assistant".green().bold());
     println!("{}", "-------------------------------------".green());
     
     // Display game state
     if let Some(map) = &state.map {
         println!("Game Time: {}", format_game_time(map.game_time));
         println!("Game State: {}", map.game_state.as_deref().unwrap_or("Unknown"));
         
         // Show team scores if available
         println!("Score: {} Radiant - {} Dire", 
             map.radiant_score.unwrap_or(0).to_string().green(), 
             map.dire_score.unwrap_or(0).to_string().red());
         
         println!("-------------------------------------");
     }
     
     // Display player info
     if let Some(player) = &state.player {
         println!("{}", "Player Information:".yellow().bold());
         println!("Name: {}", player.name.as_deref().unwrap_or("Unknown"));
         println!("Team: {}", match player.team_name.as_deref() {
             Some("radiant") => "Radiant".green(),
             Some("dire") => "Dire".red(),
             _ => "Unknown".normal()
         });
         println!("KDA: {}/{}/{}", 
             player.kills.unwrap_or(0),
             player.deaths.unwrap_or(0),
             player.assists.unwrap_or(0));
         println!("Net Worth: {}", player.net_worth.unwrap_or(0));
         println!("-------------------------------------");
     }
     
     // Display hero info
     if let Some(hero) = &state.hero {
         println!("{}", "Hero Information:".yellow().bold());
         println!("Hero: {}, Level: {}", 
             format_hero_name(&hero.name.as_deref().unwrap_or("Unknown").to_string()),
             hero.level.unwrap_or(0));
         
         // Show health and mana
         if let Some(health_pct) = hero.health_percent {
             println!("Health: {}%", health_pct);
         }
         if let Some(mana_pct) = hero.mana_percent {
             println!("Mana: {}%", mana_pct);
         }
         
         println!("-------------------------------------");
     }
     
     // Display enemy heroes
     println!("{}", "Enemy Heroes:".red().bold());
     
     if enemy_heroes.is_empty() {
         println!("No enemy heroes detected yet");
     } else {
         for (_, hero) in enemy_heroes {
             println!("Hero: {} (Est. Level: {})", hero.name, hero.estimated_level);
             println!("  Last seen: {}", format_game_time(Some(hero.last_seen)));
             println!("  Position: ({}, {})", hero.position.0, hero.position.1);
             
             if !hero.items.is_empty() {
                 println!("  Estimated items:");
                 for item in &hero.items {
                     println!("    - {}", item);
                 }
             }
             
             println!();
         }
     }
     
     println!("-------------------------------------");
 }
 
 /**
  * 3.5 Generating Coaching Insights
  * 
  * How to generate and display coaching insights based on game state:
  */
 
 fn generate_coaching_insights(state: &GameState, enemy_heroes: &HashMap<String, EnemyHero>) {
     println!("{}", "Coach Insights:".cyan().bold());
     
     // Get game time
     let game_time = state.map.as_ref().and_then(|m| m.game_time).unwrap_or(0);
     let minutes = game_time / 60;
     
     // Early game insights
     if minutes < 10 {
         println!("Early Game Phase:");
         
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
         
         // Stack timing reminder
         let seconds = game_time % 60;
         if seconds >= 45 && seconds <= 48 {
             println!("  ⏰ Stack camps now! Pull at X:53.");
         }
         
         // Rune spawning reminder
         if minutes > 0 && minutes % 2 == 0 && seconds >= 55 {
             println!("  ⏰ Water runes spawning in a few seconds!");
         }
     }
     // Mid game insights
     else if minutes < 25 {
         println!("Mid Game Phase:");
         
         // Item suggestions based on gold
         if let Some(player) = &state.player {
             if let Some(gold) = player.gold {
                 suggest_items_based_on_gold(gold);
             }
         }
         
         // Enemy position warnings
         for (name, hero) in enemy_heroes {
             // If enemy was seen in the last 30 seconds and is near
             if game_time - hero.last_seen < 30 && is_position_dangerous(&hero.position) {
                 println!("  ⚠️ {} was recently spotted nearby - be careful!", name);
             }
         }
         
         // Roshan timing
         if minutes >= 10 {
             println!("  🔶 Roshan is available. Consider checking/taking with team coordination.");
         }
     }
     // Late game insights
     else {
         println!("Late Game Phase:");
         
         // Buyback reminder
         if let Some(hero) = &state.hero {
             if let Some(buyback_cost) = hero.buyback_cost {
                 if let Some(player) = &state.player {
                     if let Some(gold) = player.gold {
                         if gold < buyback_cost {
                             println!("  ⚠️ You don't have buyback gold! Need {} more gold.", buyback_cost - gold);
                         } else {
                             println!("  ✅ You have buyback available ({} gold).", buyback_cost);
                         }
                     }
                 }
             }
         }
         
         // Team fight readiness
         let team_fight_readiness = assess_team_fight_readiness(state);
         println!("  Team fight readiness: {}", team_fight_readiness);
     }
     
     println!("-------------------------------------");
 }
 
 // Helper function to suggest items based on gold
 fn suggest_items_based_on_gold(gold: i32) {
     if gold >= 4000 {
         println!("  💰 You have sufficient gold for major items (BKB, Blink, etc.)");
     } else if gold >= 2000 {
         println!("  💰 You have gold for mid-tier items (Force Staff, Eul's, etc.)");
     } else if gold >= 1000 {
         println!("  💰 Consider purchasing support/utility items");
     }
 }
 
 // Check if a position is dangerous to the player
 fn is_position_dangerous(position: &(i32, i32)) -> bool {
     // This is a simplistic check - in a real implementation,
     // you would compare with the player's position
     let (x, y) = *position;
     
     // Check if position is near common "dangerous" areas
     // This is just an example with arbitrary values
     (x > -1000 && x < 1000 && y > -1000 && y < 1000)
 }
 
 // Assess team fight readiness based on hero state
 fn assess_team_fight_readiness(state: &GameState) -> String {
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

// Translate score to readiness assessment
match score {
    n if n >= 4 => "Excellent! All systems ready for team fight.".green().to_string(),
    n if n >= 2 => "Good. Most resources available.".yellow().to_string(),
    n if n >= 0 => "Caution advised. Limited resources.".red().to_string(),
    _ => "Not ready for team fight. Consider retreating.".red().bold().to_string(),
}
}

/**
 * 3.6 Format Game Time Helper Function
 * 
 * Utility function to format game time from seconds to MM:SS format:
 */

fn format_game_time(seconds: Option<i32>) -> String {
    if let Some(secs) = seconds {
        let minutes = secs / 60;
        let remaining_seconds = secs % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    } else {
        "Unknown".to_string()
    }
}

/**
 * 4. COACH IMPLEMENTATION EXAMPLES
 * ===============================
 * 
 * Complete examples for different coaching features.
 */

/**
 * 4.1 Item Timing Coach
 * 
 * This feature tracks and suggests optimal item timings:
 */

fn analyze_item_timings(state: &GameState) {
    // Get game time and player's net worth
    let game_time = state.map.as_ref().and_then(|m| m.game_time).unwrap_or(0);
    let minutes = game_time / 60;
    
    let net_worth = state.player.as_ref()
        .and_then(|p| p.net_worth)
        .unwrap_or(0);
    
    // Item benchmark timings based on role and minutes
    println!("{}", "Item Timing Analysis:".yellow().bold());
    
    // Example item benchmarks for a core position
    let item_benchmarks = [
        (10, 4000, "Power Treads + Wraith Bands"),
        (15, 7000, "Core farming item (Battlefury/Maelstrom)"),
        (20, 11000, "Second major item (BKB/Desolator)"),
        (30, 18000, "Third major item (Satanic/Butterfly)"),
    ];
    
    // Find the closest benchmark for current time
    let current_benchmark = item_benchmarks
        .iter()
        .filter(|(time, _, _)| *time <= minutes)
        .last();
    
    if let Some((benchmark_time, benchmark_networth, items)) = current_benchmark {
        let next_benchmark = item_benchmarks
            .iter()
            .find(|(time, _, _)| *time > minutes);
        
        // Calculate if player is ahead or behind the curve
        let expected_net_worth = benchmark_networth;
        let net_worth_diff = net_worth - expected_net_worth;
        
        if net_worth_diff >= 1000 {
            println!("  ✅ You're ahead of item timings! +{} gold", net_worth_diff);
        } else if net_worth_diff >= -1000 {
            println!("  ⚖️ You're on track with item timings");
        } else {
            println!("  ⚠️ You're behind on item timings: {} gold", net_worth_diff);
        }
        
        println!("  Current benchmark ({} min): {}", benchmark_time, items);
        
        // Show next item timing goal
        if let Some((next_time, next_networth, next_items)) = next_benchmark {
            let time_left = next_time - minutes;
            let gold_needed = next_networth - net_worth;
            
            println!("  Next goal ({} min): {}", next_time, next_items);
            println!("  Need {} gold in {} minutes ({} GPM)", 
                gold_needed, time_left, gold_needed / time_left);
        }
    } else {
        println!("  No item benchmarks available for current game time");
    }
    
    println!("-------------------------------------");
}

/**
 * 4.2 Map Control Coach
 * 
 * This feature analyzes building status to assess map control:
 */

fn analyze_map_control(state: &GameState) {
    println!("{}", "Map Control Analysis:".magenta().bold());
    
    if let Some(buildings) = &state.buildings {
        // Count standing towers for each team
        let mut radiant_towers = 0;
        let mut dire_towers = 0;
        
        // Player's team
        let player_team = state.player.as_ref()
            .and_then(|p| p.team_name.as_ref())
            .map(|t| t.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        
        // Count towers for each team
        if let Some(radiant_buildings) = buildings.get("radiant") {
            radiant_towers = radiant_buildings.iter()
                .filter(|(name, _)| name.contains("tower"))
                .count();
        }
        
        if let Some(dire_buildings) = buildings.get("dire") {
            dire_towers = dire_buildings.iter()
                .filter(|(name, _)| name.contains("tower"))
                .count();
        }
        
        // Analyze map control based on tower advantage
        let tower_diff = if player_team == "radiant" {
            radiant_towers as i32 - dire_towers as i32
        } else {
            dire_towers as i32 - radiant_towers as i32
        };
        
        println!("  Your team has {} towers, enemy has {} towers", 
            if player_team == "radiant" { radiant_towers } else { dire_towers },
            if player_team == "radiant" { dire_towers } else { radiant_towers });
        
        // Assessment
        if tower_diff >= 3 {
            println!("  ✅ Strong map control advantage. Consider aggressive warding.");
        } else if tower_diff >= 1 {
            println!("  ✅ Slight map control advantage. Maintain pressure.");
        } else if tower_diff == 0 {
            println!("  ⚖️ Even map control. Focus on objectives.");
        } else if tower_diff >= -2 {
            println!("  ⚠️ Losing map control. Defend remaining towers.");
        } else {
            println!("  ⚠️ Significant map control disadvantage. Play defensively.");
        }
        
        // Tips based on map control
        if tower_diff < 0 {
            println!("  💡 Tip: When behind in towers, focus on smoke ganks and pick-offs.");
        } else if tower_diff > 0 {
            println!("  💡 Tip: Use your map control to secure Roshan and invade jungle.");
        }
    } else {
        println!("  Building data not available");
    }
    
    println!("-------------------------------------");
}

/**
 * 4.3 Enemy Position Tracker
 * 
 * This feature tracks enemy positions over time from minimap data:
 */

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
        
        // Don't update if we've already processed this time
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
                        
                        // Add position to history
                        self.positions
                            .entry(hero_name)
                            .or_insert_with(Vec::new)
                            .push((current_game_time, (obj.xpos, obj.ypos)));
                        
                        // Limit history size (optional)
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
    
    fn display_enemy_movements(&self, game_time: i32) {
        println!("{}", "Enemy Movement Patterns:".cyan().bold());
        
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
                println!("  {}: Last seen {} seconds ago at ({}, {})", 
                    hero_name,
                    time_since_seen,
                    latest.1.0,
                    latest.1.1);
                
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
                    
                    println!("    Moving {}", direction);
                }
            }
        }
        
        println!("-------------------------------------");
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
}

/**
 * 4.4 Advanced Enemy Tracking
 * 
 * Complete implementation that combines all techniques:
 */

fn track_enemies(state: &GameState, tracker: &mut EnemyPositionTracker) {
    // Update tracker with new positions
    tracker.update(state);
    
    // Get current game time
    let game_time = state.map.as_ref()
        .and_then(|m| m.game_time)
        .unwrap_or(0);
    
    // Display enemy movements
    tracker.display_enemy_movements(game_time);
    
    // Show movement predictions
    let predictions = tracker.predict_movements(game_time);
    
    if !predictions.is_empty() {
        println!("{}", "Enemy Movement Predictions:".red().bold());
        
        for (hero_name, (x, y)) in predictions {
            println!("  {} likely at ({}, {})", hero_name, x, y);
            
            // Check if the predicted position is near player
            if let Some(hero) = &state.hero {
                if let (Some(player_x), Some(player_y)) = (hero.xpos, hero.ypos) {
                    let dx = x - player_x;
                    let dy = y - player_y;
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    
                    if distance < 2000.0 {
                        println!("    ⚠️ WARNING: This enemy may be very close to you!");
                    }
                }
            }
        }
        
        println!("-------------------------------------");
    }
}

/**
 * 5. ADVANCED FEATURES
 * ===================
 * 
 * More sophisticated coaching features that combine multiple data sources.
 */

/**
 * 5.1 Team Fight Analyzer
 * 
 * Analyzes team fight readiness and opportunities:
 */

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
    
    fn update(&mut self, state: &GameState, last_state: Option<&GameState>) {
        // Get current game time
        let game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Detect player death
        if let (Some(current_hero), Some(last_hero)) = (
            state.hero.as_ref(),
            last_state.and_then(|s| s.hero.as_ref())
        ) {
            let was_alive = last_hero.alive.unwrap_or(true);
            let is_alive = current_hero.alive.unwrap_or(true);
            
            if was_alive && !is_alive {
                self.player_deaths.push(game_time);
                self.last_kill_time = game_time;
            }
        }
        
        // Detect enemy deaths by looking at kill list changes
        if let (Some(current_player), Some(last_player)) = (
            state.player.as_ref(),
            last_state.and_then(|s| s.player.as_ref())
        ) {
            if let (Some(current_kills), Some(last_kills)) = (
                current_player.kill_list.as_ref(),
                last_player.kill_list.as_ref()
            ) {
                for (victim_id, kill_count) in current_kills {
                    let last_count = last_kills.get(victim_id).unwrap_or(&0);
                    
                    if kill_count > last_count {
                        // A new kill happened
                        if let Some(enemy_name) = self.get_enemy_name_from_id(victim_id) {
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
    
    fn get_enemy_name_from_id(&self, victim_id: &str) -> Option<String> {
        // In a real implementation, you would maintain a mapping
        // of victim IDs to hero names. This is a placeholder.
        Some(format!("Enemy{}", victim_id.replace("victimid_", "")))
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
    
    fn display_team_fight_status(&self, game_time: i32) {
        if self.team_fight_detected {
            println!("{}", "⚔️ TEAM FIGHT IN PROGRESS!".red().bold());
            println!("  Started {} seconds ago", game_time - self.team_fight_start);
        } else {
            // Check if a team fight might be coming soon
            let kills_in_window = self.count_kills_in_window(game_time, 60);
            
            if kills_in_window >= 2 {
                println!("{}", "⚠️ Skirmishes detected - team fight may be developing!".yellow().bold());
            }
        }
    }
}

/**
 * 5.2 Hero Performance Analyzer
 * 
 * Tracks performance metrics for your hero:
 */

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
    
    fn update(&mut self, state: &GameState, last_state: Option<&GameState>) {
        // Get current game time
        let game_time = state.map.as_ref()
            .and_then(|m| m.game_time)
            .unwrap_or(0);
        
        // Update GPM and XPM
        if let Some(player) = &state.player {
            if let Some(gpm) = player.gpm {
                self.gpm_samples.push((game_time, gpm));
            }
            
            if let Some(xpm) = player.xpm {
                self.xpm_samples.push((game_time, xpm));
            }
            
            if let Some(last_hits) = player.last_hits {
                self.last_hits_samples.push((game_time, last_hits));
            }
        }
        
        // Detect death and update counters
        if let (Some(current_hero), Some(last_hero)) = (
            state.hero.as_ref(),
            last_state.and_then(|s| s.hero.as_ref())
        ) {
            let was_alive = last_hero.alive.unwrap_or(true);
            let is_alive = current_hero.alive.unwrap_or(true);
            
            if was_alive && !is_alive {
                self.last_death_time = game_time;
                self.death_count += 1;
            }
        }
    }
    
    fn display_performance_metrics(&self, game_time: i32) {
        println!("{}", "Hero Performance Metrics:".yellow().bold());
        
        // GPM trend
        if self.gpm_samples.len() >= 2 {
            let current_gpm = self.gpm_samples.last().unwrap().1;
            let avg_gpm = self.gpm_samples.iter()
                .map(|(_, gpm)| gpm)
                .sum::<i32>() / self.gpm_samples.len() as i32;
            
            println!("  GPM: {} (Avg: {})", current_gpm, avg_gpm);
            
            if current_gpm > avg_gpm + 100 {
                println!("    ✅ GPM trending up significantly!");
            } else if current_gpm > avg_gpm + 20 {
                println!("    ✅ GPM trending up");
            } else if current_gpm < avg_gpm - 100 {
                println!("    ⚠️ GPM trending down significantly");
            } else if current_gpm < avg_gpm - 20 {
                println!("    ⚠️ GPM trending down");
            }
        }
        
        // XPM trend
        if self.xpm_samples.len() >= 2 {
            let current_xpm = self.xpm_samples.last().unwrap().1;
            println!("  XPM: {}", current_xpm);
        }
        
        // Last hit efficiency
        if self.last_hits_samples.len() >= 2 {
            let current_last_hits = self.last_hits_samples.last().unwrap().1;
            let minutes = game_time / 60;
            
            if minutes > 0 {
                let cs_per_min = current_last_hits as f32 / minutes as f32;
                println!("  CS/min: {:.1}", cs_per_min);
                
                // Benchmark CS/min (very rough benchmarks)
                if minutes < 10 {
                    if cs_per_min >= 7.0 {
                        println!("    ✅ Excellent early game CS");
                    } else if cs_per_min >= 5.0 {
                        println!("    ✅ Good early game CS");
                    } else if cs_per_min < 3.0 {
                        println!("    ⚠️ Early game CS needs improvement");
                    }
                } else {
                    if cs_per_min >= 8.0 {
                        println!("    ✅ Excellent CS");
                    } else if cs_per_min >= 6.0 {
                        println!("    ✅ Good CS");
                    } else if cs_per_min < 4.0 {
                        println!("    ⚠️ CS needs improvement");
                    }
                }
            }
        }
        
        // Death analysis
        if self.death_count > 0 {
            println!("  Deaths: {}", self.death_count);
            
            let minutes = game_time / 60;
            if minutes > 0 {
                let deaths_per_min = self.death_count as f32 / minutes as f32;
                
                if deaths_per_min > 0.2 {
                    println!("    ⚠️ High death rate, play more cautiously");
                }
            }
            
            let time_since_last_death = game_time - self.last_death_time;
            if time_since_last_death > 300 {  // 5 minutes
                println!("    ✅ Good survival streak: {} minutes without dying", 
                    time_since_last_death / 60);
            }
        } else {
            println!("  Deaths: 0 - Excellent survival!");
        }
        
        println!("-------------------------------------");
    }
}

/**
 * 6. TROUBLESHOOTING
 * =================
 * 
 * Common issues and solutions when working with Dota 2 GSI.
 */

/**
 * 6.1 No GSI Data Being Received
 * 
 * If your application is not receiving any GSI data:
 * 
 * 1. Verify GSI configuration file is correctly placed
 *    - Check the file path: [Steam]/steamapps/common/dota 2 beta/game/dota/cfg/
 *    - Make sure filename starts with "gamestate_integration_"
 *    - Check file contents match the example in this document
 * 
 * 2. Ensure Dota 2 is launched with -gamestateintegration flag
 *    - Check Steam launch options for Dota 2
 * 
 * 3. Verify your HTTP server is running and listening on the correct port
 *    - Try using a tool like curl to test your server: 
 *      curl -X POST http://127.0.0.1:3000/ -d '{}'
 * 
 * 4. Check firewall settings
 *    - Make sure your application can receive incoming connections
 * 
 * 5. Check console output
 *    - Look for any error messages in Dota 2 console (press ~ to open)
 *    - Add "developer 1" to Dota 2 console for more verbose output
 */

/**
 * 6.2 Debugging GSI Data
 * 
 * To debug GSI data issues:
 * 
 * 1. Add logging to your application
 *    - Log raw JSON received from Dota 2
 *    - Log parsing errors with details
 * 
 * 2. Implement a debug mode that saves GSI data to files
 *    - Save at regular intervals (e.g., every 30 seconds)
 *    - Save whenever parsing errors occur
 * 
 * Example debug logging function:
 */

fn debug_log_gsi_data(data: &Value, game_time: Option<i32>) {
    // Create debug directory if it doesn't exist
    std::fs::create_dir_all("gsi_debug").unwrap_or_else(|_| {
        eprintln!("Failed to create debug directory");
    });
    
    // Create filename with timestamp
    let time = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let game_time_str = game_time.map_or("unknown".to_string(), |t| t.to_string());
    let filename = format!("gsi_debug/gsi_data_{}_{}.json", time, game_time_str);
    
    // Write JSON data to file
    if let Err(e) = std::fs::write(&filename, data.to_string()) {
        eprintln!("Failed to write debug data to {}: {}", filename, e);
    } else {
        println!("Saved debug data to {}", filename);
    }
}

/**
 * 6.3 Performance Optimization
 * 
 * Tips for optimizing your coach application:
 * 
 * 1. Minimize UI updates
 *    - Only update when game state changes significantly
 *    - Throttle updates to a reasonable rate (e.g., 2-4 times per second)
 * 
 * 2. Use efficient data structures
 *    - Use appropriate collections for frequent lookups
 *    - Limit history size for time-series data
 * 
 * 3. Parallelize processing when possible
 *    - Use separate threads for data processing and UI rendering
 *    - Use async processing for non-blocking operations
 * 
 * 4. Profile and optimize hot code paths
 *    - Use #[inline] attribute for frequently called functions
 *    - Cache computation results when appropriate
 */

/**
 * 6.4 Data Validation
 * 
 * Best practices for handling GSI data:
 * 
 * 1. Always check for None values before using Option types
 *    - Use unwrap_or() or unwrap_or_else() with appropriate defaults
 *    - Use if let or match for conditional processing
 * 
 * 2. Validate data ranges and values
 *    - Check for unreasonable values that might indicate parsing issues
 *    - Handle edge cases (e.g., divide by zero)
 * 
 * 3. Implement fallbacks for missing data
 *    - Use historical data when current data is unavailable
 *    - Provide clear feedback when data is missing
 */

/**
 * CONCLUSION
 * ==========
 * 
 * This comprehensive reference document provides all the information needed
 * to implement a full-featured Dota 2 coach application using Game State
 * Integration. By using the data structures, implementation patterns, and
 * example features provided, you can create a custom coaching tool that
 * helps improve gameplay by providing real-time insights and feedback.
 * 
 * Key points to remember:
 * 
 * 1. Set up GSI configuration correctly in Dota 2
 * 2. Implement a robust HTTP server to receive GSI data
 * 3. Parse the data into appropriate structures
 * 4. Extract valuable insights from the data
 * 5. Present information in a clear, actionable format
* 6. Test and debug thoroughly with real game data
* 7. Optimize for performance and user experience
* 
* The Dota 2 GSI system provides real-time access to a wealth of game state 
* information that can be leveraged to create powerful coaching tools. By 
* focusing on extracting and presenting the most relevant information at 
* the right time, your coach application can provide valuable insights that 
* help improve gameplay decisions and overall performance.
* 
* Remember that coaching is most effective when it offers clear, actionable 
* advice without overwhelming the player. Focus on the most important insights, 
* prioritize immediate information, and save more detailed analysis for post-game 
* review.
* 
* With this comprehensive guide, you should be able to build, expand, and refine 
* your Dota 2 coaching application to meet your specific needs and preferences.
*/

     