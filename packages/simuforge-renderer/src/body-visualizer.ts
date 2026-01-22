/**
 * Body visualization for SimuForge renderer
 */

import {
  Scene,
  Mesh,
  MeshBuilder,
  StandardMaterial,
  Color3,
  Vector3,
  Quaternion,
} from '@babylonjs/core';
import type { BodyTransform } from './wasm-loader';

export interface BodyMeshOptions {
  color?: Color3;
  wireframe?: boolean;
  alpha?: number;
  showVelocity?: boolean;
  showName?: boolean;
}

interface BodyMesh {
  mesh: Mesh;
  shape: BodyShape;
  isStatic: boolean;
}

interface BodyShape {
  type: 'box' | 'sphere' | 'capsule' | 'cylinder';
  halfExtents?: [number, number, number];
  radius?: number;
  halfHeight?: number;
}

const defaultColors = {
  dynamic: new Color3(0.3, 0.6, 0.9),
  static: new Color3(0.5, 0.5, 0.5),
  sleeping: new Color3(0.4, 0.4, 0.6),
  selected: new Color3(1.0, 0.8, 0.2),
};

/**
 * Manages visualization of physics bodies
 */
export class BodyVisualizer {
  private scene: Scene;
  private bodies: Map<number, BodyMesh> = new Map();
  private materials: Map<string, StandardMaterial> = new Map();
  private selectedBody: number | null = null;

  constructor(scene: Scene) {
    this.scene = scene;
    this.createMaterials();
  }

  private createMaterials(): void {
    const dynamicMat = new StandardMaterial('dynamicMat', this.scene);
    dynamicMat.diffuseColor = defaultColors.dynamic;
    dynamicMat.specularColor = new Color3(0.2, 0.2, 0.2);
    this.materials.set('dynamic', dynamicMat);

    const staticMat = new StandardMaterial('staticMat', this.scene);
    staticMat.diffuseColor = defaultColors.static;
    staticMat.specularColor = new Color3(0.1, 0.1, 0.1);
    this.materials.set('static', staticMat);

    const sleepingMat = new StandardMaterial('sleepingMat', this.scene);
    sleepingMat.diffuseColor = defaultColors.sleeping;
    sleepingMat.specularColor = new Color3(0.1, 0.1, 0.1);
    this.materials.set('sleeping', sleepingMat);

    const selectedMat = new StandardMaterial('selectedMat', this.scene);
    selectedMat.diffuseColor = defaultColors.selected;
    selectedMat.emissiveColor = new Color3(0.3, 0.2, 0.0);
    this.materials.set('selected', selectedMat);

    // Ground material
    const groundMat = new StandardMaterial('groundMat', this.scene);
    groundMat.diffuseColor = new Color3(0.35, 0.35, 0.35);
    groundMat.specularColor = new Color3(0.05, 0.05, 0.05);
    this.materials.set('ground', groundMat);
  }

  /**
   * Create or update body meshes from shapes
   */
  initializeBodies(shapes: Array<{
    id: number;
    name: string;
    shape: BodyShape;
    isStatic: boolean;
  }>): void {
    // Remove bodies that no longer exist
    const newIds = new Set(shapes.map(s => s.id));
    for (const [id, body] of this.bodies) {
      if (!newIds.has(id)) {
        body.mesh.dispose();
        this.bodies.delete(id);
      }
    }

    // Create or update bodies
    for (const shapeInfo of shapes) {
      if (this.bodies.has(shapeInfo.id)) {
        continue; // Already exists
      }

      const mesh = this.createMeshForShape(shapeInfo.shape, shapeInfo.name);

      // Set material
      const isGround = shapeInfo.name.toLowerCase().includes('ground') ||
                       shapeInfo.name.toLowerCase().includes('floor');
      if (isGround) {
        mesh.material = this.materials.get('ground')!;
      } else if (shapeInfo.isStatic) {
        mesh.material = this.materials.get('static')!;
      } else {
        mesh.material = this.materials.get('dynamic')!;
      }

      this.bodies.set(shapeInfo.id, {
        mesh,
        shape: shapeInfo.shape,
        isStatic: shapeInfo.isStatic,
      });
    }
  }

  private createMeshForShape(shape: BodyShape, name: string): Mesh {
    switch (shape.type) {
      case 'box': {
        const he = shape.halfExtents ?? [0.5, 0.5, 0.5];
        return MeshBuilder.CreateBox(name, {
          width: he[0] * 2,
          height: he[1] * 2,
          depth: he[2] * 2,
        }, this.scene);
      }
      case 'sphere': {
        return MeshBuilder.CreateSphere(name, {
          diameter: (shape.radius ?? 0.5) * 2,
          segments: 16,
        }, this.scene);
      }
      case 'capsule': {
        const radius = shape.radius ?? 0.5;
        const halfHeight = shape.halfHeight ?? 0.5;
        return MeshBuilder.CreateCapsule(name, {
          radius,
          height: (halfHeight + radius) * 2,
          tessellation: 16,
        }, this.scene);
      }
      case 'cylinder': {
        return MeshBuilder.CreateCylinder(name, {
          diameter: (shape.radius ?? 0.5) * 2,
          height: (shape.halfHeight ?? 0.5) * 2,
          tessellation: 16,
        }, this.scene);
      }
      default:
        // Default to box
        return MeshBuilder.CreateBox(name, { size: 1 }, this.scene);
    }
  }

  /**
   * Update body transforms from simulation
   */
  updateTransforms(transforms: BodyTransform[], sleepingBodies?: Set<number>): void {
    for (const transform of transforms) {
      const body = this.bodies.get(transform.id);
      if (!body) continue;

      // Update position
      body.mesh.position.set(
        transform.position[0],
        transform.position[1],
        transform.position[2]
      );

      // Update rotation (quaternion: x, y, z, w)
      body.mesh.rotationQuaternion = new Quaternion(
        transform.rotation[0],
        transform.rotation[1],
        transform.rotation[2],
        transform.rotation[3]
      );

      // Update material based on state
      if (transform.id === this.selectedBody) {
        body.mesh.material = this.materials.get('selected')!;
      } else if (sleepingBodies?.has(transform.id)) {
        body.mesh.material = this.materials.get('sleeping')!;
      } else if (body.isStatic) {
        const isGround = transform.name.toLowerCase().includes('ground') ||
                         transform.name.toLowerCase().includes('floor');
        body.mesh.material = this.materials.get(isGround ? 'ground' : 'static')!;
      } else {
        body.mesh.material = this.materials.get('dynamic')!;
      }
    }
  }

  /**
   * Select a body by ID
   */
  selectBody(id: number | null): void {
    this.selectedBody = id;
  }

  /**
   * Get selected body ID
   */
  getSelectedBody(): number | null {
    return this.selectedBody;
  }

  /**
   * Get body mesh by ID
   */
  getBodyMesh(id: number): Mesh | undefined {
    return this.bodies.get(id)?.mesh;
  }

  /**
   * Get all body meshes
   */
  getAllBodyMeshes(): Mesh[] {
    return Array.from(this.bodies.values()).map(b => b.mesh);
  }

  /**
   * Get bounding box of all bodies
   */
  getBoundingBox(): { min: Vector3; max: Vector3 } {
    let min = new Vector3(Infinity, Infinity, Infinity);
    let max = new Vector3(-Infinity, -Infinity, -Infinity);

    for (const body of this.bodies.values()) {
      const pos = body.mesh.position;
      min = Vector3.Minimize(min, pos);
      max = Vector3.Maximize(max, pos);
    }

    // Add some padding
    const padding = new Vector3(2, 2, 2);
    return {
      min: min.subtract(padding),
      max: max.add(padding),
    };
  }

  /**
   * Clear all bodies
   */
  clear(): void {
    for (const body of this.bodies.values()) {
      body.mesh.dispose();
    }
    this.bodies.clear();
    this.selectedBody = null;
  }

  /**
   * Dispose visualizer
   */
  dispose(): void {
    this.clear();
    for (const material of this.materials.values()) {
      material.dispose();
    }
    this.materials.clear();
  }
}
