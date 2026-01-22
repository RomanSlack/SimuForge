/**
 * SimuForge Web Application
 */

// Import WASM module directly
import init, { Simulation, get_available_scenarios, validate_spec } from '../../simuforge-renderer/pkg/simuforge_wasm.js';

// Import renderer (without WASM loader for now)
import {
  Engine,
  Scene,
  ArcRotateCamera,
  HemisphericLight,
  DirectionalLight,
  Vector3,
  Color3,
  Color4,
  MeshBuilder,
  StandardMaterial,
  Quaternion,
} from '@babylonjs/core';

// DOM elements
const canvas = document.getElementById('render-canvas') as HTMLCanvasElement;
const loadingEl = document.getElementById('loading') as HTMLElement;
const statusEl = document.getElementById('status') as HTMLElement;

const scenarioSelect = document.getElementById('scenario-select') as HTMLSelectElement;
const stepsInput = document.getElementById('steps-input') as HTMLInputElement;
const timestepInput = document.getElementById('timestep-input') as HTMLInputElement;
const loadBtn = document.getElementById('load-btn') as HTMLButtonElement;

const resetBtn = document.getElementById('reset-btn') as HTMLButtonElement;
const playBtn = document.getElementById('play-btn') as HTMLButtonElement;
const stepBtn = document.getElementById('step-btn') as HTMLButtonElement;
const speedInput = document.getElementById('speed-input') as HTMLInputElement;
const speedDisplay = document.getElementById('speed-display') as HTMLSpanElement;

const stepDisplay = document.getElementById('step-display') as HTMLSpanElement;
const timeDisplay = document.getElementById('time-display') as HTMLSpanElement;

const energyValue = document.getElementById('energy-value') as HTMLSpanElement;
const kineticValue = document.getElementById('kinetic-value') as HTMLSpanElement;
const potentialValue = document.getElementById('potential-value') as HTMLSpanElement;
const contactsValue = document.getElementById('contacts-value') as HTMLSpanElement;
const penetrationValue = document.getElementById('penetration-value') as HTMLSpanElement;

// Babylon.js setup
let engine: Engine;
let scene: Scene;
let simulation: Simulation | null = null;
let bodyMeshes: Map<number, any> = new Map();
let isPlaying = false;
let playbackSpeed = 1.0;

/**
 * Generate experiment spec JSON from form inputs
 */
function generateSpec(): string {
  const scenario = scenarioSelect.value;
  const steps = parseInt(stepsInput.value, 10);
  const timestep = parseFloat(timestepInput.value);

  const spec = {
    apiVersion: 'simuforge/v1',
    kind: 'Experiment',
    metadata: {
      name: `${scenario}-${Date.now()}`,
    },
    spec: {
      physics: {
        timestep,
        gravity: [0, -9.81, 0],
        solver_iterations: 8,
        enhanced_determinism: true,
      },
      duration: {
        type: 'fixed',
        steps,
      },
      scenario: {
        type: 'builtin',
        name: scenario,
        params: getScenarioParams(scenario),
      },
      metrics: {
        per_frame: ['total_energy', 'momentum', 'penetration', 'contacts'],
        aggregate: ['energy_drift_percent', 'max_penetration', 'stability_time'],
      },
      criteria: {},
    },
  };

  return JSON.stringify(spec);
}

/**
 * Get default parameters for each scenario
 */
function getScenarioParams(scenario: string): Record<string, unknown> {
  switch (scenario) {
    case 'box_stack':
      return { count: 10, box_size: [1, 1, 1], friction: 0.5 };
    case 'rolling_sphere':
      return { radius: 0.5, initial_velocity: [5, 0, 0], friction: 0.5 };
    case 'bouncing_ball':
      return { radius: 0.5, drop_height: 10, restitution: 0.8 };
    case 'friction_ramp':
      return { ramp_angle: 0.5, ramp_length: 10, friction: 0.3 };
    default:
      return {};
  }
}

/**
 * Create body meshes from simulation
 */
function createBodyMeshes() {
  // Clear existing meshes
  bodyMeshes.forEach(mesh => mesh.dispose());
  bodyMeshes.clear();

  if (!simulation) return;

  const transforms = simulation.get_body_transforms() as any[];

  // Create materials
  const dynamicMat = new StandardMaterial('dynamicMat', scene);
  dynamicMat.diffuseColor = new Color3(0.3, 0.6, 0.9);

  const staticMat = new StandardMaterial('staticMat', scene);
  staticMat.diffuseColor = new Color3(0.4, 0.4, 0.4);

  const groundMat = new StandardMaterial('groundMat', scene);
  groundMat.diffuseColor = new Color3(0.35, 0.35, 0.35);

  for (const t of transforms) {
    const isGround = t.name.includes('ground') || t.name.includes('floor');
    const isRamp = t.name.includes('ramp');
    const isSphere = t.name.includes('sphere') || t.name.includes('ball');

    let mesh;

    if (isGround) {
      // Ground plane - large flat box
      mesh = MeshBuilder.CreateBox(t.name, { width: 100, height: 1, depth: 100 }, scene);
      mesh.material = groundMat;
    } else if (isRamp) {
      // Ramp
      mesh = MeshBuilder.CreateBox(t.name, { width: 10, height: 1, depth: 4 }, scene);
      mesh.material = staticMat;
    } else if (isSphere) {
      // Sphere
      mesh = MeshBuilder.CreateSphere(t.name, { diameter: 1, segments: 16 }, scene);
      mesh.material = dynamicMat;
    } else {
      // Default box (stacked boxes etc)
      mesh = MeshBuilder.CreateBox(t.name, { size: 1 }, scene);
      mesh.material = dynamicMat;
    }

    // Set initial transform
    mesh.position = new Vector3(t.position[0], t.position[1], t.position[2]);
    mesh.rotationQuaternion = new Quaternion(t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]);

    bodyMeshes.set(t.id, mesh);
  }
}

/**
 * Update body meshes from simulation state
 */
function updateBodyMeshes() {
  if (!simulation) return;

  const transforms = simulation.get_body_transforms() as any[];

  for (const t of transforms) {
    const mesh = bodyMeshes.get(t.id);
    if (mesh) {
      mesh.position.set(t.position[0], t.position[1], t.position[2]);
      mesh.rotationQuaternion = new Quaternion(t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]);
    }
  }
}

/**
 * Update metrics display
 */
function updateMetrics(frame: any): void {
  energyValue.textContent = frame.energy.total.toFixed(2);
  kineticValue.textContent = frame.energy.kinetic.toFixed(2);
  potentialValue.textContent = frame.energy.potential.toFixed(2);
  contactsValue.textContent = frame.contacts.contact_count.toString();
  penetrationValue.textContent = frame.contacts.max_penetration.toFixed(6);

  stepDisplay.textContent = `Step: ${simulation?.current_step() ?? 0} / ${simulation?.target_steps() ?? 0}`;
  timeDisplay.textContent = `Time: ${frame.time.toFixed(3)}s`;
}

/**
 * Set status message
 */
function setStatus(message: string, type: 'success' | 'error' | 'warning' | 'info' = 'info'): void {
  statusEl.textContent = message;
  statusEl.className = type === 'info' ? '' : type;
}

/**
 * Load experiment
 */
async function loadExperiment(): Promise<void> {
  try {
    setStatus('Loading experiment...');

    if (simulation) {
      simulation.free();
    }

    const spec = generateSpec();
    simulation = new Simulation(spec);

    createBodyMeshes();

    setStatus('Experiment loaded');

    // Reset metrics display
    energyValue.textContent = '-';
    kineticValue.textContent = '-';
    potentialValue.textContent = '-';
    contactsValue.textContent = '-';
    penetrationValue.textContent = '-';
    stepDisplay.textContent = `Step: 0 / ${simulation.target_steps()}`;
    timeDisplay.textContent = 'Time: 0.000s';

    playBtn.textContent = '▶';
    isPlaying = false;
  } catch (error) {
    console.error('Load error:', error);
    setStatus(`Load error: ${error}`, 'error');
  }
}

/**
 * Animation loop
 */
let lastTime = 0;
let accumulator = 0;
const timestep = 1/60;

function animate(currentTime: number) {
  const deltaTime = (currentTime - lastTime) / 1000;
  lastTime = currentTime;

  if (isPlaying && simulation && !simulation.is_complete()) {
    accumulator += deltaTime * playbackSpeed;

    while (accumulator >= timestep) {
      const frame = simulation.step();
      updateMetrics(frame);
      accumulator -= timestep;
    }

    updateBodyMeshes();

    if (simulation.is_complete()) {
      isPlaying = false;
      playBtn.textContent = '▶';
      setStatus('Simulation complete');
    }
  }

  scene.render();
  requestAnimationFrame(animate);
}

/**
 * Initialize application
 */
async function initApp(): Promise<void> {
  try {
    // Load WASM module
    setStatus('Loading WASM...');
    await init();

    // Create Babylon engine
    engine = new Engine(canvas, true, { preserveDrawingBuffer: true, stencil: true });
    scene = new Scene(engine);
    scene.clearColor = new Color4(0.15, 0.15, 0.18, 1);

    // Camera
    const camera = new ArcRotateCamera('camera', -Math.PI / 2, Math.PI / 3, 25, Vector3.Zero(), scene);
    camera.attachControl(canvas, true);
    camera.wheelPrecision = 20;

    // Lights
    const ambient = new HemisphericLight('ambient', new Vector3(0, 1, 0), scene);
    ambient.intensity = 0.6;

    const main = new DirectionalLight('main', new Vector3(-1, -2, -1).normalize(), scene);
    main.intensity = 0.8;

    // Hide loading overlay
    loadingEl.classList.add('hidden');
    setStatus('Ready');

    // Load default experiment
    await loadExperiment();

    // Start render loop
    lastTime = performance.now();
    requestAnimationFrame(animate);

    // Handle resize
    window.addEventListener('resize', () => engine.resize());
  } catch (error) {
    console.error('Initialization error:', error);
    setStatus(`Error: ${error}`, 'error');
  }
}

// Event listeners
loadBtn.addEventListener('click', loadExperiment);

resetBtn.addEventListener('click', () => {
  if (simulation) {
    simulation.reset();
    createBodyMeshes();
    setStatus('Reset');
    playBtn.textContent = '▶';
    isPlaying = false;
  }
});

playBtn.addEventListener('click', () => {
  if (!simulation) return;

  if (isPlaying) {
    isPlaying = false;
    playBtn.textContent = '▶';
    setStatus('Paused');
  } else {
    isPlaying = true;
    playBtn.textContent = '⏸';
    setStatus('Playing');
  }
});

stepBtn.addEventListener('click', () => {
  if (!simulation || simulation.is_complete()) return;

  const frame = simulation.step();
  updateMetrics(frame);
  updateBodyMeshes();
});

speedInput.addEventListener('input', () => {
  playbackSpeed = parseFloat(speedInput.value);
  speedDisplay.textContent = `${playbackSpeed.toFixed(1)}x`;
});

// Start application
initApp();
