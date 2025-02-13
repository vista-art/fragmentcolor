[
  // depth pre-pass and population of structure buffer
  {
      name: depth_prepass_structure,
      type: scene,
      outputs: [ structure_buffer, depth_stencil_buffer ],
      clear: [ depth, color ],
      render_state: opaque,
      queues: main_scene,
      context: depth_and_structure
  },

  {
      name: shadow_map,
      type: cascaded_shadow_map,
      outputs: [ shadow_map ],
      clear: [ depth ],
      queues: main_scene,
      render_state: opaque_shadowmap,
      context: cascaded_shadow_map
  },

  {
      name: ssao_buffer,
      type: screen_space,
      clear: [
          color,
          depth
      ],      
      shader: ssao,
      render_state: fullscreen_noblend,
      inputs: [
          structure_buffer,
          ssao_rotations
      ],
      outputs: [ occlussion_buffer ]
  },

  {
      name: ssao_buffer_blur,
      type: screen_space,
      clear: [ color, depth ],
      shader: ssao_blur,
      render_state: fullscreen_noblend,
      inputs: [ structure_buffer, occlussion_buffer ],
      outputs: [ occlussion_buffer_blurred ]
  },

  {
      name: forward_light_culling,
      type: compute,
      shader: light_culling,
      workgroupsx: "(screen_width + (screen_width % tile_size)) / tile_size",
      workgroupsy: "(screen_height + (screen_height % tile_size)) / tile_size",
      write_buffers: [ visible_light_indices_buffer ],
      inputs: [ depth_stencil_buffer ]
  },

  {
      name: forward_light_culling_debug,
      type: screen_space,
      clear: [ color ],
      render_state: fullscreen_noblend,
      shader: light_culling_debug,
      outputs: [ debug_color ] 
   },

  {
      name: opaque_geometry,
      type: scene,
      clear: [ color, depth ],
      render_state: opaque,
      inputs: [ shadow_map, occlussion_buffer_blurred ],
      outputs: default_framebuffer,
      queues: [ scene_opaque, scene_alpha_tested ]
  },

  {
      name: hdr_light_accum,
      type: scene,
      clear: [ color ],
      render_state: opaque_nodepthwrite,
      inputs: [ occlussion_buffer_blurred, shadow_map ],
      outputs: [ hdr, hdr_brightness, depth_stencil_buffer, main_color ],        
      queues: main_scene
  },

  {
      name: bloom,
      type: blur,
      repeats: 10,
      render_state: fullscreen_noblend,
      inputs: [ hdr_brightness, hdr_blur ],
      outputs: [ hdr_brightness, hdr_blur ],
      shader: blur_gaussian
  },      

  {
      name: hdr_tone_mapping,
      type: screen_space,
      clear: [ color ],
      render_state: fullscreen_noblend,
      inputs: [ hdr, hdr_brightness ],
      outputs: [ base ],
      shader: tone_mapping        
  },    

  {
      name: skybox,
      type: screen_space,
      render_state: fullscreen_depthtest,
      inputs: [ skybox ],        
      outputs: [ base, depth_stencil_buffer ],
      shader: skybox    
  },      

  {
      name: fxaa_filter,
      type: screen_space,
      clear: [ color ],
      inputs: [ base ],
      outputs: [ base_2 ],        
      render_state: fullscreen_noblend,
      shader: filter_fxaa
  },

  {
      name: debug_draw,
      type: scene,
      outputs: [ base_2, depth_stencil_buffer ],
      render_state: debug_depth_blend,
      queues: debug
  },           

  {
      name: debug_draw_no_depth,
      type: scene,
      outputs: [ base_2, depth_stencil_buffer ],
      render_state: debug_no_depth_blend,
      queues: debug_nodepth
  },  
  
  {
      name: blit_main_color,
      type: blit,
      clear: [ color, depth ],
      inputs: [ base_2 ],
      outputs: default_framebuffer,
      render_state: opaque
  },

  {
      name: imgui,
      type: overlay,
      outputs: default_framebuffer,
      render_state: overlay,
      queues: imgui
  },

  {
      name: game_ui,
      type: overlay,
      outputs: default_framebuffer,
      render_state: overlay,
      queues: [ game_ui ]
  }
]
