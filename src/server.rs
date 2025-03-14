// src/server.rs
use std::sync::{Arc, Mutex};
use warp::Filter;
use serde_json::Value;
use tokio::task::JoinHandle;
use crate::state::GameState;

pub async fn start_server(
    game_state: Arc<Mutex<Option<GameState>>>,
    last_game_state: Arc<Mutex<Option<GameState>>>,
) -> JoinHandle<()> {
    // Clone for the endpoint closure
    let game_state_clone = game_state.clone();
    let last_game_state_clone = last_game_state.clone();
    
    // Set up an endpoint to receive GSI data
    let gsi_endpoint = warp::post()
        .and(warp::body::content_length_limit(1024 * 1024 * 10))
        .and(warp::body::json())
        .map(move |data: Value| {
            // Convert the incoming JSON to GameState struct
            match serde_json::from_value::<GameState>(data.clone()) {
                Ok(state) => {
                    // Update last game state before setting current
                    let current_gs = {
                        let gs = game_state_clone.lock().unwrap();
                        gs.clone()
                    };
                    
                    // Store the last game state
                    {
                        let mut last_gs = last_game_state_clone.lock().unwrap();
                        *last_gs = current_gs;
                    }
                    
                    // Store the new game state
                    {
                        let mut gs = game_state_clone.lock().unwrap();
                        *gs = Some(state);
                    }
                },
                Err(e) => {
                    eprintln!("Error parsing game state: {}", e);
                }
            }
            
            "OK"
        });
    
    // Spawn the server in a new task
    tokio::spawn(async move {
        warp::serve(gsi_endpoint)
            .run(([127, 0, 0, 1], 3000))
            .await;
    })
}