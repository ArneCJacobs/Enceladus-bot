use std::{collections::{BTreeSet, BTreeMap}, fs::File, process::exit};
use std::io::Write;

use mcmf::{GraphBuilder, Vertex, Capacity, Cost};
use smallvec::SmallVec;

use crate::{structs::{Move, PlanetId}, state::State};

const LOOK_AHEAD: usize = 1;
const IDLE_PENALTY_COST: i32 = 1000;

pub struct Flow1Algorithm {
    pub id: Option<u8>,
}

impl Flow1Algorithm {
    pub fn calculate(&mut self, state: &mut State) -> Vec<Move> {
        // let mut cost_edges: BTreeSet<_> = BTreeSet::new();
        let mut graph_builder = GraphBuilder::new();

        for origin_planet_id in 0..state.planet_names.len() {
            for turns_ahead in 0..LOOK_AHEAD+1 {
                let origin_planet_id = origin_planet_id as i32;
                let turns_ahead = turns_ahead as i32;
                let origin_planet_node_in = (origin_planet_id, turns_ahead, 0);
                let origin_planet_node_out = (origin_planet_id, turns_ahead, 1);
                // let time_index = state.turn + turns_ahead;
                let (owner, fleet_size) = state.predict_planet(turns_ahead as i64, origin_planet_id as PlanetId);
                if owner == self.id {
                    graph_builder.add_edge(origin_planet_node_in, origin_planet_node_out, Capacity(i32::MAX), Cost(0)); // TODO: cost based on score/priority

                    if turns_ahead == 0 {
                        // starting fleet size
                        graph_builder.add_edge(Vertex::Source, origin_planet_node_in, Capacity(fleet_size as i32), Cost(0)); 
                    } else {
                        let (last_owner, _last_fleet_size) = state.predict_planet(turns_ahead as i64 - 1, origin_planet_id as PlanetId);
                        if last_owner == owner {
                            // growth on owned planet
                            graph_builder.add_edge(Vertex::Source, origin_planet_node_in, Capacity(1), Cost(0));
                        } else {
                            // allied expedition arrives
                            graph_builder.add_edge(Vertex::Source, origin_planet_node_in, Capacity(fleet_size as i32), Cost(0));
                            // TODO: edge (with negative cost, to that it is always taken) straight to Sink if an enemy expedition arrives
                        }
                        // if fleet isn't moved
                        graph_builder.add_edge((origin_planet_id, turns_ahead-1, 1), origin_planet_node_in, Capacity(i32::MAX), Cost(0)); //TODO: play with stagnancy cost
                    }

                    if turns_ahead == LOOK_AHEAD as i32 {
                        // last nodes need an outflow
                        graph_builder.add_edge(origin_planet_node_out, Vertex::Sink, Capacity(i32::MAX), Cost(IDLE_PENALTY_COST));
                    }
                } else {
                    graph_builder.add_edge(origin_planet_node_in, origin_planet_node_out, Capacity(i32::MAX), Cost(-400)); // TODO: negative cost based on score/priority
                    if turns_ahead == LOOK_AHEAD as i32 {
                        // last nodes need an outflow
                        graph_builder.add_edge(
                            origin_planet_node_out, 
                            Vertex::Sink, 
                            Capacity(i32::MAX), 
                            Cost(0)
                        );
                    }
                }

                // ================================ outgoing connections ==========================
                for (distance, destination_planet_id) in &state.nearest_planets[origin_planet_id as usize] {
                    // TODO: filter planets where turns_ahead + time_delta is past max turns
                    let time_delta = distance.ceil() as i32;
                    let new_turns_ahead = turns_ahead + time_delta;
                    graph_builder.add_edge(
                        origin_planet_node_out,
                        (*destination_planet_id as i32, new_turns_ahead, 0), 
                        Capacity(i32::MAX), 
                        Cost(time_delta)
                    );

                    if new_turns_ahead > LOOK_AHEAD as i32 { 

                        graph_builder.add_edge(
                            (*destination_planet_id as i32, new_turns_ahead, 0), 
                            (*destination_planet_id as i32, new_turns_ahead, 1), 
                            Capacity(i32::MAX), 
                            Cost(0) // TODO: negative cost based on score/priority
                        );

                        graph_builder.add_edge(
                            (*destination_planet_id as i32, new_turns_ahead, 1), 
                            Vertex::Sink,
                            Capacity(i32::MAX), 
                            Cost(0) // TODO: negative cost based on score/priority
                        );
                         
                    }
                }
            }
        }

        let mut id = 0;
        let mut id_map = BTreeMap::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        // eprintln!("BEGIN DEBUG PRINT GRAPH");
        for (begin, end, cap, cost) in &graph_builder.edge_list {
            // eprintln!("{vertex:?}"); 
            let start_node_id = id_map.entry(begin).or_insert({
                id += 1;
                // format!("id{id}")
                id
            }).clone();

            let end_node_id = id_map.entry(end).or_insert({
                id += 1;
                id
                // format!("id{id}")
            }).clone();
                
            // nodes.push(format!("{{\"from\": {start_node_id}, \"to\": {end_node_id}, \"label\":\"cap: {}, cost: {}\" }}", cap.0, cost.0));
            if cap.0 == i32::MAX {
                edges.push(format!("{{\"from\": {start_node_id}, \"to\": {end_node_id}, \"label\":\"cap: {}, cost: {}\" }}", "MAX", cost.0));
                // graph_entries.push(format!("{start_node_id}-- \"cap: {}, cost: {}\" -->{end_node_id}", "MAX", cost.0));
            } else {
                edges.push(format!("{{\"from\": {start_node_id}, \"to\": {end_node_id}, \"label\":\"cap: {}, cost: {}\" }}", cap.0, cost.0));
                // graph_entries.push(format!("{start_node_id}-- \"cap: {}, cost: {}\" -->{end_node_id}", cap.0, cost.0));
            }
        }
        for (key, value) in id_map.iter() {
            nodes.push(format!("{{ \"id\":{value}, \"label\": \"{key:?}\" }}"));
            // graph_entries.push(format!("{value}(\"{key:?}\")"));
        }
        let mut node_file = File::create("./src/nodes.json").unwrap();
        write!(&mut node_file, "[\n\t{}\n]", nodes.join(",\n")).unwrap();
        let mut edge_file = File::create("./src/edges.json").unwrap();
        write!(&mut edge_file, "[\n\t{}\n]", edges.join(",\n")).unwrap();
        exit(0);
        // for entry in edges /* { */
        // }
        // eprintln!("END DEBUG PRINT GRAPH");

        let (_cost, paths) = graph_builder.mcmf();
        eprintln!("COST: {_cost}");
        // TODO: transform paths into moves 
        let moves: Vec<_> = paths.iter()
            // .map(|path| {
            //     eprintln!("PATH START == COST: {}, FLOW: {}", path.cost(), path.amount());
            //     for vertex in path.vertices() {
            //         eprintln!("\t{vertex:?}");
            //     }
            //     eprintln!("PATH END");
            //     path
            // })
            .flat_map(|path| path.edges()) //TODO: take the first 3 or 4 or so
            .filter(|edge| {
                match (edge.a, edge.b) {
                    (Vertex::Node((origin_planet_id, 0, 1)), Vertex::Node((destination_planet_id, _, _))) if origin_planet_id != destination_planet_id => true,
                    _ => false
                }
            })
            .map(|edge| {
                let Vertex::Node((origin_planet_id, _, _)) = edge.a else {
                    panic!("Move start node has unexpected value");
                }; 
                let Vertex::Node((destination_planet_id, _, _)) = edge.b else {
                    panic!("Move end node has unexpected value");
                };
                Move {
                    origin: state.planet_names[origin_planet_id as usize].clone(),
                    destination: state.planet_names[destination_planet_id as usize].clone(),
                    ship_count: edge.amount.try_into().unwrap()
                }
            }).collect();

        eprintln!("MOVE COUNT: {}", moves.len());
        moves
    }
}
