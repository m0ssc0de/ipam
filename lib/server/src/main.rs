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
            std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<nodemng::NodeMNG<'static>>>>,
        >,
        web::Data<std::sync::mpsc::Sender<bool>>,
        web::Path<(String,)>,
    ),
) -> impl Responder {
    println!("start");
    tx.send(true);
    match rx.lock() {
        Ok(rx) => match rx.recv() {
            Ok(mut mng) => match mng.get_node(&path.0) {
                Ok(cfg) => HttpResponse::Ok(),
                Err(e) => {
                    println!("{}", e);
                    HttpResponse::InternalServerError()
                }
            },
            Err(e) => {
                println!("{}", e);
                HttpResponse::InternalServerError()
            }
        },
        Err(e) => {
            println!("{}", e);
            HttpResponse::InternalServerError()
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
    let mnger = nodemng::NodeMNG::new(
        Path::new("./ca.crt").clone(),
        Path::new("./ca.key").clone(),
        Path::new("./config.yml").clone(),
        None,
    )
    .unwrap();
    // let mnger = Arc::new(Mutex::new(mnger));
    let (tx0, rx0): (Sender<nodemng::NodeMNG>, Receiver<nodemng::NodeMNG>) = mpsc::channel();
    let rx0 = Arc::new(Mutex::new(rx0));
    let (tx1, rx1): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    // let rx1 = Arc::new(Mutex::new(rx1));
    std::thread::spawn(move || loop {
        match rx1.recv() {
            Ok(_) => tx0.send(mnger),
            Err(e) => println!("the thread receive error: {}", e),
        }
    });
    HttpServer::new(move || {
        App::new()
            .data(appstate.clone())
            // .data(mnger.clone())
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
