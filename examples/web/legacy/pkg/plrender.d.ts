/* tslint:disable */
/* eslint-disable */
/**
*/
export class ControllerOptions {
  free(): void;
/**
*/
  _fixation?: FixationOptions;
/**
*/
  gaze?: GazeOptions;
}
/**
*/
export class FixationOptions {
  free(): void;
/**
*/
  alpha: number;
/**
*/
  border: number;
/**
*/
  color: string;
/**
*/
  radius: number;
}
/**
*/
export class GazeOptions {
  free(): void;
/**
*/
  alpha?: number;
/**
*/
  border?: number;
/**
*/
  color?: string;
/**
*/
  name?: string;
/**
*/
  radius?: number;
}
/**
*/
export class Options {
  free(): void;
/**
*/
  controllers?: ControllerOptions;
/**
*/
  window?: WindowOptions;
}
/**
*/
export class PLRender {
  free(): void;
/**
*/
  constructor();
/**
* @param {any} options
*/
  config(options: any): void;
/**
*/
  run(): void;
/**
* @param {string} controller
* @param {string} event
* @param {any[]} params
*/
  trigger(controller: string, event: string, params: any[]): void;
/**
* @returns {Resolution}
*/
  resolution(): Resolution;
}
/**
*/
export class Resolution {
  free(): void;
/**
*/
  height: number;
/**
*/
  width: number;
}
/**
*/
export class WindowOptions {
  free(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_windowoptions_free: (a: number) => void;
  readonly __wbg_options_free: (a: number) => void;
  readonly __wbg_get_options_window: (a: number) => number;
  readonly __wbg_set_options_window: (a: number, b: number) => void;
  readonly __wbg_get_options_controllers: (a: number) => number;
  readonly __wbg_set_options_controllers: (a: number, b: number) => void;
  readonly __wbg_plrender_free: (a: number) => void;
  readonly __wbg_resolution_free: (a: number) => void;
  readonly __wbg_get_resolution_width: (a: number) => number;
  readonly __wbg_set_resolution_width: (a: number, b: number) => void;
  readonly __wbg_get_resolution_height: (a: number) => number;
  readonly __wbg_set_resolution_height: (a: number, b: number) => void;
  readonly plrender_new: () => number;
  readonly plrender_config: (a: number, b: number) => void;
  readonly plrender_run: (a: number) => void;
  readonly plrender_trigger: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly plrender_resolution: (a: number) => number;
  readonly __wbg_controlleroptions_free: (a: number) => void;
  readonly __wbg_get_controlleroptions_gaze: (a: number) => number;
  readonly __wbg_set_controlleroptions_gaze: (a: number, b: number) => void;
  readonly __wbg_get_controlleroptions__fixation: (a: number) => number;
  readonly __wbg_set_controlleroptions__fixation: (a: number, b: number) => void;
  readonly __wbg_gazeoptions_free: (a: number) => void;
  readonly __wbg_get_gazeoptions_name: (a: number, b: number) => void;
  readonly __wbg_set_gazeoptions_name: (a: number, b: number, c: number) => void;
  readonly __wbg_get_gazeoptions_radius: (a: number, b: number) => void;
  readonly __wbg_set_gazeoptions_radius: (a: number, b: number, c: number) => void;
  readonly __wbg_get_gazeoptions_border: (a: number, b: number) => void;
  readonly __wbg_set_gazeoptions_border: (a: number, b: number, c: number) => void;
  readonly __wbg_get_gazeoptions_color: (a: number, b: number) => void;
  readonly __wbg_set_gazeoptions_color: (a: number, b: number, c: number) => void;
  readonly __wbg_get_gazeoptions_alpha: (a: number, b: number) => void;
  readonly __wbg_set_gazeoptions_alpha: (a: number, b: number, c: number) => void;
  readonly __wbg_fixationoptions_free: (a: number) => void;
  readonly __wbg_get_fixationoptions_radius: (a: number) => number;
  readonly __wbg_set_fixationoptions_radius: (a: number, b: number) => void;
  readonly __wbg_get_fixationoptions_border: (a: number) => number;
  readonly __wbg_set_fixationoptions_border: (a: number, b: number) => void;
  readonly __wbg_get_fixationoptions_color: (a: number, b: number) => void;
  readonly __wbg_set_fixationoptions_color: (a: number, b: number, c: number) => void;
  readonly __wbg_get_fixationoptions_alpha: (a: number) => number;
  readonly __wbg_set_fixationoptions_alpha: (a: number, b: number) => void;
  readonly wgpu_compute_pass_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_compute_pass_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_compute_pass_set_push_constant: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_compute_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_push_debug_group: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_pop_debug_group: (a: number) => void;
  readonly wgpu_compute_pass_write_timestamp: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_begin_pipeline_statistics_query: (a: number, b: number, c: number) => void;
  readonly wgpu_compute_pass_end_pipeline_statistics_query: (a: number) => void;
  readonly wgpu_compute_pass_dispatch_workgroups: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_compute_pass_dispatch_workgroups_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_bundle_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_render_bundle_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_vertex_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_draw: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_bundle_draw_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_bundle_draw_indexed_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_set_pipeline: (a: number, b: number) => void;
  readonly wgpu_render_pass_set_bind_group: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_vertex_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_push_constants: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_draw: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_draw_indexed: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_draw_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_draw_indexed_indirect: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_multi_draw_indirect: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_render_pass_multi_draw_indexed_indirect: (a: number, b: number, c: number, d: number) => void;
  readonly wgpu_render_pass_multi_draw_indirect_count: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_multi_draw_indexed_indirect_count: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wgpu_render_pass_set_blend_constant: (a: number, b: number) => void;
  readonly wgpu_render_pass_set_scissor_rect: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_pass_set_viewport: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wgpu_render_pass_set_stencil_reference: (a: number, b: number) => void;
  readonly wgpu_render_pass_insert_debug_marker: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_push_debug_group: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_pop_debug_group: (a: number) => void;
  readonly wgpu_render_pass_write_timestamp: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_begin_pipeline_statistics_query: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_end_pipeline_statistics_query: (a: number) => void;
  readonly wgpu_render_pass_execute_bundles: (a: number, b: number, c: number) => void;
  readonly wgpu_render_pass_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_set_index_buffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wgpu_render_bundle_pop_debug_group: (a: number) => void;
  readonly wgpu_render_bundle_insert_debug_marker: (a: number, b: number) => void;
  readonly wgpu_render_bundle_push_debug_group: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h03fbb671f6443f92: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5e54a70659327263: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7e7927a658461779: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h70724005cd510070: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
