use std::{io::{self, BufRead}, time::{Instant, Duration}, collections::BTreeMap};

use bit_set::BitSet;
use itertools::Itertools;
use serde::{Deserialize, Serialize};


type PlanetName = String;
type ExpeditionId = u64;
type PlayerId = u8;

#[derive(Deserialize, Debug, Clone)]
struct Input {
    planets: Vec<Planet>,
    expeditions: Vec<Expedition>,
}

#[derive(Deserialize, Debug, Clone)]
struct Planet {
    ship_count: i64,
    x: f64,
    y: f64,
    owner: Option<PlayerId>, 
    name: PlanetName
}

#[derive(Deserialize, Debug, Clone)]
struct PlanetLocation {
    x: f64,
    y: f64,
    name: PlanetName
}

impl From<Planet> for PlanetLocation {
    fn from(planet: Planet) -> Self {
        let Planet{x, y, name, ..} = planet;
        PlanetLocation { x, y, name } 
    }
}


#[derive(Deserialize, Debug, Clone)]
struct Expedition {
    id: ExpeditionId,
    ship_count: i64,
    origin: PlanetName,
    destination: PlanetName,
    owner: PlayerId,
    turns_remaining: i64
}

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    origin: PlanetName,
    destination: PlanetName,
    ship_count: i64,
}

const MAX_TURNS: u64 = 500;
const HARD_MAX_DURATION: u64 = 1000;
const MAX_DURATION: u64 = 800;

#[derive(Clone, Debug)]
struct StateCell {
    deltas: Vec<(PlayerId, i64)>,
}

#[derive(Clone, Debug)]
struct State {
    state: Vec<Vec<StateCell>>,
    current_state: Input,
    saved_expeditions: BitSet, 
    planet_map: BTreeMap<PlanetName, usize>,
    turn: u64,
}

impl State {
    fn tick(&mut self) {
        self.turn += 1;
    }

    fn get_state_cell(&mut self, planet_name: &PlanetName, turns_ahead: u64) -> &mut StateCell {
        let planet_index = self.planet_map.get(planet_name).unwrap();
        // TODO: check if correct or if it is off by one 
        let turn_index = self.turn + turns_ahead;
        &mut self.state[turn_index as usize][*planet_index]
    }


    fn predict_planets(&self, turns_ahead: u64) -> &Vec<(Option<PlayerId>, i64)> {
        let turn_index = self.turn + turns_ahead;
        // &self.state[turn_index as usize]
    }

    // TODO: test correctness
    fn predict_planet(&self, turns_ahead: u64, planet_name: PlanetName) -> (Option<PlayerId>, i64) {
        let turn_index = (self.turn + turns_ahead) as usize;
        let planet_index = *self.planet_map.get(&planet_name).unwrap();
        let planet_state = self.current_state.planets
            .iter()
            .find(|planet| planet.name == planet_name )
            .unwrap();
        let mut count = planet_state.ship_count;
        let mut owner = planet_state.owner;
        let mut map = BTreeMap::new();
        for i in (self.turn as usize)..=turn_index {
            map.clear();
            map.insert(&owner, count);

            let deltas = &self.state[turn_index][planet_index].deltas;
            for &(owner, amount) in deltas {
                let key = Some(owner);
                map.entry(&key)
                    .and_modify(|val| *val += amount)
                    .or_insert(amount);
            }

            let key_values = map.iter()
                .sorted_by_key(|(k, &v)| -v) // sorts by ascending order, so for negative value
                // sorts descending order
                .take(2)
                .collect_vec();

            let (largest_owner, largest_count) = key_values[0];
            if key_values.len() == 1 {
                (owner, count) = (**largest_owner, *largest_count);
            } else {
                let (next_owner, next_count) = key_values[1];
                count = largest_count - next_count;
                if count == 0 {
                    owner = None;
                } else {
                    owner = **largest_owner;
                }
            }
        }

        (owner, count)
    }

    fn check_gameover(&self) -> GameSituation {
        //TODO: look at next turn
        let ship_counts = self.get_planet_future(1)
            .iter()
            .into_grouping_map_by(|planet| planet.planet.owner)
            .fold(0, |acc, _key, val| acc + val.planet.ship_count);

        eprintln!("{:?}", ship_counts);
        if ship_counts[&Some(1)] == 0 {
            return GameSituation::LOST;
        } 

        let other_key = ship_counts.keys()
            .into_iter()
            .find(|key| key.is_some())
            .unwrap();
        if ship_counts[other_key] == 0 {
            return GameSituation::WON;
        }
        return GameSituation::ONGOING; 
    } 

    fn new(planets: &[Planet]) -> Self {
        let mut entry = vec![];
        let mut planet_map = BTreeMap::new();

        for (index, planet) in planets.iter().enumerate() {
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
        State {
            state: state_vec,
            planet_map,
            saved_expeditions: BitSet::new(),
            turn: 0,
        }
    }

    fn update(&mut self, input: &Input) {
        for expedition in &input.expeditions {
            if self.saved_expeditions.contains(expedition.id as usize) {
                continue;
            } 
            self.saved_expeditions.insert(expedition.id as usize);
            let state_cell = self.get_state_cell(&expedition.destination, expedition.turns_remaining);
            state_cell.deltas.push(
                (expedition.owner, expedition.ship_count as i64)
            );
        }
        self.state = input;
    }
}

enum GameSituation {
    WON,
    LOST,
    ONGOING,
}

fn main() {

    let stdin = io::stdin();
    let mut state = State {
        state: vec![],
        saved_expeditions: BitSet::new(),
        planet_map: BTreeMap::new(),
        turn: 0,
    };


    for line in stdin.lock().lines() {
        let now = Instant::now();
        let line = line.unwrap();
        eprintln!("=========================================================");
        // eprintln!("{}", line);
        let input: Input = serde_json::from_str(&line).unwrap();

        
        if state.turn == 0 {
            state = State::new(&input.planets);
        }

        state.update(&input);

        let output = Output {
            moves: vec![]
        };

        // while now.elapsed() < Duration::from_millis(MAX_DURATION) {
        //     //TODO: do things
        //
        // }
        println!("{}", serde_json::to_string(&output).unwrap());
        state.tick();
    }
}
