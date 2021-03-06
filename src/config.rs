pub mod configuration {

    use crate::iotwins_model::config as model;
    use serde::Deserialize;
    use std::{fs::File, io::Write};

    #[derive(Deserialize)]
    struct Output {
        results_dir: String,
        results_file: String,
        logs_file: String,
    }

    #[derive(Debug, Deserialize)]
    struct Logs {
        print_in_console: bool,
        print_instrumentation: bool,
    }

    #[derive(Debug, Deserialize)]
    struct Size {
        height: usize,
        width: usize,
    }

    #[derive(Debug, Deserialize)]
    struct Simulation {
        num_agents: u64,
        num_counters: u32,
    }

    #[derive(Deserialize)]
    pub struct Parameters {
        // General engine configuration
        output: Output,
        logs: Logs,
        size: Size,
        input_data: Simulation,

        // Model-specific configuration
        agent_data: model::AgentStats,
        coefficients: model::Coeffs,
        pub topology: model::Topology,
        pub venue_tags: model::Venue,
        match_timings: model::Match,
    }

    impl Parameters {
        // Returns configuration
        pub fn load_configuration(path: String) -> Parameters {
            // Open config file
            let data = match std::fs::read_to_string(path) {
                Ok(file) => file,
                Err(error) => panic!("Problem opening the file: {:?}", error),
            };

            // Deserialize config file into config struct
            match toml::from_str(&data) {
                Ok(file) => file,
                Err(error) => panic!("Problem opening the file: {:?}", error),
            }
        }

        // Grid size for computation
        pub fn get_world_size(&self) -> (usize, usize) {
            (self.size.height, self.size.width)
        }

        // Consolidate results
        pub fn write_results(&self, data: String) {
            let mut file =
                File::create(&self.output.results_file).expect("[ERROR] Unable to create file");
            file.write_all(data.as_bytes())
                .expect("[Error] Unable to write data");
        }

        // Display logs info on terminal and write down to log historial
        pub fn logs(&self, info: String) {
            if self.logs.print_in_console {
                println!("{info}");
            }

            let mut file =
                File::create(&self.output.logs_file).expect("[ERROR] Unable to create file");
            file.write_all(info.as_bytes())
                .expect("[Error] Unable to write data");
        }

        // Total agents to be simulated
        pub fn total_agents(&self) -> u64 {
            self.input_data.num_agents
        }
    }
}
