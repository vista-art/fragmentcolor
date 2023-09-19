from fractions import Fraction

import numpy as np
import plrender
import pupil_labs.video as av

context = plrender.Context()


world_times = np.fromfile("world.time", dtype=np.int64)
world_container = av.open("world.mp4")
world_frames = world_container.decode(video=0)

eye_times = np.fromfile("eye.time", dtype=np.int64)
eye_container = av.open("eye.mp4")
eye_frames = eye_container.decode(video=0)

world_times_synced = (world_times - world_times[0]) / 1e9
eye_times_synced = (eye_times - world_times[0]) / 1e9


video_output_filename = "/tmp/foo.mp4"

output_container = av.open(video_output_filename, "w")
output_video_stream = output_container.add_stream("h264")
output_audio_stream = output_container.add_stream("aac")
output_video_stream.bit_rate = 1000000
output_video_stream.time_base = Fraction(1, 90000)
output_video_stream.height = 1200
output_video_stream.width = 800


def process_frame(timestamp: float, image_rgb: np.ndarray):
    height, width = image_rgb.shape
    frame = av.VideoFrame(width, height, "yuv420p")

    assert output_video_stream.time_base is not None
    frame.pts = int(timestamp / output_video_stream.time_base)

    frame = frame.from_ndarray(format="rgb24")
    packets = output_video_stream.encode(frame)
    output_container.mux(packets)


callback = context.add_callback_target(process_frame)
window = context.add_window_target(
    title="My Window",
    size=(600, 400),
)
file = context.add_file_target(
    filename=video_output_filename,
    height=1000,
    width=800,
    bitrate=1000000,
)
canvas = context.add_canvas_target(
    selector="#canvas",
)
