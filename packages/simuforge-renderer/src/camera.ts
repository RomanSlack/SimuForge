/**
 * Camera controller for SimuForge renderer
 */

import {
  Scene,
  ArcRotateCamera,
  FreeCamera,
  Vector3,
  PointerEventTypes,
} from '@babylonjs/core';

export type CameraMode = 'orbit' | 'free' | 'follow';

export interface CameraOptions {
  mode?: CameraMode;
  target?: Vector3;
  distance?: number;
  fov?: number;
  nearClip?: number;
  farClip?: number;
}

const defaultOptions: Required<CameraOptions> = {
  mode: 'orbit',
  target: Vector3.Zero(),
  distance: 20,
  fov: 0.8,
  nearClip: 0.1,
  farClip: 1000,
};

/**
 * Camera controller with multiple viewing modes
 */
export class CameraController {
  private scene: Scene;
  private canvas: HTMLCanvasElement;
  private camera: ArcRotateCamera | FreeCamera;
  private mode: CameraMode;
  private followTarget: string | null = null;

  constructor(scene: Scene, canvas: HTMLCanvasElement, options: CameraOptions = {}) {
    this.scene = scene;
    this.canvas = canvas;
    this.mode = options.mode ?? defaultOptions.mode;

    const opts = { ...defaultOptions, ...options };

    if (this.mode === 'orbit') {
      this.camera = this.createOrbitCamera(opts);
    } else {
      this.camera = this.createFreeCamera(opts);
    }

    this.camera.attachControl(canvas, true);
  }

  private createOrbitCamera(opts: Required<CameraOptions>): ArcRotateCamera {
    const camera = new ArcRotateCamera(
      'camera',
      -Math.PI / 2,
      Math.PI / 3,
      opts.distance,
      opts.target,
      this.scene
    );

    camera.fov = opts.fov;
    camera.minZ = opts.nearClip;
    camera.maxZ = opts.farClip;
    camera.wheelPrecision = 20;
    camera.panningSensibility = 100;
    camera.lowerRadiusLimit = 2;
    camera.upperRadiusLimit = 100;

    return camera;
  }

  private createFreeCamera(opts: Required<CameraOptions>): FreeCamera {
    const position = opts.target.add(new Vector3(0, opts.distance / 2, -opts.distance));
    const camera = new FreeCamera('camera', position, this.scene);

    camera.setTarget(opts.target);
    camera.fov = opts.fov;
    camera.minZ = opts.nearClip;
    camera.maxZ = opts.farClip;
    camera.speed = 0.5;
    camera.keysUp.push(87); // W
    camera.keysDown.push(83); // S
    camera.keysLeft.push(65); // A
    camera.keysRight.push(68); // D

    return camera;
  }

  /**
   * Set camera mode
   */
  setMode(mode: CameraMode): void {
    if (mode === this.mode) return;

    const currentTarget = this.getTarget();
    const currentDistance = this.getDistance();

    this.camera.detachControl();
    this.camera.dispose();

    this.mode = mode;

    const opts: Required<CameraOptions> = {
      ...defaultOptions,
      target: currentTarget,
      distance: currentDistance,
    };

    if (mode === 'orbit') {
      this.camera = this.createOrbitCamera(opts);
    } else {
      this.camera = this.createFreeCamera(opts);
    }

    this.camera.attachControl(this.canvas, true);
  }

  /**
   * Get current camera mode
   */
  getMode(): CameraMode {
    return this.mode;
  }

  /**
   * Set camera target
   */
  setTarget(target: Vector3): void {
    if (this.camera instanceof ArcRotateCamera) {
      this.camera.setTarget(target);
    } else {
      this.camera.setTarget(target);
    }
  }

  /**
   * Get current target
   */
  getTarget(): Vector3 {
    if (this.camera instanceof ArcRotateCamera) {
      return this.camera.target.clone();
    } else {
      return (this.camera as FreeCamera).getTarget().clone();
    }
  }

  /**
   * Set camera distance (orbit mode only)
   */
  setDistance(distance: number): void {
    if (this.camera instanceof ArcRotateCamera) {
      this.camera.radius = distance;
    }
  }

  /**
   * Get current distance
   */
  getDistance(): number {
    if (this.camera instanceof ArcRotateCamera) {
      return this.camera.radius;
    } else {
      return this.camera.position.subtract(this.getTarget()).length();
    }
  }

  /**
   * Set body to follow (by name)
   */
  setFollowTarget(bodyName: string | null): void {
    this.followTarget = bodyName;
  }

  /**
   * Update follow target position (call each frame)
   */
  updateFollowTarget(bodyTransforms: Map<string, { position: Vector3 }>): void {
    if (this.followTarget && bodyTransforms.has(this.followTarget)) {
      const transform = bodyTransforms.get(this.followTarget)!;
      this.setTarget(transform.position);
    }
  }

  /**
   * Reset camera to default position
   */
  reset(): void {
    this.setTarget(Vector3.Zero());
    this.setDistance(20);

    if (this.camera instanceof ArcRotateCamera) {
      this.camera.alpha = -Math.PI / 2;
      this.camera.beta = Math.PI / 3;
    }
  }

  /**
   * Frame all visible objects
   */
  frameAll(boundingBox: { min: Vector3; max: Vector3 }): void {
    const center = boundingBox.min.add(boundingBox.max).scale(0.5);
    const size = boundingBox.max.subtract(boundingBox.min);
    const maxDim = Math.max(size.x, size.y, size.z);
    const distance = maxDim * 2;

    this.setTarget(center);
    this.setDistance(distance);
  }

  /**
   * Get underlying Babylon camera
   */
  getCamera(): ArcRotateCamera | FreeCamera {
    return this.camera;
  }

  /**
   * Dispose camera
   */
  dispose(): void {
    this.camera.detachControl();
    this.camera.dispose();
  }
}
