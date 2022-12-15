use std::fs::{self, read_to_string};

use serde::Deserialize;



#[derive(Deserialize, Debug)]
struct Input {
    planets: Vec<Planet>,
    expeditions: Vec<Expedition>,
}

#[derive(Deserialize, Debug)]
struct Planet {
    ship_count: u64,
    x: f64,
    y: f64,
    owner: Option<u8>, 
    name: String
}

#[derive(Deserialize, Debug)]
struct Expedition {
    id: u64,
    ship_count: u64,
    origin: String,
    destination: String,
    owner: u8,
    turns_remaining: u64
}


fn main() {
    let input_str = fs::read_to_string("./resources/example_input.json").unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();
    eprintln!("{:#?}", input);
}
