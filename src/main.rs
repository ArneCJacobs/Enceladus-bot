use std::{fs::{self, read_to_string}, io::{stdin, self, BufRead}, time::{Instant, Duration}, thread::sleep, collections::HashMap};

use bit_set::BitSet;
use serde::{Deserialize, Serialize};


type PlanetName = String;
type ExpeditionId = u64;

#[derive(Deserialize, Debug)]
struct Input {
    planets: Vec<Planet>,
    expeditions: Vec<Expedition>,
}

#[derive(Deserialize, Debug, Clone)]
struct Planet {
    ship_count: u64,
    x: f64,
    y: f64,
    owner: Option<u8>, 
    name: PlanetName
}

#[derive(Deserialize, Debug)]
struct Expedition {
    id: ExpeditionId,
    ship_count: u64,
    origin: PlanetName,
    destination: PlanetName,
    owner: u8,
    turns_remaining: u64
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    origin: PlanetName,
    destination: PlanetName,
    ship_count: u64,
}

const MAX_TURNS: u64 = 500;
const HARD_MAX_DURATION: u64 = 1000;
const MAX_DURATION: u64 = 800;

#[derive(Clone, Debug)]
struct StateCell {
    planet: Planet,
    deltas: Vec<i64>,
}

#[derive(Clone, Debug)]
struct State {
    state: Vec<Vec<StateCell>>,
    saved_expeditions: BitSet, 
    planet_map: HashMap<PlanetName, usize>,
    turn: u64,
}

impl State {
    fn tick(&mut self) {
        self.turn += 1;
    }

    fn getStateCell(&mut self, planet_name: &PlanetName, turns_ahead: u64) -> &mut StateCell {
        let planet_index = self.planet_map.get(planet_name).unwrap();
        // TODO: check if correct or if it is off by one 
        let turn_index = self.turn + turns_ahead;
        &mut self.state[turn_index as usize][*planet_index]
    }
}

fn main() {

    let stdin = io::stdin();
    let mut state = State {
        state: vec![],
        saved_expeditions: BitSet::new(),
        planet_map: HashMap::new(),
        turn: 0,
    };


    for line in stdin.lock().lines() {
        let now = Instant::now();
        let line = line.unwrap();
        let mut planet_map = HashMap::new();
        eprintln!("=========================================================");
        // eprintln!("{}", line);
        let input: Input = serde_json::from_str(&line).unwrap();
        if state.turn == 0 {
            // TODO: make function of State, accepting Input
            let mut entry = vec![];
            for (index, planet) in input.planets.iter().enumerate() {
                entry.push(
                    StateCell {
                        planet: planet.clone(),
                        deltas: vec![],
                    }
                );

                planet_map.insert(planet.name.clone(), index);
            }

            let mut state_vec = vec![];
            for _ in 0..MAX_TURNS {
                state_vec.push(entry.clone());
            }
            state = State {
                state: state_vec,
                planet_map,
                saved_expeditions: BitSet::new(),
                turn: 0,
            };
        }

        // TODO: make function of State
        for expedition in input.expeditions {
            if state.saved_expeditions.contains(expedition.id as usize) {
                continue;
            } 

            let factor = if expedition.owner == 1 {
                1
            } else {
                -1
            };

            let state_cell = state.getStateCell(&expedition.destination, expedition.turns_remaining);
            state_cell.deltas.push(factor * (expedition.ship_count as i64));
        }

        let output = Output {
            moves: vec![]
        };

        while now.elapsed() < Duration::from_millis(MAX_DURATION) {
            //TODO: do things

        }
        println!("{}", serde_json::to_string(&output).unwrap());
        state.tick();
    }
}
