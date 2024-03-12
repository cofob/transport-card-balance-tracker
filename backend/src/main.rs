use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
    sync::RwLock,
    time::sleep,
};

struct AppState {
    balances: Arc<RwLock<HashMap<String, u16>>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SetBalance {
    pub balance: u16,
}

#[get("/balance/{id}")]
async fn get_balance(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let balance = {
        let balances = state.balances.read().await;
        *balances.get(&id).unwrap_or(&0)
    };
    HttpResponse::Ok().body(balance.to_string())
}

#[post("/balance/{id}")]
async fn set_balance(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Query<SetBalance>,
) -> impl Responder {
    let id = path.into_inner();
    let mut balances = state.balances.write().await;
    if body.balance == 0 {
        balances.remove(&id);
    } else {
        balances.insert(id.clone(), body.balance);
    }
    HttpResponse::Ok().body("OK")
}

async fn save_balances(balances: Arc<RwLock<HashMap<String, u16>>>) -> anyhow::Result<()> {
    let database_path = Path::new("database.json");

    let mut file = File::create(database_path).await?;
    file.write_all(
        serde_json::to_string(&*balances.read().await)
            .unwrap()
            .as_bytes(),
    )
    .await?;
    file.flush().await?;

    Ok(())
}

async fn save_balances_loop(balances: Arc<RwLock<HashMap<String, u16>>>) {
    loop {
        sleep(Duration::from_secs(5)).await;
        save_balances(balances.clone()).await.ok();
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get host and port from environment
    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();

    // Check if database.json exists
    let database_path = Path::new("database.json");
    if !database_path.exists() {
        // Create database.json
        let mut file = File::create("database.json").await.unwrap();
        file.write_all(b"{}").await.unwrap();
    }

    // Load database.json
    let database = read_to_string(database_path).await.unwrap();
    let database: HashMap<String, u16> = serde_json::from_str(&database).unwrap();
    let balances = Arc::new(RwLock::new(database));

    // Save balances every 5 seconds
    tokio::spawn(save_balances_loop(balances.clone()));

    println!("Server started on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                balances: balances.clone(),
            }))
            .service(get_balance)
            .service(set_balance)
    })
    .bind((host, port))?
    .run()
    .await
}
