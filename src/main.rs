use actix_web::{get, post, web, App, HttpRequest, HttpServer, Responder, Result};
use serde::Deserialize;
use std::{collections::HashMap, str::FromStr, sync::Mutex};

use crate::turtle::Command;

mod turtle;

async fn luafile(_: HttpRequest) -> impl Responder {
    let contents: String =
        std::fs::read_to_string("main.lua").expect("Something went wrong reading the file");
    contents
}

type Turtles = HashMap<String, turtle::Turtle>;

#[get("/request/{name}")]
async fn request(
    web::Path(name): web::Path<String>,
    turtles: web::Data<Mutex<Turtles>>,
) -> Result<String> {
    log::info!("Request received from turtle {}", name);
    let mut turtles = turtles.lock().unwrap();
    let turtle = turtles.get_mut(&name);
    if let Some(turtle) = turtle {
        let result = turtle.orders();
        return Ok(result);
    } else {
        turtles.insert(name, turtle::Turtle::default());
    }
    Ok(String::from("sleep(2)"))
}

#[derive(Deserialize)]
struct Info {
    info: String,
}

#[post("/info/{name}/{topic}")]
async fn add_information(
    web::Path((name, topic)): web::Path<(String, String)>,
    form: web::Form<Info>,
    turtles: web::Data<Mutex<Turtles>>,
) -> Result<String> {
    log::info!("Info received from turtle {}, topic: {}", name, topic);
    let mut turtles = turtles.lock().unwrap();
    let turtle = turtles.get_mut(&name);
    if let Some(turtle) = turtle {
        turtle.infos.insert(topic, form.info.clone());
    } else {
        let mut newturtle = turtle::Turtle::default();
        newturtle.infos.insert(topic, form.info.clone());
        turtles.insert(name, newturtle);
    }
    Ok(String::new())
}

#[get("/pos/{name}")]
async fn get_position(
    web::Path(name): web::Path<String>,
    turtles: web::Data<Mutex<Turtles>>,
) -> Result<String> {
    let turtles = turtles.lock().unwrap();
    let turtle = turtles.get(&name);
    if let Some(turtle) = turtle {
        Ok(turtle.get_position())
    } else {
        Ok(String::from("Missing turtle"))
    }
}

#[get("/info/{name}/{topic}")]
async fn get_information(
    web::Path((name, topic)): web::Path<(String, String)>,
    turtles: web::Data<Mutex<Turtles>>,
) -> Result<String> {
    log::info!("Info received from turtle {}, topic: {}", name, topic);
    let mut turtles = turtles.lock().unwrap();
    let turtle = turtles.get_mut(&name);
    if let Some(turtle) = turtle {
        Ok(turtle
            .infos
            .get(&topic)
            .unwrap_or(&String::from("N/A"))
            .clone())
    } else {
        Ok(String::new())
    }
}

#[derive(Deserialize)]
struct Orders {
    orders: String,
}

#[post("/order/{name}")]
async fn add_orders(
    web::Path(name): web::Path<String>,
    form: web::Form<Orders>,
    turtles: web::Data<Mutex<Turtles>>,
) -> impl Responder {
    log::info!("Adding orders for {}", name);
    let orders: Vec<(Command, u32)> = form
        .orders
        .split('\n')
        .map(|order| {
            let order_split: Vec<_> = order.split(',').collect();
            (
                Command::from_str(order_split[0])
                    .unwrap_or_else(|_| panic!("Unable to parse order {}", order)),
                order_split[1]
                    .parse::<u32>()
                    .unwrap_or_else(|_| panic!("Unable to parse order: {}", order)),
            )
        })
        .collect();
    let mut turtles = turtles.lock().unwrap();
    let turtle = turtles.get_mut(&name);
    if let Some(turtle) = turtle {
        turtle.orders = orders;
        "ok"
    } else {
        "turtle not found"
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();
    let turtles: web::Data<Mutex<Turtles>> = web::Data::new(Mutex::new(HashMap::new()));
    log::info!("Starting http server on port 8787");
    HttpServer::new(move || {
        App::new()
            .app_data(turtles.clone())
            .route("luafile", web::get().to(luafile))
            .service(request)
            .service(add_orders)
            .service(add_information)
            .service(get_information)
            .service(get_position)
    })
    .bind(("0.0.0.0", 8787))?
    .run()
    .await
}
