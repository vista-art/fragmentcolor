// Example function to get the (x, y) position based on the video's current time
const positionForTime = (time) => {
  return {
    x: Math.sin(time) * 0.5 + 0.5,
    y: Math.cos(time) * 0.5 + 0.5,
  };
};

// Params from Anna's gallery video
const undistortParams = () => {
  return {
    camera_matrix: [
      [765.469082753157, 0.0, 567.0954541838603],
      [0.0, 765.2673080778297, 545.3177094076727],
      [0.0, 0.0, 1.0],
    ],
    distortion_coefficients: [
      [
        -0.12591202244566754, 0.101181790269097, 0.0006819575150849819,
        -0.0004815612057156913, 0.018952897627046664, 0.20535990250436098,
        0.007459782333401985, 0.06701927729864841,
      ],
    ],
  };
};
