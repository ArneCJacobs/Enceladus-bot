use priq::PriorityQueue;
use smallvec::SmallVec;
use vec_map::VecMap;

use crate::{state::State, structs::{Output, Move}};

// TODO: make field of SimpleAlrorithm 
const LOOK_AHEAD: usize = 20;

pub struct SimpleAlrorithm {
    // first index is planet, second is time
    // scores: VecMap<VecMap<f32>>

}


impl SimpleAlrorithm {
    pub fn calculate(&mut self, state: &mut State) -> Vec<Move> {
        // self.scores.clear();
        let mut moves = Vec::new();


        let mut queue = self.calculate_scores(state).into_sorted_vec();
        queue.reverse();
        // dbg!(&state);
        // eprintln!("QUEUE {queue:?}");
        for (score, (planet_id, turns_ahead)) in queue.iter() {
            if *score <= 0.0 {
                break;
            }
            // eprintln!("PLANET {planet_id:?}, TURNS_AHEAD: {turns_ahead:?}");
            
            let nearest: SmallVec<[&(f32, usize); 3]> = state.nearest_planets[*planet_id]
                .iter()
                .filter(|(_distance, other_planet_id)| state.predict_planet(0, *other_planet_id).0 == Some(1))
                .take(3)
                .collect();

            let destination_fleet_size = state.predict_planet(0, *planet_id).1; 
            if nearest.is_empty() { 
                break;
            }
            let (distance, origin_planet_id) = nearest[0];
            // let origin_fleet_size = state.predict_planet(0, *origin_planet_id).1;

            let origin_deficit = (0..LOOK_AHEAD).map(|ta| {
                let (owner, owner_fleet_size) = state.predict_planet(ta as i64, *origin_planet_id);
                if owner == Some(1) {
                    owner_fleet_size
                } else {
                    -owner_fleet_size
                }
            }).min().unwrap();
            if origin_deficit <= 0 {
                continue;
            }

            let deployable_origin_fleet_size = origin_deficit - 1;
            if destination_fleet_size + (distance.ceil() as i64) < deployable_origin_fleet_size {
                moves.push(
                    crate::structs::Move { 
                        origin: state.planet_names[*origin_planet_id].clone(), 
                        destination: state.planet_names[*planet_id].clone(), 
                        ship_count:  deployable_origin_fleet_size
                    }
                )
            }
        }
        moves
    }

    // fn set_score(&mut self, planet_id: usize, turns_ahead: usize, score: f32) {
    //     let planet_scores = self.scores.entry(planet_id)
    //         .or_insert_with(VecMap::new);
    //
    //     planet_scores.insert(turns_ahead, score);
    // } 

    // first element of tuple is planet_id, second it turns ahead
    fn calculate_scores(&mut self, state: &mut State) -> PriorityQueue<f32, (usize, usize)> {
        let mut queue = PriorityQueue::new();
        
        for planet_id in 0..state.planet_names.len() {
            for turns_ahead in 0..LOOK_AHEAD {
                let score: f32 = self.calculate_score(planet_id, turns_ahead as i64, state);
                queue.put(score, (planet_id, turns_ahead));
            }
        }

        queue 
    }

    fn calculate_score(&self, planet_id: usize, turns_ahead: i64, state: &mut State) -> f32 {
        let (owner, fleet_size) = state.predict_planet(turns_ahead, planet_id);
        if owner == Some(1) {
            return -3.0;
        }
        let nearest: SmallVec<[_; 3]> = state.nearest_planets[planet_id]
            .iter()
            .map(|(distance, other_planet_id)| {
                let (owner, fleet_size) = state.predict_planet(turns_ahead, *other_planet_id);
               (distance, other_planet_id, owner, fleet_size)
            }) 
            .filter(|(_, _, owner, _)| *owner == Some(1)) // TODO: change Some(1) to variable stored in self
            .take(1)
            .collect();
        if nearest.is_empty() {
            return 0.0;
        }
        let (distance, _, _, other_fleet_size) = nearest[0];
        let score = (other_fleet_size - fleet_size) as f32 - distance.ceil();
        return score;
    }

}
