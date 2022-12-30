
use std::collections::BTreeMap;

use bit_set::BitSet;
use itertools::Itertools;
use prettytable::{Table, Cell, Row};
use priq::PriorityQueue;

use crate::{MAX_TURNS, structs::{GameSituation, Input, PlayerId, PlanetName, PlanetLocation, PlanetId, IntoPlanetId}};

#[derive(Clone, Debug)]
pub struct StateCell {
    // TODO: use rust-smallvec https://crates.io/crates/smallvec
    deltas: Vec<(PlayerId, i64)>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub state: Vec<Vec<StateCell>>,
    pub current_state: Input,
    pub saved_expeditions: BitSet, 
    pub planet_map: BTreeMap<PlanetName, usize>,
    pub planet_names: Vec<PlanetName>,
    pub turn: i64,
    // maps planet_id to a list of planet_ids and distances, sorted by distance ascending
    pub nearest_planets: Vec<Vec<(f32, PlanetId)>>,
}


impl State {
    pub fn tick(&mut self) {
        self.turn += 1;
    }

    pub fn get_state_cell(&mut self, planet_name: &PlanetName, turns_ahead: i64) -> &mut StateCell {
        let planet_index = self.planet_map.get(planet_name).unwrap();
        // TODO: check if correct or if it is off by one 
        let turn_index = (self.turn + turns_ahead) as usize;
        &mut self.state[turn_index][*planet_index]
    }


    #[allow(dead_code)]
    pub fn predict_planets(&self, turns_ahead: i64) -> Vec<(Option<PlayerId>, i64)> {
        // let turn_index = self.turn + turns_ahead;
        let mut return_vec = Vec::new();
        for planet_name in &self.planet_names {
            return_vec.push(
                self.predict_planet(turns_ahead, planet_name)
            );
        }
        return_vec
        // &self.state[turn_index as usize]
    }

    // TODO: cache results
    pub fn predict_planet(&self, turns_ahead: i64, into_planet_id: impl IntoPlanetId) -> (Option<PlayerId>, i64) {
        let turn_index = (self.turn + turns_ahead) as usize;
        let planet_index = into_planet_id.into_planet_id(&self.planet_map);
        let planet_state = &self.current_state.planets[planet_index];

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
                map.insert(key, value.unwrap_or(&0) + amount);
            }

            let key_values = map.iter()
                .sorted_by_key(|(_, &v)| -v) // sorts by ascending order, so for negative value
                // sorts descending order
                .take(2)
                .collect_vec();

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

    #[allow(dead_code)]
    pub fn debug_print_predictions(&self) {
        let mut table = Table::new();
        let mut header = vec![Cell::new("turn")];
        for planet in &self.planet_names {
            header.push(Cell::new(&planet.clone()));
        }
        table.set_titles(Row::new(header));
        for i in 0..=12 {
            let mut predict_row = vec![Cell::new(&(self.turn + i).to_string())];
            for planet_name in &self.planet_names {
                let (owner, amount) = self.predict_planet(i, planet_name);
                let cell = Cell::new(
                    &format!("{owner:?}\n{amount:?}")
                );
                predict_row.push(cell);
            }
            table.add_row(Row::new(predict_row));
        }
        eprintln!("{:?}", self.current_state.expeditions);
        eprintln!("{table}");

    }

    #[allow(dead_code)]
    pub fn check_gameover(&self) -> GameSituation {
        // TODO: currently only checks units on planets and does not take expeditions into account 
        let binding = self.predict_planets(1);
        let ship_counts = binding
            .iter()
            .into_grouping_map_by(|(owner, _)| owner)
            .fold(0, |acc, _key, (_, val)| acc + val);

        // let temp = self.current_state.planets
        //     .iter()
        //     .into_grouping_map_by(|planet| planet.owner)
        //     .fold(0, |acc, _key, val| acc + val.ship_count);
        // eprintln!("current   : {:?}", self.current_state.expeditions);
        // eprintln!("current   : {:?}", temp);
        // eprintln!("prediction: {:?}", ship_counts);
        if !ship_counts.contains_key(&Some(1)) {
            return GameSituation::Lost;
        } 

        let other_key = ship_counts.keys()
            .into_iter()
            .find(|&&&key| key.is_some() && key != Some(1));
        if other_key.is_none() {
            return GameSituation::Won;
        }
        GameSituation::Ongoing 
    } 

    pub fn new(input: Input) -> Self {
        let mut entry = vec![];
        let mut planet_map = BTreeMap::new();
        let mut planet_names = vec![];
        let mut planet_locations: Vec<PlanetLocation> = vec![];
        let mut nearest_planets = Vec::new();

        for (index, planet) in input.planets.iter().enumerate() {
            entry.push(
                StateCell {
                    deltas: vec![],
                }
            );

            planet_map.insert(planet.name.clone(), index);
            planet_names.push(planet.name.clone());
            planet_locations.push(planet.into());
        }

        for (index, _planet) in input.planets.iter().enumerate() {
            let mut queue = PriorityQueue::new();
            let planet_location = &planet_locations[index];
            for (other_index, _other_planet) in input.planets.iter().enumerate() {
                if other_index == index {
                    continue;
                }
                let other_location = &planet_locations[other_index];
                let distance = planet_location.distance(other_location);
                queue.put(distance, other_index);
            }

            nearest_planets.push(queue.into_sorted_vec());
        }



        let mut state_vec = vec![];
        for _ in 0..MAX_TURNS {
            state_vec.push(entry.clone());
        }
        State {
            nearest_planets,
            planet_names,
            current_state: input,
            state: state_vec,
            planet_map,
            saved_expeditions: BitSet::new(),
            turn: 0,
        }
    }

    pub fn update(&mut self, mut input: Input) {
        for expedition in &input.expeditions {
            if self.saved_expeditions.contains(expedition.id as usize) {
                continue;
            } 
            self.saved_expeditions.insert(expedition.id as usize);
            let state_cell = self.get_state_cell(&expedition.destination, expedition.turns_remaining);
            state_cell.deltas.push(
                (expedition.owner, expedition.ship_count)
            );
        }
        input.planets.sort_by_key(|planet| self.planet_map[&planet.name]);
        self.current_state = input;
    }
}
