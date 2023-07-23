use std::collections::{HashMap};
use petgraph::algo::dijkstra;
use petgraph::prelude::*;

const MOVE_COST: usize = 1;
const CHANGE_GEAR_COST: usize = 7;

#[derive(Copy, Clone)]
pub enum RegionType {
    Rocky,
    Wet,
    Narrow,
}

impl RegionType {
    pub fn get_tools(&self) -> [Tool; 2] {
        match self {
            RegionType::Rocky => [Tool::ClimbingGear, Tool::Torch],
            RegionType::Wet => [Tool::ClimbingGear, Tool::None],
            RegionType::Narrow => [Tool::Torch, Tool::None],
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Tool {
    Torch,
    ClimbingGear,
    None,
}

impl Tool {
    pub fn get_regions(&self) -> [RegionType; 2] {
        match self {
            Tool::ClimbingGear => [RegionType::Rocky, RegionType::Wet],
            Tool::Torch => [RegionType::Rocky, RegionType::Narrow],
            Tool::None => [RegionType::Wet, RegionType::Narrow],
        }
    }
}

pub struct Map {
    depth: usize,
    target: (usize, usize),
    regions: HashMap<(usize, usize), RegionType>,
    erosion_levels: HashMap<(usize, usize), usize>,
    distances: HashMap<(usize, usize, Tool), usize>,
}

impl Map {
    pub fn new(depth: usize, target: (usize, usize)) -> Self {
        Map {
            depth,
            target,
            regions: HashMap::new(),
            erosion_levels: HashMap::new(),
            distances: HashMap::new(),
        }
    }

    pub fn get_risk_level(&mut self, from: (usize, usize), to: (usize, usize)) -> usize {
        let mut risk_level = 0;
        for x in from.0..=to.0 {
            for y in from.1..=to.1 {
                let region_type = self.get_region_type((x, y));
                risk_level += match region_type {
                    RegionType::Rocky => 0,
                    RegionType::Wet => 1,
                    RegionType::Narrow => 2,
                }
            }
        }
        risk_level
    }

    pub fn find_distance_to_target(&mut self) -> usize {
        let graph = self.build_graph();

        let zero_index = graph.node_indices().find(|n| graph[*n] == (0, 0, Tool::Torch)).unwrap();
        let target_index = graph.node_indices().find(|n| graph[*n] == (self.target.0, self.target.1, Tool::Torch)).unwrap();
        let distances = dijkstra(&graph, zero_index, Some(target_index), |e| *e.weight());
        distances[&target_index]
    }

    fn build_graph(&mut self) -> UnGraph<(usize, usize, Tool), usize> {
        let mut graph = UnGraph::new_undirected();
        for x in 0..=(self.target.0 * 3) {
            for y in 0..=(self.target.1 * 3) {
                let region_type = self.get_region_type((x, y));
                let tools = region_type.get_tools();
                for tool in tools.iter() {
                    graph.add_node((x, y, *tool));
                }
            }
        }
        for node in graph.node_indices() {
            let coords = graph[node];
            let current_region_type = self.get_region_type((coords.0, coords.1));
            let current_tools = current_region_type.get_tools();
            let neighbours = self.get_neighbours((coords.0, coords.1));
            for neighbour in neighbours {
                let region_type = self.get_region_type(neighbour);
                let tools = region_type.get_tools();
                for tool in tools.iter() {
                    let neighbour = graph.node_indices().find(|n| graph[*n] == (neighbour.0, neighbour.1, *tool));
                    if neighbour.is_none() {
                        continue;
                    }

                    let neighbour = neighbour.unwrap();
                    if tool == &coords.2 {
                        graph.add_edge(node, neighbour, MOVE_COST);
                    } else if current_tools.contains(tool) {
                        graph.add_edge(node, neighbour, MOVE_COST + CHANGE_GEAR_COST);
                    }
                }
            }
        }

        graph
    }

    fn get_neighbours(&mut self, coords: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbours = Vec::new();
        neighbours.push((coords.0 + 1, coords.1));
        neighbours.push((coords.0, coords.1 + 1));
        neighbours
    }

    fn get_region_type(&mut self, coords: (usize, usize)) -> RegionType {
        let region_type = self.regions.get(&coords);
        if let Some(region_type) = region_type {
            return *region_type;
        }

        let region_type = match self.get_erosion_level(coords) % 3 {
            0 => RegionType::Rocky,
            1 => RegionType::Wet,
            2 => RegionType::Narrow,
            _ => unreachable!(),
        };
        self.regions.insert(coords, region_type);
        region_type
    }

    fn get_erosion_level(&mut self, coords: (usize, usize)) -> usize {
        let erosion_level = self.erosion_levels.get(&coords);
        if let Some(erosion_level) = erosion_level {
            return *erosion_level;
        }

        let geologic_index = self.get_geologic_index(coords);
        let erosion_level = (geologic_index + self.depth) % 20183;
        self.erosion_levels.insert(coords, erosion_level);
        erosion_level
    }

    #[inline]
    fn get_geologic_index(&mut self, coords: (usize, usize)) -> usize {
        if coords == self.target {
            return 0;
        }

        match coords {
            (0, 0) => 0,
            (0, y) => y * 48271,
            (x, 0) => x * 16807,
            coords if coords == self.target => 0,
            (x, y) => self.get_erosion_level((x - 1, y)) * self.get_erosion_level((x, y - 1)),
        }
    }
}