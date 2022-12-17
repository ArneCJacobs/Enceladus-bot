use priq::PriorityQueue;
use smallvec::SmallVec;
use vec_map::VecMap;

use crate::{state::State, structs::{Output, Move}};

// TODO: make field of SimpleAlrorithm 
const LOOK_AHEAD: usize = 5;

pub struct SimpleAlrorithm {
    // first index is planet, second is time
    // scores: VecMap<VecMap<f32>>

}


impl SimpleAlrorithm {
    pub fn calculate(&mut self, state: &mut State) -> Vec<Move> {
        // self.scores.clear();
        let mut moves = Vec::new();


        let queue = self.calculate_scores(state).into_sorted_vec();
        for (_, (planet_id, turns_ahead)) in queue.iter() {
            let nearest: SmallVec<[&(f32, usize); 3]> = state.nearest_planets[*planet_id]
                .iter()
                .filter(|planet| state.predict_planet(0, *planet_id).0 == Some(1))
                .take(3)
                .collect();

            let ship_count = state.predict_planet(0, *planet_id).1; 
            let (distance, nearest_planet_id) = nearest[0];
            let ship_count_nearest = state.predict_planet(0, *nearest_planet_id).1;
            let fleet_size = ship_count_nearest - 1;
            if ship_count + (distance.ceil() as i64) < fleet_size {
                moves.push(
                    crate::structs::Move { 
                        origin: state.planet_names[*nearest_planet_id].clone(), 
                        destination: state.planet_names[*planet_id].clone(), 
                        ship_count:  fleet_size
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
                let score: f32 = self.calculate_score(planet_id, turns_ahead, state);
                queue.put(score, (planet_id, turns_ahead));
            }
        }

        queue 
    }

    fn calculate_score(&self, planet_id: usize, turns_ahead: usize, state: &mut State) -> f32 {
        todo!()
    }

}
