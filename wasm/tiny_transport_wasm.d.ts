/* tslint:disable */
/* eslint-disable */
/**
*/
export class Dataset {
  free(): void;
/**
* @param {Uint8Array} data 
* @returns {Dataset} 
*/
  static parse(data: Uint8Array): Dataset;
/**
* @param {number} time_passed 
*/
  update(time_passed: number): void;
/**
* @param {View} view 
* @param {number} x 
* @param {number} y 
* @returns {string | undefined} 
*/
  findStation(view: View, x: number, y: number): string | undefined;
/**
* @returns {number} 
*/
  stationCount(): number;
/**
* @returns {Float32Array} 
*/
  stationPositions(): Float32Array;
/**
* @returns {Uint8Array} 
*/
  stationTypes(): Uint8Array;
/**
* @returns {number} 
*/
  lineCount(): number;
/**
* @returns {Float32Array} 
*/
  lineColors(): Float32Array;
/**
* @returns {Uint32Array} 
*/
  lineVerticesSizes(): Uint32Array;
/**
* @returns {Float32Array} 
*/
  lineVertices(): Float32Array;
/**
* @returns {string} 
*/
  lineNames(): string;
/**
* @returns {number} 
*/
  trainCount(): number;
/**
* @returns {Float32Array} 
*/
  trainVertices(): Float32Array;
/**
* @returns {Float32Array} 
*/
  trainColors(): Float32Array;
/**
* @returns {Uint16Array} 
*/
  trainLineNumbers(): Uint16Array;
/**
* @returns {Uint8Array} 
*/
  trainSides(): Uint8Array;
/**
* @returns {Float32Array} 
*/
  trainExtents(): Float32Array;
}
/**
*/
export class View {
  free(): void;
/**
* @param {number} scaling 
* @param {number} width 
* @param {number} height 
*/
  constructor(scaling: number, width: number, height: number);
/**
* @param {number} width 
* @param {number} height 
*/
  resize(width: number, height: number): void;
/**
* @returns {number} 
*/
  scaling(): number;
/**
* @returns {Float32Array} 
*/
  calculateViewProjection(): Float32Array;
/**
* @param {number} x 
* @param {number} y 
*/
  scroll(x: number, y: number): void;
/**
* @param {number} scaling 
* @param {number} x 
* @param {number} y 
*/
  zoom(scaling: number, x: number, y: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_dataset_free: (a: number) => void;
  readonly dataset_parse: (a: number, b: number) => number;
  readonly dataset_update: (a: number, b: number) => void;
  readonly dataset_findStation: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly dataset_stationCount: (a: number) => number;
  readonly dataset_stationPositions: (a: number, b: number) => void;
  readonly dataset_stationTypes: (a: number, b: number) => void;
  readonly dataset_lineCount: (a: number) => number;
  readonly dataset_lineColors: (a: number, b: number) => void;
  readonly dataset_lineVerticesSizes: (a: number, b: number) => void;
  readonly dataset_lineVertices: (a: number, b: number) => void;
  readonly dataset_lineNames: (a: number, b: number) => void;
  readonly dataset_trainCount: (a: number) => number;
  readonly dataset_trainVertices: (a: number, b: number) => void;
  readonly dataset_trainColors: (a: number, b: number) => void;
  readonly dataset_trainLineNumbers: (a: number, b: number) => void;
  readonly dataset_trainSides: (a: number, b: number) => void;
  readonly dataset_trainExtents: (a: number, b: number) => void;
  readonly __wbg_view_free: (a: number) => void;
  readonly view_new: (a: number, b: number, c: number) => number;
  readonly view_resize: (a: number, b: number, c: number) => void;
  readonly view_scaling: (a: number) => number;
  readonly view_calculateViewProjection: (a: number, b: number) => void;
  readonly view_scroll: (a: number, b: number, c: number) => void;
  readonly view_zoom: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
        