use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

// Estrutura do usuário
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    password: String,
}

// Tipo para armazenar usuários em memória
type Users = Arc<Mutex<HashMap<u64, User>>>;

// HTML do frontend
const FRONTEND_HTML: &str = include_str!("index.html");

// Handler principal para as requisições
async fn handle_request(
    req: Request<Body>,
    users: Users,
) -> Result<Response<Body>, Infallible> {
    let response_builder = Response::builder()
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, PUT, DELETE")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type");

    match (req.method(), req.uri().path()) {
        // Servir o frontend
        (&Method::GET, "/") => {
            Ok(response_builder
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(FRONTEND_HTML))
                .unwrap())
        }

        // Criar usuário
        (&Method::POST, "/users") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let user: User = serde_json::from_slice(&body_bytes).unwrap();
            
            let mut users = users.lock().await;
            users.insert(user.id, user.clone());
            
            Ok(response_builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&user).unwrap()))
                .unwrap())
        }

        // Atualizar usuário
        (&Method::PUT, path) if path.starts_with("/users/") => {
            let id = path.trim_start_matches("/users/").parse::<u64>().unwrap();
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let mut user: User = serde_json::from_slice(&body_bytes).unwrap();
            user.id = id;
            
            let mut users = users.lock().await;
            users.insert(id, user.clone());
            
            Ok(response_builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&user).unwrap()))
                .unwrap())
        }

        // Deletar usuário
        (&Method::DELETE, path) if path.starts_with("/users/") => {
            let id = path.trim_start_matches("/users/").parse::<u64>().unwrap();
            let mut users = users.lock().await;
            users.remove(&id);
            
            Ok(response_builder
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap())
        }

        // Rota não encontrada
        _ => Ok(response_builder
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Usuario Criado com sucesso"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

    let users_ref = users.clone();
    let make_svc = make_service_fn(move |_conn| {
        let users = users_ref.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| handle_request(req, users.clone())))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Servidor rodando em http://localhost:3000");
    println!("Acesse http://localhost:3000 no navegador para usar a interface");

    if let Err(e) = server.await {
        eprintln!("Erro no servidor: {}", e);
    }
} 