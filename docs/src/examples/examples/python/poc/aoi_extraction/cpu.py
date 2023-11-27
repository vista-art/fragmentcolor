import PIL.Image


def decompose(composed: PIL.Image.Image):
    composed_rgba_array = composed.getdata()
    w, h = composed.width, composed.height
    for aoi in range(32):
        print("aoi", aoi)
        aoi_png = PIL.Image.new("L", (w, h))
        aoi_png_data = list(aoi_png.getdata())
        color_channel = aoi // 8  # 0,1,2,3 for r, g, b, a
        channel_bitmask = 1 << aoi % 8  # eg 0x00000100 for channel 3
        for x in range(w):
            for y in range(h):
                composed_rgba = composed_rgba_array[y * w + x]
                color_value = composed_rgba[color_channel]
                if channel_bitmask & color_value:
                    aoi_png_data[y * w + x] = 255
        aoi_png.putdata(aoi_png_data)
        aoi_png.save(f"out{aoi}.png")


decompose(PIL.Image.open("composed.png"))
