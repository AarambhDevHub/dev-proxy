use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    println!("🚀 Example Backend starting on http://0.0.0.0:8000");

    // GET /api/users
    let get_users = warp::path!("api" / "users")
        .and(warp::get())
        .map(|| {
            let users = vec![
                User {
                    id: 1,
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                },
                User {
                    id: 2,
                    name: "Bob".to_string(),
                    email: "bob@example.com".to_string(),
                },
            ];
            warp::reply::json(&users)
        });

    // GET /api/users/:id
    let get_user = warp::path!("api" / "users" / u32)
        .and(warp::get())
        .map(|id: u32| {
            let user = User {
                id,
                name: format!("User {}", id),
                email: format!("user{}@example.com", id),
            };
            warp::reply::json(&user)
        });

    // POST /api/users
    let create_user = warp::path!("api" / "users")
        .and(warp::post())
        .and(warp::body::json())
        .map(|user: CreateUser| {
            let new_user = User {
                id: 3,
                name: user.name,
                email: user.email,
            };
            warp::reply::json(&new_user)
        });

    // Health check
    let health = warp::path!("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    let routes = get_users
        .or(get_user)
        .or(create_user)
        .or(health);

    warp::serve(routes)
        .run(([0, 0, 0, 0], 8000))
        .await;
}
