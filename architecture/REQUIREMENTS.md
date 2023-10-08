# Requirements and Use Cases <!-- omit in toc -->

The library aims to support the following use cases:

- [Arbitrary User-defined Scene Graphs](#arbitrary-user-defined-scene-graphs)
- [Handle Multiple Scenes Simultaneously](#handle-multiple-scenes-simultaneously)
- [Support for Custom Shaders](#support-for-custom-shaders)
- [Camera Lens Correction](#camera-lens-correction)
- [Offscreen Rendering](#offscreen-rendering)
- [Lower-Priority](#lower-priority)
  - [Support 3D Data](#support-3d-data)

## Arbitrary User-defined Scene Graphs

- **Use Cases:**
  - Display multiple fixation points, with connecting lines and text
  - Render multiple gaze circles with one parent position
  - Display two eye videos on top of the world video
  - Watermark, debug text, and other overlays
  - Combinations of the above cases

## Handle Multiple Scenes Simultaneously

- **Use Cases:**
  - World Video (scene 1) and Reference Image (scene 2)

## Support for Custom Shaders

- **Use Cases:**
  - Quick prototyping
  - Copy & Paste ShaderToy code to any Scene object
  - ShaderToy Interoperability / Drop-in replacement

## Camera Lens Correction

- **Use Cases:**
  - Undistort the video before displaying it (equivalent to `cv2.undistort()`)
  - Undistort gaze position without distorting the circle shape

## Offscreen Rendering

- **Use Cases:**
  - Save a video and display the preview in a window at the same time
  - Target composition (ex. merge two scenes into one video frame)

## Lower-Priority

### Support 3D Data

- **Use Cases:**
  - Render a Point cloud
  - Gaze point in 3D space
  - AR overlay
  - NeRF
