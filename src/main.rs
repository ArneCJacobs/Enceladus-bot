use std::{io::{self, BufRead}, time::Instant, collections::BTreeMap};

use bit_set::BitSet;
use itertools::Itertools;
use prettytable::{Table, Cell, Row};
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
    planet_names: Vec<String>,
    turn: i64,
}

impl State {
    fn tick(&mut self) {
        self.turn += 1;
    }

    fn get_state_cell(&mut self, planet_name: &PlanetName, turns_ahead: i64) -> &mut StateCell {
        let planet_index = self.planet_map.get(planet_name).unwrap();
        // TODO: check if correct or if it is off by one 
        let turn_index = (self.turn + turns_ahead) as usize;
        &mut self.state[turn_index as usize][*planet_index]
    }


    fn predict_planets(&self, turns_ahead: i64) -> Vec<(Option<PlayerId>, i64)> {
        // let turn_index = self.turn + turns_ahead;
        let mut return_vec = Vec::new();
        for planet_name in &self.planet_names {
            return_vec.push(
                self.predict_planet(turns_ahead, &planet_name)
            );
        }
        return_vec
        // &self.state[turn_index as usize]
    }

    // TODO: test correctness
    fn predict_planet(&self, turns_ahead: i64, planet_name: &PlanetName) -> (Option<PlayerId>, i64) {
        let planet_name = planet_name.clone();
        let turn_index = (self.turn + turns_ahead) as usize;
        let planet_index = *self.planet_map.get(&planet_name).unwrap();
        let planet_state = self.current_state.planets
            .iter()
            .find(|planet| planet.name == planet_name )
            .unwrap();
        let mut current_owner = planet_state.owner;
        let mut current_count = planet_state.ship_count;
        let mut map: BTreeMap<Option<u8>, i64> = BTreeMap::new();
        for i in (self.turn as usize)..=turn_index {
            map.clear();
            // if not checking the current state and the owner of the planet is a player, then
            // ship count will have grown with 1 
            if (i as i64) != self.turn && current_owner.is_some() {
                map.insert(current_owner, current_count + 1);
            } else {
                map.insert(current_owner, current_count);
            }

            let deltas = &self.state[i][planet_index].deltas;
            for &(owner, amount) in deltas {
                let key = Some(owner);
                let value = map.get(&key);
                if planet_name == "protos" && i < 12 {
                    eprintln!("{} {:?}", i, value);
                }
                map.insert(key, value.unwrap_or(&0) + amount);
            }

            let key_values = map.iter()
                .sorted_by_key(|(_, &v)| -v) // sorts by ascending order, so for negative value
                // sorts descending order
                .take(2)
                .collect_vec();


            if planet_name == "protos" && i == 12 {
                eprintln!("{}, {:?}",i, key_values);
            }

            let (largest_owner, largest_count) = key_values[0];
            if key_values.len() == 1 {
                (current_owner, current_count) = (*largest_owner, *largest_count);
            } else {
                let (_, next_count) = key_values[1];
                current_count = largest_count - next_count;
                if current_count == 0 {
                    current_owner = None;
                } else {
                    current_owner = *largest_owner;
                }
            }
        }

        (current_owner, current_count)
    }

    fn check_gameover(&self) -> GameSituation {
        // TODO: currently only checks units on planets and does not take expeditions into account 
        let binding = self.predict_planets(1);
        let ship_counts = binding
            .iter()
            .into_grouping_map_by(|(owner, _)| owner)
            .fold(0, |acc, _key, (_, val)| acc + val);

        let temp = self.current_state.planets
            .iter()
            .into_grouping_map_by(|planet| planet.owner)
            .fold(0, |acc, _key, val| acc + val.ship_count);
        // eprintln!("current   : {:?}", self.current_state.expeditions);
        // eprintln!("current   : {:?}", temp);
        // eprintln!("prediction: {:?}", ship_counts);
        if !ship_counts.contains_key(&Some(1)) {
            return GameSituation::LOST;
        } 

        let other_key = ship_counts.keys()
            .into_iter()
            .find(|&&&key| key.is_some() && key != Some(1));
        if other_key.is_none() {
            return GameSituation::WON;
        }
        GameSituation::ONGOING 
    } 

    fn new(input: Input) -> Self {
        let mut entry = vec![];
        let mut planet_map = BTreeMap::new();
        let mut planet_names = vec![];

        for (index, planet) in input.planets.iter().enumerate() {
            entry.push(
                StateCell {
                    deltas: vec![],
                }
            );

            planet_map.insert(planet.name.clone(), index);
            planet_names.push(planet.name.clone());
        }

        let mut state_vec = vec![];
        for _ in 0..MAX_TURNS {
            state_vec.push(entry.clone());
        }
        State {
            planet_names,
            current_state: input,
            state: state_vec,
            planet_map,
            saved_expeditions: BitSet::new(),
            turn: 0,
        }
    }

    fn update(&mut self, input: Input) {
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
        self.current_state = input;
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
        current_state: Input { planets: vec![], expeditions: vec![] },
        planet_names: vec![],
        state: vec![],
        saved_expeditions: BitSet::new(),
        planet_map: BTreeMap::new(),
        turn: 0,
    };


    for line in stdin.lock().lines() {

        if state.turn > 1 {
            break;
        }
        let now = Instant::now();
        let line = line.unwrap();
        eprintln!("=========================================================");
        // eprintln!("{}", line);
        let input: Input = serde_json::from_str(&line).unwrap();
        if state.turn == 0 {
            state = State::new(input);
        } else {
            state.update(input);
        }
        let mut table = Table::new();
        let mut header = vec![Cell::new("turn")];
        for planet in &state.planet_names {
            header.push(Cell::new(&planet.clone()));
        }
        table.set_titles(Row::new(header));
        for i in 0..=12 {
            let mut predict_row = vec![Cell::new(&(state.turn + i).to_string())];
            for planet_name in &state.planet_names {
                let (owner, amount) = state.predict_planet(i, planet_name);
                let cell = Cell::new(
                    &format!("{:?}\n{:?}", owner, amount)
                );
                predict_row.push(cell);
            }
            table.add_row(Row::new(predict_row));
        }
        eprintln!("{:?}", state.current_state.expeditions);
        eprintln!("{}", table);

        match state.check_gameover() {
            GameSituation::WON => eprintln!("WE HAVE WON!"),
            GameSituation::LOST => eprintln!("WE HAVE LOST!"),
            _ => {}
        }


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
