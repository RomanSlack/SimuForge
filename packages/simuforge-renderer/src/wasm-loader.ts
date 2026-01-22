/**
 * WASM module loader for SimuForge
 */

export interface BodyTransform {
  id: number;
  name: string;
  position: [number, number, number];
  rotation: [number, number, number, number];
}

export interface MetricFrame {
  step: number;
  time: number;
  energy: {
    kinetic: number;
    potential: number;
    total: number;
  };
  momentum: {
    linear: { x: number; y: number; z: number };
    angular: { x: number; y: number; z: number };
    linear_magnitude: number;
    angular_magnitude: number;
  };
  contacts: {
    contact_count: number;
    max_penetration: number;
    total_penetration: number;
    constraint_violations: number;
  };
  bodies: Array<{
    id: number;
    name: string;
    transform: {
      position: { x: number; y: number; z: number };
      rotation: { x: number; y: number; z: number; w: number };
    };
    velocity: { x: number; y: number; z: number };
    angular_velocity: { x: number; y: number; z: number };
    sleeping: boolean;
  }>;
}

export interface SimulationReport {
  status: 'pending' | 'passed' | 'failed' | 'error';
  experiment_name: string;
  total_steps: number;
  total_time: number;
  metrics: {
    energy_drift_percent: number;
    initial_energy: number;
    final_energy: number;
    max_penetration_ever: number;
    total_constraint_violations: number;
    stabilization_step: number | null;
    stability_time: number | null;
    average_contact_count: number;
    frame_count: number;
  };
  criteria_results: Record<string, {
    value: number;
    min?: number;
    max?: number;
    passed: boolean;
  }>;
  baseline_comparison?: {
    baseline_name: string;
    metrics_improved: string[];
    metrics_regressed: string[];
    recommendation: 'ACCEPT' | 'REJECT' | 'REVIEW';
  };
  error?: string;
}

export interface Simulation {
  step(): MetricFrame;
  get_frame(): MetricFrame;
  get_body_transforms(): BodyTransform[];
  run_to_completion(): SimulationReport;
  current_step(): number;
  current_time(): number;
  target_steps(): number;
  is_complete(): boolean;
  body_count(): number;
  reset(): void;
  free(): void;
}

export interface WasmModule {
  Simulation: new (specJson: string) => Simulation;
  create_simulation_from_yaml(yaml: string): Simulation;
  validate_spec(specJson: string): string;
  get_available_scenarios(): Array<{
    name: string;
    description: string;
    params: string[];
  }>;
}

let wasmModule: WasmModule | null = null;
let initPromise: Promise<WasmModule> | null = null;

/**
 * Load and initialize the WASM module
 */
export class WasmLoader {
  private static wasmPath: string = './pkg/simuforge_wasm.js';

  /**
   * Set custom path to WASM module
   */
  static setWasmPath(path: string): void {
    WasmLoader.wasmPath = path;
  }

  /**
   * Load the WASM module (cached)
   */
  static async load(): Promise<WasmModule> {
    if (wasmModule) {
      return wasmModule;
    }

    if (initPromise) {
      return initPromise;
    }

    initPromise = (async () => {
      try {
        const wasm = await import(/* @vite-ignore */ WasmLoader.wasmPath);
        await wasm.default();
        wasmModule = wasm as unknown as WasmModule;
        return wasmModule;
      } catch (error) {
        initPromise = null;
        throw error;
      }
    })();

    return initPromise;
  }

  /**
   * Check if WASM is loaded
   */
  static isLoaded(): boolean {
    return wasmModule !== null;
  }

  /**
   * Create a simulation from JSON spec
   */
  static async createSimulation(specJson: string): Promise<Simulation> {
    const wasm = await WasmLoader.load();
    return new wasm.Simulation(specJson);
  }

  /**
   * Create a simulation from YAML spec
   */
  static async createSimulationFromYaml(yaml: string): Promise<Simulation> {
    const wasm = await WasmLoader.load();
    return wasm.create_simulation_from_yaml(yaml);
  }

  /**
   * Validate a spec without creating simulation
   */
  static async validateSpec(specJson: string): Promise<{ valid: boolean; error?: string }> {
    const wasm = await WasmLoader.load();
    const result = wasm.validate_spec(specJson);

    if (result === 'valid') {
      return { valid: true };
    }
    return { valid: false, error: result };
  }

  /**
   * Get available built-in scenarios
   */
  static async getAvailableScenarios(): Promise<Array<{
    name: string;
    description: string;
    params: string[];
  }>> {
    const wasm = await WasmLoader.load();
    return wasm.get_available_scenarios();
  }
}
