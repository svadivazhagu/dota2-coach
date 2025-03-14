// src/main.rs
use color_eyre::Result;

mod app;
mod event;
mod server;
mod state;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup error handling
    color_eyre::install()?;

    println!("Starting Dota 2 Coach...");
    println!("Make sure you have configured the GSI config file in Dota 2.");
    println!("Remember to add -gamestateintegration to Dota 2 launch options");

    // Initialize terminal
    let terminal = ratatui::init();

    // Create application state
    let app = App::new();
    
    // Start the server to receive Dota 2 GSI data
    let game_state_clone = app.game_state.clone();
    let last_game_state_clone = app.last_game_state.clone();
    let _server_handle = server::start_server(game_state_clone, last_game_state_clone).await;
    
    println!("Server running on http://127.0.0.1:3000");
    println!("Waiting for Dota 2 game data...");
    
    // Run the main application loop
    let result = app.run(terminal).await;
    
    // Restore terminal
    ratatui::restore();
    
    result
}