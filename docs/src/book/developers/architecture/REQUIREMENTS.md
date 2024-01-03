# Requirements and Use Cases <!-- omit in toc -->

This document lists the **Requirements** for this project and the **Use Cases** for each of them. It serves as **general guideline** for the [Design Decisions](./decisions/README.md) of the library.

**FragmentColor** aims to support the following Requirements:

- [Multiple Scenes and Rendering Targets](#multiple-scenes-and-rendering-targets)
- [User-defined Scene Graphs](#user-defined-scene-graphs)
- [Support Custom Shaders](#support-custom-shaders)
- [Camera Lens Correction](#camera-lens-correction)
- [Offscreen Rendering](#offscreen-rendering)
- [Lower-Priority](#lower-priority)
  - [Support 3D Data](#support-3d-data)

## Multiple Scenes and Rendering Targets

### Use Cases <!-- omit in toc -->

- World Video (Scene 1) and Reference Image (Scene 2)
- Render each Scene into a different Target (ex. two web canvases)
- Combine multiple Scenes into one Target (ex. one video file)

## User-defined Scene Graphs

### Use Cases <!-- omit in toc -->

- Display multiple fixation points, with connecting lines and text
- Render multiple gaze circles with one parent position
- Display two eye videos on top of the world video
- Watermark, debug text, and other overlays
- Combinations of the above cases

## Support Custom Shaders

### Use Cases <!-- omit in toc -->

- Quick prototyping
- Copy & Paste ShaderToy code to any Scene object
- ShaderToy Interoperability / Drop-in replacement

## Camera Lens Correction

### Use Cases <!-- omit in toc -->

- Undistort the video before displaying it (equivalent to `cv2.undistort()`)
- Undistort gaze position without distorting the circle shape

## Offscreen Rendering

### Use Cases <!-- omit in toc -->

- Save a video and display the preview in a window at the same time
- Target composition (ex. merge two scenes into one video frame)

## Lower-Priority

### Support 3D Data

### Use Cases <!-- omit in toc -->

- Render a Point cloud
- Gaze point in 3D space
- AR overlay
- NeRF
