use crate::{
    engine::matrix::Matrix,
    iotwins_model::{
        agent::Agent,
        routes::{find_route, Route},
        structures::{generate_structures, load_mouths, Structure},
    },
};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Floor {
    pub ground_truth: Matrix<u8>,
    pub structures: HashMap<u8, HashSet<Structure>>, // Mapping Position -> matrix by type of structure
    pub structures_paths: HashSet<Route>,
    pub mouths: HashMap<u16, Structure>, // Agent destinations
    pub mouths_paths: HashMap<u16, HashSet<Route>>, // From down-stairs -> mouths (grandstands)
    pub agents: Vec<Agent>,              // All agents in floor
    pub agents_paths: HashMap<usize, Vec<usize>>, // Path to follow by every agent in layer (Only own agent modifies this, analytics)
}

impl Floor {
    pub fn create_floor(path: String, name: String) -> Floor {
        let ground_truth = Floor::ground_truth(&Matrix::load_layer(&path));
        let structures = generate_structures(&ground_truth);
        let mouths = load_mouths(&name);

        Floor {
            structures_paths: Floor::stairs_paths(&ground_truth, &structures, &name),
            mouths_paths: Floor::mouth_paths(&ground_truth, &structures, &mouths, &name),
            mouths,
            structures,
            ground_truth,
            ..Default::default()
        }
    }

    pub fn insert_agents(&mut self, agents: Vec<Agent>, current_locaton: Structure) {
        if agents.is_empty() {
            return;
        }

        // Get one agent to find needed data
        let origin = agents[0].to_owned();

        // Destination of current route (target)
        let agents_target = origin.target;

        // Current target is an stair
        if self.structures.get(&11).expect("").contains(&agents_target) {
            // Get route current -> target and insert agents if exists
            if let Some(route) = self.structures_paths.get(&Route {
                origin: current_locaton,
                destination: agents_target,
                ..Default::default()
            }) {
                // Assign route to each agent & insert them into layer
                let agents_routes = agents.iter().map(|agent| (agent.id, route.get_path()));

                self.agents_paths.extend(agents_routes);
                self.agents.extend(agents);
            }
        } else if let Some(mouth) = self.mouths.get(&origin.destination) {
            // Agent towards final destination (mouth) from stairs
            if let Some(routes) = self.mouths_paths.get(&origin.destination) {
                if let Some(route) = routes.get(&Route {
                    origin: current_locaton,
                    destination: mouth.to_owned(),
                    ..Default::default()
                }) {
                    // Assign route to each agent & insert them into layer
                    let agents_routes = agents.iter().map(|agent| (agent.id, route.get_path()));

                    self.agents_paths.extend(agents_routes);
                    self.agents.extend(agents);
                }
            }
        }
    }

    pub fn load_floor(
        path: &str,
        name: &str,
        structures: HashMap<u8, HashSet<Structure>>,
        structures_paths: HashSet<Route>,
        mouths_paths: HashMap<u16, HashSet<Route>>,
    ) -> Floor {
        Floor {
            structures,
            ground_truth: Floor::ground_truth(&Matrix::load_layer(path)),
            structures_paths,
            mouths: load_mouths(name),
            mouths_paths,
            ..Default::default()
        }
    }

    fn stairs_paths(
        gt: &Matrix<u8>,
        structures: &HashMap<u8, HashSet<Structure>>,
        layer: &str,
    ) -> HashSet<Route> {
        let down = structures.get(&10).expect("");
        let up = structures.get(&11).expect("");

        // Progress bar
        let progress_bar = ProgressBar::new(down.len().try_into().unwrap());

        progress_bar.set_message(format!("{layer} - Map jumps"));

        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{msg} {bar:10} {pos}/{len}")
                .progress_chars("#>#-"),
        );
        // End of progress bar

        let stairs_paths = down
            .par_iter()
            .progress_with(progress_bar)
            .flat_map(|p1| up.par_iter().filter_map(|p2| find_route(gt, p1, p2)));

        HashSet::from_par_iter(stairs_paths)
    }

    // For each destination mouth, a set of possible routes from up-stairs
    fn mouth_paths(
        gt: &Matrix<u8>,
        structures: &HashMap<u8, HashSet<Structure>>,
        mouths: &HashMap<u16, Structure>,
        layer: &str,
    ) -> HashMap<u16, HashSet<Route>> {
        // Down stairs are the only positions from where you can go to the grandstands, agents will arrive at down-stairs
        let down = structures.get(&10).expect("");

        // Progress bar
        let progress_bar = ProgressBar::new(down.len().try_into().unwrap());

        progress_bar.set_message(format!("{layer} - Mouths paths"));

        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{msg} {bar:10} {pos}/{len}")
                .progress_chars("#>#-"),
        );
        // End of progress bar

        let mouths_routes = mouths
            .par_iter()
            .progress_with(progress_bar)
            .map(|(id, mouth)| {
                let mouth_routes = down
                    .par_iter()
                    .filter_map(|stair| find_route(gt, stair, mouth));

                (*id, HashSet::from_par_iter(mouth_routes))
            });

        // mouths_paths
        HashMap::from_par_iter(mouths_routes)
    }

    pub fn ground_truth(blueprint: &Matrix<u8>) -> Matrix<u8> {
        Matrix::ground_thruth(
            blueprint,
            HashMap::from([
                // Walls
                (84, 1),  // Obstacles
                (202, 1), // No transit
                (159, 1), // Kitchen
                (7, 1),   // Technical staff
                (241, 1), // Lockers
                (103, 1), // SAT
                (131, 1), // Men restroom
                (118, 1), // Women restroom
                (164, 1), // Restroom
                (122, 1), // Bar
                // Map jumps
                (109, 10), // Down stair
                (232, 11), // Up stair
                (112, 20), // Down ramp
                (189, 21), // Up ramp
                (182, 3),  // Elevator
            ]),
        )
    }
}
