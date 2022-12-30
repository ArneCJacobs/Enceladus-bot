use itertools::Itertools;

use crate::{structs::Move, state::State};



pub struct NeighbourhoodAlrorithm {
    pub id: Option<u8>,
    pub neighbourhood_size: usize,
    pub friendly_load_balancing: usize,
    pub look_ahead: usize,
}

impl Default for NeighbourhoodAlrorithm {
    fn default() -> Self {
        NeighbourhoodAlrorithm { 
            id: Some(1), 
            neighbourhood_size: 7, 
            friendly_load_balancing: 2,
            look_ahead: 20,
        }
    }
}


impl NeighbourhoodAlrorithm {
    pub fn calculate(&mut self, state: &mut State) -> Vec<Move> {
        let mut moves = Vec::new();

        // TODO: for state, take into account currently planned moves
        for origin_planet_id in 0..state.planet_names.len() {
            let (origin_planet_owner, _origin_planet_fleet_size) = state.predict_planet(0, origin_planet_id);
            if origin_planet_owner != self.id {
                continue;
            }

            // TODO: check if all planets can be reached if only connecting the n nearest planets
            let mut nearest = state.nearest_planets[origin_planet_id]
                .iter()
                .take(self.neighbourhood_size)
                .map(|(distance, destination_planet_id)| {
                    let time_delta = distance.ceil() as i64; 
                    let (destination_owner, destination_fleet_size) = state.predict_planet(time_delta, *destination_planet_id);
                    (distance, destination_planet_id, destination_owner, destination_fleet_size)
                })
                .collect_vec();

            let mut enemies = nearest
                .drain_filter(|(_, _, owner, _)| *owner != self.id)
                .collect_vec();


            let origin_surplus = (0..self.look_ahead).map(|ta| {
                let (owner, owner_fleet_size) = state.predict_planet(ta as i64, origin_planet_id);
                if owner == Some(1) {
                    owner_fleet_size
                } else {
                    -owner_fleet_size
                }
            }).min().unwrap();
            if origin_surplus <= 0 {
                continue;
            }

            // TODO: calculate sendable origin fleet size based of future incoming expiditions 
            let mut sendable_origin_fleet_size = origin_surplus - 1;


            if !enemies.is_empty() && sendable_origin_fleet_size >= 0 {
                enemies.sort_by_key(|(distance, _destination_planet_id, _destination_owner, destination_fleet_size)| {
                    distance.ceil() as i64 + destination_fleet_size - sendable_origin_fleet_size
                });
                
                for (distance, destination_planet_id, _destination_owner, destination_fleet_size) in enemies {
                    let nessesary_fleet = destination_fleet_size + distance.ceil() as i64;
                    let expedition_size = i64::min(nessesary_fleet, sendable_origin_fleet_size);
                    sendable_origin_fleet_size -= expedition_size;

                    moves.push(Move{
                        origin: state.planet_names[origin_planet_id].clone(),
                        destination: state.planet_names[*destination_planet_id].clone(),
                        ship_count: expedition_size
                    });

                    if sendable_origin_fleet_size <= 0 {
                        break;
                    }
                }
            }
            if !nearest.is_empty() && sendable_origin_fleet_size >= 0 {
                let origin_risk_score = self.calcualte_risk(state, origin_planet_id);
                let mut nearest_scored = nearest.iter().map(|(_distance, destination_planet_id, _destination_owner, _destination_fleet_size)| {
                    // TODO: extract into function
                    let risk_score: f32 = self.calcualte_risk(state, **destination_planet_id);
                    (destination_planet_id, risk_score)
                })
                    .filter(|(_, risk_score)| *risk_score > origin_risk_score)
                    .collect_vec();

                if nearest_scored.is_empty() {
                    continue;
                }

                nearest_scored.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());
                // nearest_scored.sort_by_key(|(_, risk_score)| *risk_score.round() as i64);
                // high risk score => high priority => low priority value
                nearest_scored.reverse();


                let nearest_scored = &nearest_scored[0..usize::min(self.friendly_load_balancing, nearest_scored.len())];
                let sum_risk: f32 = nearest_scored.iter()
                    .map(|(_, score)| *score)
                    .sum();

                for (destination_planet_id, risk_score) in nearest_scored {
                    moves.push(Move { 
                        origin: state.planet_names[origin_planet_id].clone(), 
                        destination: state.planet_names[***destination_planet_id].clone(), 
                        ship_count: ((*risk_score / sum_risk) * sendable_origin_fleet_size as f32).floor() as i64
                    })
                }
            }
        }
        moves
    }

    fn calcualte_risk(&self, state: &State, destination_planet_id: usize) -> f32 {
        state.nearest_planets[destination_planet_id]
            .iter()
            .map(|(other_distance, other_planet_id)| {
                let (other_owner, other_fleet_size) = state.predict_planet(0, *other_planet_id);
                (other_distance, other_planet_id, other_owner, other_fleet_size)
            })
            .filter(|(_, _, owner, _)| *owner != self.id)
            .map(|(other_distance, _, other_owner, other_fleet_size)| {
                if other_owner.is_none() {
                    // nautral planets pose no risk
                    return 0.0;
                }
                other_fleet_size as f32 / other_distance
            })
            .sum()
    }
}
