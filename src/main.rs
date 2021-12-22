use actix_web::{get, post, web, App, HttpRequest, HttpServer, Responder, Result};
use mining_plots::MiningPlot;
use mongodb::{
    bson::{self, doc},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Mutex};

mod mining_plots;
mod persistance;
mod turtle;
mod utils;

use crate::{
    persistance::find_one_tutle,
    turtle::{Command, CommandName, Turtle},
};

async fn luafile(_: HttpRequest) -> impl Responder {
    let contents: String =
        std::fs::read_to_string("main.lua").expect("Something went wrong reading the file");
    contents
}

type Turtles = Collection<Turtle>;
type MiningPlots = Collection<MiningPlot>;

#[get("/request/{name}")]
async fn request(
    path: web::Path<String>,
    turtles: web::Data<Mutex<Turtles>>,
    mining_plots: web::Data<Mutex<MiningPlots>>,
) -> Result<String> {
    let name = path.into_inner();
    log::info!("Request received from turtle {}", name);
    let turtles = turtles.lock().unwrap();
    let mining_plots = mining_plots.lock().unwrap();
    let turtle = find_one_tutle(&turtles, &name).await;
    if let Some(mut turtle) = turtle {
        let result = turtle.orders(&mining_plots).await;
        let _ = turtles
            .find_one_and_replace(doc! { "name": &name }, &turtle, None)
            .await;
        return Ok(result);
    } else {
        let _ = turtles
            .insert_one(turtle::Turtle::default(name), None)
            .await
            .expect("Unable to insert new turtle");
    }
    Ok(String::from("sleep(2)"))
}

#[derive(Deserialize)]
struct Info {
    info: String,
}

#[post("/info/{name}/{topic}")]
async fn add_information(
    path: web::Path<(String, String)>,
    form: web::Form<Info>,
    turtles: web::Data<Mutex<Turtles>>,
) -> impl Responder {
    let (name, topic) = path.into_inner();
    log::info!("Info received from turtle {}, topic: {}", name, topic);
    let turtles = turtles.lock().unwrap();
    let result = turtles
        .update_one(
            doc! { "name": &name},
            doc! { "$set": { "infos": { topic: &form.info } } },
            None,
        )
        .await
        .expect("Unable to save update turtle infos");
    if result.matched_count == 1 {
        "ok"
    } else {
        "Turtle not found"
    }
}

// #[get("/pos/{name}")]
// async fn get_position(
//     web::Path(name): web::Path<String>,
//     turtles: web::Data<Mutex<Turtles>>,
// ) -> Result<String> {
//     let turtles = turtles.lock().unwrap();
//     let turtle = turtles.get(&name);
//     if let Some(turtle) = turtle {
//         Ok(turtle.get_position())
//     } else {
//         Ok(String::from("Missing turtle"))
//     }
// }

#[get("/info/{name}/{topic}")]
async fn get_information(
    path: web::Path<(String, String)>,
    turtles: web::Data<Mutex<Turtles>>,
) -> Result<String> {
    let (name, topic) = path.into_inner();
    log::info!("Info received from turtle {}, topic: {}", name, topic);
    let turtles = turtles.lock().unwrap();
    let turtle = find_one_tutle(&turtles, &name).await;
    if let Some(turtle) = turtle {
        Ok(turtle
            .infos
            .get(&topic)
            .unwrap_or(&String::from("N/A"))
            .clone())
    } else {
        Ok(format!("No turtle with name: {} found", name))
    }
}

#[derive(Deserialize)]
struct Orders {
    orders: String,
}

#[post("/order/{name}")]
async fn add_orders(
    path: web::Path<String>,
    form: web::Form<Orders>,
    turtles: web::Data<Mutex<Turtles>>,
) -> impl Responder {
    let name = path.into_inner();
    log::info!("Adding orders for {}", name);
    let orders: Vec<Command> = form
        .orders
        .split('\n')
        .map(|order| {
            let order_split: Vec<_> = order.split(',').collect();
            Command::new(
                CommandName::from_str(order_split[0])
                    .unwrap_or_else(|_| panic!("Unable to parse order {}", order)),
                order_split[1]
                    .parse::<u32>()
                    .unwrap_or_else(|_| panic!("Unable to parse order: {}", order)),
            )
        })
        .collect();
    let turtles = turtles.lock().unwrap();
    let result = turtles
        .update_one(
            doc! { "name": &name },
            doc! { "$set": { "orders": bson::to_bson(&orders).expect("Unable to convert orders to bson") } },
            None,
        )
        .await
        .expect("Unable to update orders of the turtle");
    if result.matched_count == 1 {
        "ok"
    } else {
        "Turtle not found"
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init_timed();
    let (turtles_db, mining_plots_db) = persistance::connect().await;
    let turtles: web::Data<Mutex<Turtles>> = web::Data::new(Mutex::new(turtles_db));
    let mining_plots: web::Data<Mutex<MiningPlots>> = web::Data::new(Mutex::new(mining_plots_db));
    log::info!("Starting http server on port 8787");
    HttpServer::new(move || {
        App::new()
            .app_data(turtles.clone())
            .app_data(mining_plots.clone())
            .route("luafile", web::get().to(luafile))
            .service(request)
            .service(add_orders)
            .service(add_information)
            .service(get_information)
        // .service(get_position)
    })
    .bind(("0.0.0.0", 8787))?
    .run()
    .await
}
