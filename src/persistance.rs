use mongodb::bson::doc;
use mongodb::Collection;
use mongodb::{options::ClientOptions, Client};

use crate::mining_plots::MiningPlot;
use crate::turtle::Turtle;

pub async fn connect() -> (mongodb::Collection<Turtle>, mongodb::Collection<MiningPlot>) {
    let client_options = ClientOptions::parse("mongodb://root:example@localhost:27017")
        .await
        .expect("Unable to connect to the database");
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("cc-api");

    (
        db.collection::<Turtle>("turtles"),
        db.collection::<MiningPlot>("miningplot"),
    )
}

pub async fn find_one_tutle<'a>(turtles: &Collection<Turtle>, name: &str) -> Option<Turtle> {
    turtles
        .find_one(doc! { "name": &name }, None)
        .await
        .expect("DB error: Unable to get turtles")
}
