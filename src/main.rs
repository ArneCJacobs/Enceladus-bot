#![feature(drain_filter)]

use std::{io::{self, BufRead, Write}, collections::BTreeMap};

use bit_set::BitSet;
use neighbourhood::NeighbourhoodAlrorithm;
use state::State;
use structs::Input;

use crate::structs::Output;


mod state;
mod structs;
mod simple_algoritm;
mod direct_flow;
mod neighbourhood;

const MAX_TURNS: u64 = 500;
#[allow(dead_code)]
const HARD_MAX_DURATION: u64 = 1000;
#[allow(dead_code)]
const MAX_DURATION: u64 = 800;

fn main() {

    let stdin = io::stdin();
    let mut state = State {
        nearest_planets: vec![],
        current_state: Input { planets: vec![], expeditions: vec![] },
        planet_names: vec![],
        state: vec![],
        saved_expeditions: BitSet::new(),
        planet_map: BTreeMap::new(),
        turn: 0,
    };
    let mut algorithm = NeighbourhoodAlrorithm::default();
    // let mut algorithm = SimpleAlrorithm {};


    for line in stdin.lock().lines() {
        // let now = Instant::now();
        let line = line.unwrap();
        eprintln!("=========================================================");
        // eprintln!("{}", line);
        let input: Input = serde_json::from_str(&line).unwrap();
        if state.turn == 0 {
            state = State::new(input);
        } else {
            state.update(input);
        }

        // match state.check_gameover() {
        //     GameSituation::WON => eprintln!("WE HAVE WON!"),
        //     GameSituation::LOST => eprintln!("WE HAVE LOST!"),
        //     _ => {}
        // }


        let output = Output {
            moves: algorithm.calculate(&mut state)
        };

        // while now.elapsed() < Duration::from_millis(MAX_DURATION) {
        //     //TODO: do things
        //
        // }
        println!("{}\n", serde_json::to_string(&output).unwrap());
        std::io::stdout().flush().unwrap();
        state.tick();
    }
}
