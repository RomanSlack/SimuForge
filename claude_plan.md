Here is Claude's plan:                                                                                       
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 SimuForge Implementation Plan                                                                                
                                                                                                              
 Physics Simulation Harness for Claude Code                                                                   
                                                                                                              
 Goal: Build a physics simulation harness where Claude can run experiments, measure outcomes, and iterate     
 based on structured feedback.                                                                                
                                                                                                              
 Stack:                                                                                                       
 - Physics: Rapier (Rust) - mature, deterministic, WASM-ready                                                 
 - Rendering: Babylon.js with WebGPU backend (WebGL2 fallback)                                                
 - Build: wasm-pack + wasm-bindgen for Rust-to-WASM                                                           
                                                                                                              
 First Domain: Rigid body dynamics with contacts                                                              
                                                                                                              
 ---                                                                                                          
 Repository Structure                                                                                         
                                                                                                              
 SimuForge/                                                                                                   
 ├── Cargo.toml                    # Workspace root                                                           
 ├── rust-toolchain.toml           # Pin Rust version                                                         
 ├── package.json                  # JS workspace scripts                                                     
 │                                                                                                            
 ├── crates/                                                                                                  
 │   ├── simuforge-core/           # Core types, traits (physics-agnostic)                                    
 │   ├── simuforge-physics/        # Rapier integration + metrics                                             
 │   ├── simuforge-harness/        # CLI for experiments                                                      
 │   └── simuforge-wasm/           # Browser bindings                                                         
 │                                                                                                            
 ├── packages/                                                                                                
 │   ├── simuforge-renderer/       # Babylon.js visualization                                                 
 │   └── simuforge-web/            # Web application                                                          
 │                                                                                                            
 ├── experiments/                                                                                             
 │   ├── schemas/                  # JSON schemas                                                             
 │   ├── benchmarks/               # Standard test specs                                                      
 │   └── baselines/                # Reference results                                                        
 │                                                                                                            
 ├── results/                      # Output (gitignored)                                                      
 └── scripts/                      # Build utilities                                                          
                                                                                                              
 ---                                                                                                          
 Crate Organization                                                                                           
                                                                                                              
 simuforge-core                                                                                               
                                                                                                              
 Shared types: Vec3, Quat, Transform, ExperimentSpec, MetricFrame, SimulationReport                           
                                                                                                              
 simuforge-physics                                                                                            
                                                                                                              
 Rapier wrapper with metric hooks:                                                                            
 - MetricWorld - wraps Rapier, collects metrics each step                                                     
 - Metrics: energy, momentum, penetration, constraint violations                                              
 - Scenario builders: box_stack(), rolling(), bouncing()                                                      
                                                                                                              
 simuforge-harness                                                                                            
                                                                                                              
 CLI tool:                                                                                                    
 simuforge run experiment.yaml -o results/                                                                    
 simuforge run experiment.yaml --baseline baselines/ref.json                                                  
 simuforge baseline experiment.yaml -o baselines/                                                             
 simuforge suite experiments/benchmarks/                                                                      
                                                                                                              
 simuforge-wasm                                                                                               
                                                                                                              
 Browser API:                                                                                                 
 Simulation::new(spec_json) -> Simulation                                                                     
 simulation.step() -> MetricFrame                                                                             
 simulation.get_bodies() -> BodyTransforms                                                                    
 simulation.run_to_completion() -> Report                                                                     
                                                                                                              
 ---                                                                                                          
 Experiment Spec (YAML)                                                                                       
                                                                                                              
 apiVersion: simuforge/v1                                                                                     
 kind: Experiment                                                                                             
 metadata:                                                                                                    
   name: box-stack-10                                                                                         
 spec:                                                                                                        
   physics:                                                                                                   
     timestep: 0.016666667                                                                                    
     gravity: [0, -9.81, 0]                                                                                   
     solver_iterations: 8                                                                                     
     enhanced_determinism: true                                                                               
     seed: 42                                                                                                 
   duration:                                                                                                  
     type: fixed                                                                                              
     steps: 600                                                                                               
   scenario:                                                                                                  
     type: builtin                                                                                            
     name: box_stack                                                                                          
     params:                                                                                                  
       count: 10                                                                                              
       box_size: [1, 1, 1]                                                                                    
   metrics:                                                                                                   
     per_frame: [total_energy, momentum, penetration, contacts]                                               
     aggregate: [energy_drift_percent, max_penetration, stability_time]                                       
   criteria:                                                                                                  
     energy_drift_percent: { max: 5.0 }                                                                       
     max_penetration_ever: { max: 0.01 }                                                                      
                                                                                                              
 ---                                                                                                          
 Report Structure (JSON)                                                                                      
                                                                                                              
 {                                                                                                            
   "status": "passed",                                                                                        
   "metrics": {                                                                                               
     "energy": { "drift_percent": -2.04 },                                                                    
     "contacts": { "max_penetration": 0.0023 },                                                               
     "stability": { "stabilization_step": 245 }                                                               
   },                                                                                                         
   "criteria_results": {                                                                                      
     "energy_drift_percent": { "value": -2.04, "max": 5.0, "passed": true }                                   
   },                                                                                                         
   "baseline_comparison": {                                                                                   
     "metrics_improved": ["energy_drift"],                                                                    
     "recommendation": "ACCEPT"                                                                               
   }                                                                                                          
 }                                                                                                            
                                                                                                              
 ---                                                                                                          
 Claude Feedback Loop                                                                                         
                                                                                                              
 1. Receive experiment spec + baseline metrics                                                                
 2. Analyze current physics code + failing metrics                                                            
 3. Propose code changes (solver tuning, contact handling, etc.)                                              
 4. Apply changes via Claude Code edits                                                                       
 5. Rebuild cargo build --release                                                                             
 6. Run simuforge run spec.yaml --baseline ref.json                                                           
 7. Evaluate structured JSON output                                                                           
 8. Iterate or accept changes                                                                                 
                                                                                                              
 ---                                                                                                          
 MVP Milestones                                                                                               
                                                                                                              
 Phase 1: Core Physics (Week 1-2)                                                                             
                                                                                                              
 - Cargo workspace scaffolding                                                                                
 - simuforge-core types (Vec3, ExperimentSpec, MetricFrame)                                                   
 - simuforge-physics MetricWorld wrapper                                                                      
 - Energy, momentum, contact metrics                                                                          
 - Determinism config (enhanced-determinism feature)                                                          
                                                                                                              
 Phase 2: CLI Harness (Week 2-3)                                                                              
                                                                                                              
 - YAML spec parser                                                                                           
 - run command with JSON output                                                                               
 - Baseline save/compare                                                                                      
 - Pass/fail evaluation                                                                                       
                                                                                                              
 Phase 3: WASM + Visualization (Week 3-4)                                                                     
                                                                                                              
 - wasm-bindgen exports                                                                                       
 - wasm-pack build pipeline                                                                                   
 - Babylon.js WebGPU/WebGL engine                                                                             
 - Real-time body rendering                                                                                   
 - Playback controls                                                                                          
                                                                                                              
 Phase 4: Benchmark Suite (Week 4-5)                                                                          
                                                                                                              
 - Box stack (10, 50, 100)                                                                                    
 - Rolling sphere                                                                                             
 - Bouncing ball                                                                                              
 - Friction ramp                                                                                              
 - Suite runner command                                                                                       
                                                                                                              
 Phase 5: Polish (Week 5-6)                                                                                   
                                                                                                              
 - Claude workflow docs                                                                                       
 - Web UI experiment editor                                                                                   
 - Results visualization                                                                                      
                                                                                                              
 ---                                                                                                          
 Critical Files                                                                                               
 ┌────────────────────────────────────────────────┬──────────────────────────────────────────┐                
 │                      File                      │                 Purpose                  │                
 ├────────────────────────────────────────────────┼──────────────────────────────────────────┤                
 │ crates/simuforge-physics/src/world.rs          │ MetricWorld - Rapier wrapper with hooks  │                
 ├────────────────────────────────────────────────┼──────────────────────────────────────────┤                
 │ crates/simuforge-physics/src/metrics/mod.rs    │ Metric collection implementations        │                
 ├────────────────────────────────────────────────┼──────────────────────────────────────────┤                
 │ crates/simuforge-harness/src/runner.rs         │ Experiment execution + report generation │                
 ├────────────────────────────────────────────────┼──────────────────────────────────────────┤                
 │ experiments/schemas/experiment.schema.json     │ Canonical experiment spec schema         │                
 ├────────────────────────────────────────────────┼──────────────────────────────────────────┤                
 │ packages/simuforge-renderer/src/wasm-loader.ts │ WASM-to-JS bridge                        │                
 └────────────────────────────────────────────────┴──────────────────────────────────────────┘                
 ---                                                                                                          
 Determinism Requirements                                                                                     
                                                                                                              
 1. Enable enhanced-determinism feature in Rapier                                                             
 2. Fixed timestep only (no variable dt)                                                                      
 3. Consistent body insertion order                                                                           
 4. Explicit RNG seeds                                                                                        
 5. Pin Rust version in rust-toolchain.toml                                                                   
                                                                                                              
 ---                                                                                                          
 Verification                                                                                                 
                                                                                                              
 After implementation:                                                                                        
 1. Run cargo build --release - should compile all crates                                                     
 2. Run simuforge run experiments/benchmarks/box-stack-10.yaml - should produce JSON report                   
 3. Run benchmark twice with same seed - should produce identical results (determinism check)                 
 4. Open web UI, load experiment, see 3D visualization                                                        
 5. Claude can run harness, parse output, propose changes, verify improvements       