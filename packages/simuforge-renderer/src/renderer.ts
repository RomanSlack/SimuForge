/**
 * Main SimuForge Renderer
 */

import {
  Engine,
  Scene,
  HemisphericLight,
  DirectionalLight,
  Vector3,
  Color3,
  Color4,
} from '@babylonjs/core';
import { CameraController, type CameraMode } from './camera';
import { BodyVisualizer } from './body-visualizer';
import { WasmLoader, type Simulation, type BodyTransform, type MetricFrame, type SimulationReport } from './wasm-loader';

export interface RendererOptions {
  canvas: HTMLCanvasElement;
  antialias?: boolean;
  backgroundColor?: Color4;
  cameraMode?: CameraMode;
  autoResize?: boolean;
}

export interface PlaybackState {
  isPlaying: boolean;
  speed: number;
  currentStep: number;
  totalSteps: number;
}

/**
 * Main renderer for SimuForge physics visualization
 */
export class SimuForgeRenderer {
  private canvas: HTMLCanvasElement;
  private engine: Engine;
  private scene: Scene;
  private camera: CameraController;
  private bodyVisualizer: BodyVisualizer;
  private simulation: Simulation | null = null;

  // Playback state
  private isPlaying = false;
  private playbackSpeed = 1.0;
  private lastFrameTime = 0;
  private accumulatedTime = 0;
  private timestep = 1 / 60;

  // Callbacks
  private onFrameUpdate?: (frame: MetricFrame) => void;
  private onSimulationComplete?: (report: SimulationReport) => void;

  constructor(options: RendererOptions) {
    this.canvas = options.canvas;

    // Create Babylon engine
    this.engine = new Engine(this.canvas, options.antialias ?? true, {
      preserveDrawingBuffer: true,
      stencil: true,
    });

    // Create scene
    this.scene = new Scene(this.engine);
    this.scene.clearColor = options.backgroundColor ?? new Color4(0.15, 0.15, 0.18, 1);

    // Setup camera
    this.camera = new CameraController(this.scene, this.canvas, {
      mode: options.cameraMode ?? 'orbit',
      distance: 25,
    });

    // Setup lighting
    this.setupLighting();

    // Create body visualizer
    this.bodyVisualizer = new BodyVisualizer(this.scene);

    // Auto resize
    if (options.autoResize !== false) {
      window.addEventListener('resize', this.handleResize);
    }

    // Start render loop
    this.engine.runRenderLoop(() => {
      this.update();
      this.scene.render();
    });
  }

  private setupLighting(): void {
    // Ambient light
    const ambient = new HemisphericLight('ambient', new Vector3(0, 1, 0), this.scene);
    ambient.intensity = 0.4;
    ambient.groundColor = new Color3(0.2, 0.2, 0.25);

    // Main directional light
    const main = new DirectionalLight('main', new Vector3(-1, -2, -1).normalize(), this.scene);
    main.intensity = 0.8;
    main.specular = new Color3(0.3, 0.3, 0.3);

    // Fill light
    const fill = new DirectionalLight('fill', new Vector3(1, -0.5, 1).normalize(), this.scene);
    fill.intensity = 0.3;
    fill.specular = new Color3(0.1, 0.1, 0.1);
  }

  private handleResize = (): void => {
    this.engine.resize();
  };

  /**
   * Load simulation from JSON spec
   */
  async loadSimulation(specJson: string): Promise<void> {
    if (this.simulation) {
      this.simulation.free();
    }

    this.simulation = await WasmLoader.createSimulation(specJson);
    this.initializeFromSimulation();
  }

  /**
   * Load simulation from YAML spec
   */
  async loadSimulationFromYaml(yaml: string): Promise<void> {
    if (this.simulation) {
      this.simulation.free();
    }

    this.simulation = await WasmLoader.createSimulationFromYaml(yaml);
    this.initializeFromSimulation();
  }

  private initializeFromSimulation(): void {
    if (!this.simulation) return;

    // Get initial transforms
    const transforms = this.simulation.get_body_transforms() as BodyTransform[];

    // Initialize body visualizations (assuming box shapes for now)
    const shapes = transforms.map(t => ({
      id: t.id,
      name: t.name,
      shape: {
        type: 'box' as const,
        halfExtents: [0.5, 0.5, 0.5] as [number, number, number],
      },
      isStatic: t.name.toLowerCase().includes('ground') ||
                t.name.toLowerCase().includes('floor') ||
                t.name.toLowerCase().includes('ramp'),
    }));

    this.bodyVisualizer.initializeBodies(shapes);
    this.bodyVisualizer.updateTransforms(transforms);

    // Frame all bodies
    const bounds = this.bodyVisualizer.getBoundingBox();
    this.camera.frameAll(bounds);

    this.isPlaying = false;
    this.accumulatedTime = 0;
  }

  /**
   * Update loop
   */
  private update(): void {
    if (!this.simulation || !this.isPlaying) return;

    const now = performance.now();
    const deltaTime = (now - this.lastFrameTime) / 1000;
    this.lastFrameTime = now;

    this.accumulatedTime += deltaTime * this.playbackSpeed;

    // Step simulation when enough time has accumulated
    while (this.accumulatedTime >= this.timestep && !this.simulation.is_complete()) {
      const frame = this.simulation.step();
      this.accumulatedTime -= this.timestep;

      // Update body transforms
      const transforms = this.simulation.get_body_transforms() as BodyTransform[];
      const sleepingBodies = new Set(
        frame.bodies.filter(b => b.sleeping).map(b => b.id)
      );
      this.bodyVisualizer.updateTransforms(transforms, sleepingBodies);

      // Notify frame update
      this.onFrameUpdate?.(frame);
    }

    // Check completion
    if (this.simulation.is_complete()) {
      this.isPlaying = false;
      const report = this.simulation.run_to_completion();
      this.onSimulationComplete?.(report);
    }
  }

  /**
   * Play simulation
   */
  play(): void {
    if (!this.simulation) return;
    this.isPlaying = true;
    this.lastFrameTime = performance.now();
  }

  /**
   * Pause simulation
   */
  pause(): void {
    this.isPlaying = false;
  }

  /**
   * Step simulation forward by one frame
   */
  step(): MetricFrame | null {
    if (!this.simulation || this.simulation.is_complete()) return null;

    const frame = this.simulation.step();
    const transforms = this.simulation.get_body_transforms() as BodyTransform[];
    const sleepingBodies = new Set(
      frame.bodies.filter(b => b.sleeping).map(b => b.id)
    );
    this.bodyVisualizer.updateTransforms(transforms, sleepingBodies);
    this.onFrameUpdate?.(frame);

    return frame;
  }

  /**
   * Reset simulation to initial state
   */
  reset(): void {
    if (!this.simulation) return;

    this.simulation.reset();
    this.isPlaying = false;
    this.accumulatedTime = 0;

    const transforms = this.simulation.get_body_transforms() as BodyTransform[];
    this.bodyVisualizer.updateTransforms(transforms);
  }

  /**
   * Run simulation to completion
   */
  runToCompletion(): SimulationReport | null {
    if (!this.simulation) return null;

    this.isPlaying = false;
    return this.simulation.run_to_completion();
  }

  /**
   * Set playback speed (1.0 = real-time)
   */
  setPlaybackSpeed(speed: number): void {
    this.playbackSpeed = Math.max(0.1, Math.min(10, speed));
  }

  /**
   * Get playback speed
   */
  getPlaybackSpeed(): number {
    return this.playbackSpeed;
  }

  /**
   * Get current playback state
   */
  getPlaybackState(): PlaybackState {
    return {
      isPlaying: this.isPlaying,
      speed: this.playbackSpeed,
      currentStep: this.simulation?.current_step() ?? 0,
      totalSteps: this.simulation?.target_steps() ?? 0,
    };
  }

  /**
   * Set frame update callback
   */
  setOnFrameUpdate(callback: (frame: MetricFrame) => void): void {
    this.onFrameUpdate = callback;
  }

  /**
   * Set simulation complete callback
   */
  setOnSimulationComplete(callback: (report: SimulationReport) => void): void {
    this.onSimulationComplete = callback;
  }

  /**
   * Get camera controller
   */
  getCameraController(): CameraController {
    return this.camera;
  }

  /**
   * Get body visualizer
   */
  getBodyVisualizer(): BodyVisualizer {
    return this.bodyVisualizer;
  }

  /**
   * Get Babylon scene
   */
  getScene(): Scene {
    return this.scene;
  }

  /**
   * Dispose renderer
   */
  dispose(): void {
    window.removeEventListener('resize', this.handleResize);

    if (this.simulation) {
      this.simulation.free();
      this.simulation = null;
    }

    this.bodyVisualizer.dispose();
    this.camera.dispose();
    this.scene.dispose();
    this.engine.dispose();
  }
}
