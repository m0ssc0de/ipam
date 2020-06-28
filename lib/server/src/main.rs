use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use nodemng;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

async fn index(state: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    println!("{}", state.lock().unwrap().add());
    HttpResponse::Ok()
}
async fn new_node(
    (req, rx, tx, path): (
        HttpRequest,
        web::Data<
            std::sync::Arc<
                std::sync::Mutex<
                    std::sync::mpsc::Receiver<
                        std::result::Result<std::string::String, nodemng::NodeError>,
                    >,
                >,
            >,
        >,
        web::Data<std::sync::mpsc::Sender<std::string::String>>,
        web::Path<(String,)>,
    ),
) -> impl Responder {
    println!("start");
    tx.send(path.0.clone());
    match rx.lock() {
        Ok(rx) => match rx.recv() {
            Ok(r) => match r {
                Ok(cfg) => HttpResponse::Ok().body(cfg),
                Err(e) => {
                    println!("{}", e);
                    HttpResponse::InternalServerError().body(format!("{}", e))
                }
            },
            Err(e) => {
                println!("{}", e);
                HttpResponse::InternalServerError().body(format!("{}", e))
            }
        },
        Err(e) => {
            println!("{}", e);
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

struct AppState {
    size: i32,
}

impl AppState {
    fn add(&mut self) -> i32 {
        self.size = self.size + 1;
        self.size
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let appstate = Arc::new(Mutex::new(AppState { size: 10 }));
    let mut mnger = nodemng::NodeMNG::new(
        Path::new("./ca.crt").clone(),
        Path::new("./ca.key").clone(),
        Path::new("./config.yml").clone(),
        None,
    )
    .unwrap();
    let (tx0, rx0): (
        Sender<Result<String, nodemng::NodeError>>,
        Receiver<Result<String, nodemng::NodeError>>,
    ) = mpsc::channel();
    let rx0 = Arc::new(Mutex::new(rx0));
    let (tx1, rx1): (Sender<String>, Receiver<String>) = mpsc::channel();
    std::thread::spawn(move || loop {
        match rx1.recv() {
            Ok(name) => {
                tx0.send(mnger.get_node(&name));
            }
            Err(e) => {
                println!("the thread receive error: {}", e);
                continue;
            }
        }
    });
    HttpServer::new(move || {
        App::new()
            .data(appstate.clone())
            .data(rx0.clone())
            .data(tx1.clone())
            .wrap(middleware::Logger::default())
            .route("/hello", web::get().to(index))
            .route("/new/node/{name}", web::post().to(new_node))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
