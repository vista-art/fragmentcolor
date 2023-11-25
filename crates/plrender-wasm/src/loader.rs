use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{GpuDevice, HtmlImageElement, ImageBitmap};

// Import other necessary modules from web-sys as needed

#[wasm_bindgen]
pub async fn web_gpu_texture_from_image_bitmap_or_canvas(
    gpu_device: GpuDevice,
    source: ImageBitmap,
) -> Result<GpuTexture, JsValue> {
    // Create the texture descriptor
    let texture_descriptor = GpuTextureDescriptor::new(
        source.width(),
        source.height(),
        // Specify other necessary parameters like format and usage
    );

    let texture = gpu_device.create_texture(&texture_descriptor);

    // Copy the image bitmap to the texture
    gpu_device.queue().copy_external_image_to_texture(
        &CopyExternalImageToTextureSource::ImageBitmap(&source),
        &texture,
        &texture_descriptor.size(),
    );

    Ok(texture)
}

#[wasm_bindgen]
pub async fn web_gpu_texture_from_image_url(
    gpu_device: GpuDevice,
    url: String,
) -> Result<GpuTexture, JsValue> {
    // Fetch the image and create an ImageBitmap
    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_str(&url)).await?;
    let blob = JsFuture::from(response.blob()?).await?;
    let img_bitmap = JsFuture::from(window.create_image_bitmap_with_blob(&blob)).await?;

    web_gpu_texture_from_image_bitmap_or_canvas(gpu_device, img_bitmap.dyn_into()?)
}

#[wasm_bindgen]
pub async fn web_gpu_texture_from_image_element(
    gpu_device: GpuDevice,
    img_element: HtmlImageElement,
) -> Result<GpuTexture, JsValue> {
    if img_element.complete() {
        let img_bitmap = window
            .create_image_bitmap_with_html_image_element(&img_element)
            .await?;
        web_gpu_texture_from_image_bitmap_or_canvas(gpu_device, img_bitmap.dyn_into()?)
    } else {
        // Handle the case where the image is not loaded yet
        // This part is trickier in Rust compared to JS as we have to set up event listeners
        // You might need to use `Closure` to create event listeners and then convert them to Promises
    }
}
