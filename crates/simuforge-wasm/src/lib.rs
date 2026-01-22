//! SimuForge WASM - WebAssembly bindings for browser-based physics simulation

use wasm_bindgen::prelude::*;
use simuforge_core::{ExperimentSpec, MetricFrame, SimulationReport, Transform};
use simuforge_physics::{MetricWorld, create_scenario};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Physics simulation instance for browser use
#[wasm_bindgen]
pub struct Simulation {
    world: MetricWorld,
    spec: ExperimentSpec,
    target_steps: u64,
}

#[wasm_bindgen]
impl Simulation {
    /// Create a new simulation from JSON spec
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Simulation, JsError> {
        let spec: ExperimentSpec = serde_json::from_str(spec_json)
            .map_err(|e| JsError::new(&format!("Failed to parse spec: {}", e)))?;

        spec.validate()
            .map_err(|errors| JsError::new(&format!("Invalid spec: {}", errors.join(", "))))?;

        let mut world = MetricWorld::from_spec(&spec);

        let scenario = create_scenario(&spec.spec.scenario);
        scenario.setup(&mut world);

        let target_steps = match &spec.spec.duration {
            simuforge_core::spec::DurationConfig::Fixed { steps } => *steps,
            simuforge_core::spec::DurationConfig::Time { seconds } => {
                (*seconds / spec.spec.physics.timestep) as u64
            }
            simuforge_core::spec::DurationConfig::UntilStable { max_steps, .. } => *max_steps,
        };

        Ok(Simulation {
            world,
            spec,
            target_steps,
        })
    }

    /// Step the simulation forward by one frame
    pub fn step(&mut self) -> JsValue {
        self.world.step();
        let frame = self.world.current_frame();
        serde_wasm_bindgen::to_value(&frame).unwrap_or(JsValue::NULL)
    }

    /// Get current metric frame
    pub fn get_frame(&self) -> JsValue {
        let frame = self.world.current_frame();
        serde_wasm_bindgen::to_value(&frame).unwrap_or(JsValue::NULL)
    }

    /// Get transforms of all bodies for rendering
    pub fn get_body_transforms(&self) -> JsValue {
        // Use current_frame which reads directly from physics world
        let frame = self.world.current_frame();
        let transforms: Vec<BodyTransform> = frame.bodies.iter().map(|b| BodyTransform {
            id: b.id,
            name: b.name.clone(),
            position: [b.transform.position.x, b.transform.position.y, b.transform.position.z],
            rotation: [
                b.transform.rotation.x,
                b.transform.rotation.y,
                b.transform.rotation.z,
                b.transform.rotation.w,
            ],
        }).collect();

        serde_wasm_bindgen::to_value(&transforms).unwrap_or(JsValue::NULL)
    }

    /// Run simulation to completion and return report
    pub fn run_to_completion(&mut self) -> JsValue {
        let remaining = self.target_steps.saturating_sub(self.world.step_count());
        self.world.run(remaining);

        let frames = self.world.frames();
        let mut report = SimulationReport::new(self.spec.metadata.name.clone());
        report.finalize(frames, &self.spec.spec.criteria);

        serde_wasm_bindgen::to_value(&report).unwrap_or(JsValue::NULL)
    }

    /// Get current step number
    pub fn current_step(&self) -> u64 {
        self.world.step_count()
    }

    /// Get current simulation time
    pub fn current_time(&self) -> f32 {
        self.world.time()
    }

    /// Get target step count
    pub fn target_steps(&self) -> u64 {
        self.target_steps
    }

    /// Check if simulation is complete
    pub fn is_complete(&self) -> bool {
        self.world.step_count() >= self.target_steps
    }

    /// Get number of bodies
    pub fn body_count(&self) -> usize {
        self.world.body_count()
    }

    /// Reset simulation to initial state
    pub fn reset(&mut self) {
        self.world = MetricWorld::from_spec(&self.spec);
        let scenario = create_scenario(&self.spec.spec.scenario);
        scenario.setup(&mut self.world);
    }
}

/// Body transform for rendering
#[derive(serde::Serialize)]
struct BodyTransform {
    id: u64,
    name: String,
    position: [f32; 3],
    rotation: [f32; 4],
}

/// Create a simulation from YAML spec string
#[wasm_bindgen]
pub fn create_simulation_from_yaml(yaml: &str) -> Result<Simulation, JsError> {
    let spec: ExperimentSpec = serde_yaml::from_str(yaml)
        .map_err(|e| JsError::new(&format!("Failed to parse YAML: {}", e)))?;

    let json = serde_json::to_string(&spec)
        .map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))?;

    Simulation::new(&json)
}

/// Validate experiment spec without creating simulation
#[wasm_bindgen]
pub fn validate_spec(spec_json: &str) -> JsValue {
    let result: Result<ExperimentSpec, _> = serde_json::from_str(spec_json);

    match result {
        Ok(spec) => {
            match spec.validate() {
                Ok(()) => JsValue::from_str("valid"),
                Err(errors) => {
                    let msg = format!("Validation errors: {}", errors.join(", "));
                    JsValue::from_str(&msg)
                }
            }
        }
        Err(e) => JsValue::from_str(&format!("Parse error: {}", e)),
    }
}

/// Get list of available built-in scenarios
#[wasm_bindgen]
pub fn get_available_scenarios() -> JsValue {
    let scenarios = vec![
        ScenarioInfo {
            name: "box_stack".to_string(),
            description: "Stack of boxes on ground plane".to_string(),
            params: vec!["count", "box_size", "spacing", "friction"],
        },
        ScenarioInfo {
            name: "rolling_sphere".to_string(),
            description: "Sphere rolling on flat surface".to_string(),
            params: vec!["radius", "initial_velocity", "friction"],
        },
        ScenarioInfo {
            name: "bouncing_ball".to_string(),
            description: "Ball dropped from height".to_string(),
            params: vec!["radius", "drop_height", "restitution"],
        },
        ScenarioInfo {
            name: "friction_ramp".to_string(),
            description: "Object sliding down inclined ramp".to_string(),
            params: vec!["ramp_angle", "ramp_length", "friction"],
        },
    ];

    serde_wasm_bindgen::to_value(&scenarios).unwrap_or(JsValue::NULL)
}

#[derive(serde::Serialize)]
struct ScenarioInfo {
    name: String,
    description: String,
    params: Vec<&'static str>,
}
