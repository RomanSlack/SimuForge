/**
 * SimuForge Web Application
 */

import { SimuForgeRenderer, WasmLoader, type MetricFrame, type SimulationReport } from '@simuforge/renderer';

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

let renderer: SimuForgeRenderer | null = null;
let isPlaying = false;

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
 * Update metrics display
 */
function updateMetrics(frame: MetricFrame): void {
  energyValue.textContent = frame.energy.total.toFixed(2);
  kineticValue.textContent = frame.energy.kinetic.toFixed(2);
  potentialValue.textContent = frame.energy.potential.toFixed(2);
  contactsValue.textContent = frame.contacts.contact_count.toString();
  penetrationValue.textContent = frame.contacts.max_penetration.toFixed(6);

  stepDisplay.textContent = `Step: ${frame.step}`;
  timeDisplay.textContent = `Time: ${frame.time.toFixed(3)}s`;
}

/**
 * Update playback state display
 */
function updatePlaybackState(): void {
  if (!renderer) return;

  const state = renderer.getPlaybackState();
  stepDisplay.textContent = `Step: ${state.currentStep} / ${state.totalSteps}`;

  playBtn.textContent = state.isPlaying ? '⏸' : '▶';
  isPlaying = state.isPlaying;
}

/**
 * Handle simulation complete
 */
function handleSimulationComplete(report: SimulationReport): void {
  setStatus(`Complete: ${report.status}`, report.status === 'passed' ? 'success' : 'error');
  console.log('Simulation Report:', report);
  updatePlaybackState();
}

/**
 * Set status message
 */
function setStatus(message: string, type: 'success' | 'error' | 'warning' | 'info' = 'info'): void {
  statusEl.textContent = message;
  statusEl.className = type === 'info' ? '' : type;
}

/**
 * Initialize application
 */
async function init(): Promise<void> {
  try {
    // Load WASM module
    setStatus('Loading WASM...');
    await WasmLoader.load();

    // Create renderer
    renderer = new SimuForgeRenderer({
      canvas,
      antialias: true,
      autoResize: true,
    });

    // Set up callbacks
    renderer.setOnFrameUpdate(updateMetrics);
    renderer.setOnSimulationComplete(handleSimulationComplete);

    // Hide loading overlay
    loadingEl.classList.add('hidden');
    setStatus('Ready');

    // Load default experiment
    await loadExperiment();
  } catch (error) {
    console.error('Initialization error:', error);
    setStatus(`Error: ${error}`, 'error');
  }
}

/**
 * Load experiment
 */
async function loadExperiment(): Promise<void> {
  if (!renderer) return;

  try {
    setStatus('Loading experiment...');
    const spec = generateSpec();
    await renderer.loadSimulation(spec);
    setStatus('Experiment loaded');
    updatePlaybackState();

    // Reset metrics display
    energyValue.textContent = '-';
    kineticValue.textContent = '-';
    potentialValue.textContent = '-';
    contactsValue.textContent = '-';
    penetrationValue.textContent = '-';
  } catch (error) {
    console.error('Load error:', error);
    setStatus(`Load error: ${error}`, 'error');
  }
}

// Event listeners
loadBtn.addEventListener('click', loadExperiment);

resetBtn.addEventListener('click', () => {
  renderer?.reset();
  updatePlaybackState();
  setStatus('Reset');
});

playBtn.addEventListener('click', () => {
  if (!renderer) return;

  if (isPlaying) {
    renderer.pause();
    setStatus('Paused');
  } else {
    renderer.play();
    setStatus('Playing');
  }
  updatePlaybackState();
});

stepBtn.addEventListener('click', () => {
  if (!renderer) return;

  const frame = renderer.step();
  if (frame) {
    updateMetrics(frame);
  }
  updatePlaybackState();
});

speedInput.addEventListener('input', () => {
  const speed = parseFloat(speedInput.value);
  renderer?.setPlaybackSpeed(speed);
  speedDisplay.textContent = `${speed.toFixed(1)}x`;
});

// Start application
init();
