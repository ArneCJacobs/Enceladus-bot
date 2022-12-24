use std::collections::BTreeSet;

use itertools::Itertools;
use mcmf::{GraphBuilder, Vertex, Capacity, Cost};
use smallvec::SmallVec;

use crate::{structs::Move, state::State};

const LOOK_AHEAD: usize = 20;
const IDLE_PENALTY_COST: i32 = 1000;

pub struct Flow1Algorithm {
    id: Option<u8>,
}

impl Flow1Algorithm {
    pub fn calculate(&mut self, state: &mut State) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut cost_edges: BTreeSet<_> = BTreeSet::new();
        let mut graph_builder = GraphBuilder::new();

        for origin_planet_id in 0..state.planet_names.len() {
            for turns_ahead in 0..LOOK_AHEAD {
                let origin_planet_id = origin_planet_id as i32;
                let turns_ahead = turns_ahead as i32;
                let origin_planet_node = (origin_planet_id, turns_ahead, 0);
                // let time_index = state.turn + turns_ahead;
                let (owner, fleet_size) = state.predict_planet(turns_ahead as i64, origin_planet_id as usize);
                if owner == self.id {
                    let flow_restriction_node = (origin_planet_id, -1, 0);
                    if turns_ahead == 0 {
                        graph_builder.add_edge(Vertex::Source, flow_restriction_node, Capacity(fleet_size as i32), Cost(0)); 
                        graph_builder.add_edge(flow_restriction_node, origin_planet_node, Capacity(i32::MAX), Cost(0));
                    } else {
                        graph_builder.add_edge(Vertex::Source, origin_planet_node, Capacity(turns_ahead), Cost(0));
                        graph_builder.add_edge(flow_restriction_node, origin_planet_node, Capacity(i32::MAX), Cost(0));
                        if turns_ahead == LOOK_AHEAD as i32 {
                            graph_builder.add_edge(origin_planet_node, Vertex::Sink, Capacity(i32::MAX), Cost(IDLE_PENALTY_COST));
                        }
                    }

                    let nearest: SmallVec<[_; 5]> = state.nearest_planets[origin_planet_id as usize]
                        .iter()
                        .map(|(distance, other_planet_id)| {
                            let time_delta = distance.ceil() as i64;
                            let (other_owner, other_fleet_size) = state.predict_planet(turns_ahead as i64 + time_delta, *other_planet_id);
                            (distance, other_planet_id, other_owner, other_fleet_size)
                        })
                        .filter(|(_, _, other_owner, _)| *other_owner != self.id)
                        .collect();

                    for (distance, destination_planet_id, _destination_owner, destination_fleet_size) in nearest {
                        let time_delta = distance.ceil() as i32;
                        graph_builder.add_edge(
                            origin_planet_node,
                            (*destination_planet_id as i32, turns_ahead + time_delta, 1), 
                            Capacity(i32::MAX), 
                            Cost(time_delta)
                        );

                        cost_edges.insert((*destination_planet_id as i32, turns_ahead + time_delta, destination_fleet_size));
                    }

                } else {
                    todo!();
                }
            }
        }

        for (destination_planet_id, turns_ahead, destination_fleet_size) in cost_edges {
            graph_builder.add_edge(
                (destination_planet_id, turns_ahead, 1), 
                Vertex::Sink, 
                Capacity(i32::MAX), 
                Cost(destination_fleet_size.try_into().unwrap())
            );
        }

        let (_cost, paths) = graph_builder.mcmf();
        // TODO: transform paths into moves 

        return moves;
    }
}
