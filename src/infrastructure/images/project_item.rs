use image::{ImageBuffer, Rgb, RgbImage};

pub fn generate_project_item_image() -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(100, 100);

    let bg          = Rgb([15u8,  15,  15]);
    let folder_dark = Rgb([180u8, 130, 30]);
    let folder_body = Rgb([230u8, 170, 40]);
    let line_color  = Rgb([50u8,  50,  50]);

    for p in img.pixels_mut() { *p = bg; }

    // Folder tab (top-left bump)
    for y in 28u32..=38 { for x in 12u32..=42 { img.put_pixel(x, y, folder_dark); } }
    // Rounded tab edge
    for x in 42u32..=52 { img.put_pixel(x, 38, folder_dark); }

    // Folder body
    for y in 38u32..=78 { for x in 12u32..=88 { img.put_pixel(x, y, folder_body); } }

    // Darker top edge of folder body
    for x in 12u32..=88 { img.put_pixel(x, 38, folder_dark); }
    for x in 12u32..=88 { img.put_pixel(x, 39, folder_dark); }

    // Lines inside folder to suggest files
    for x in 22u32..=78 { img.put_pixel(x, 50, line_color); }
    for x in 22u32..=78 { img.put_pixel(x, 58, line_color); }
    for x in 22u32..=60 { img.put_pixel(x, 66, line_color); }

    img
}
