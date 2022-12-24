use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub enum GameSituation {
    WON,
    LOST,
    ONGOING,
}

pub type PlanetName = String;
pub type ExpeditionId = u64;
pub type PlayerId = u8;
pub type PlanetId = usize;

#[derive(Deserialize, Debug, Clone)]
pub struct Input {
    pub planets: Vec<Planet>,
    pub expeditions: Vec<Expedition>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Planet {
    pub ship_count: i64,
    pub x: f32,
    pub y: f32,
    pub owner: Option<PlayerId>, 
    pub name: PlanetName
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlanetLocation {
    pub x: f32,
    pub y: f32,
}

impl PlanetLocation {
    pub fn distance(&self, other: &PlanetLocation) -> f32 {
        (
            (self.x - other.x).powi(2) +
            (self.y - other.y).powi(2)
        ).sqrt()
    }
}

impl From<&Planet> for PlanetLocation {
    fn from(planet: &Planet) -> Self {
        let Planet{ x, y, ..} = *planet;
        PlanetLocation { x, y } 
    }
}


#[derive(Deserialize, Debug, Clone)]
pub struct Expedition {
    pub id: ExpeditionId,
    pub ship_count: i64,
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub owner: PlayerId,
    pub turns_remaining: i64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub  moves: Vec<Move>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    pub origin: PlanetName,
    pub destination: PlanetName,
    pub ship_count: i64,
}

pub trait IntoPlanetId {
    fn into_planet_id(&self, planet_map: &BTreeMap<PlanetName, PlanetId>) -> PlanetId;
}

impl IntoPlanetId for PlanetId {
    fn into_planet_id(&self, planet_map: &BTreeMap<PlanetName, PlanetId>) -> PlanetId {
        *self
    }
}

impl IntoPlanetId for &PlanetName {
    fn into_planet_id(&self, planet_map: &BTreeMap<PlanetName, PlanetId>) -> PlanetId {
        planet_map[*self]
    }
}
