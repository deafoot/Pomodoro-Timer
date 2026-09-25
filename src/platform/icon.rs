use windows::core::BOOL;
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

#[derive(Clone, Copy)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

fn blend(dst: Rgba, src: Rgba) -> Rgba {
    let a = src.a + dst.a * (1.0 - src.a);
    if a <= 0.0001 {
        return Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    }
    Rgba {
        r: (src.r * src.a + dst.r * dst.a * (1.0 - src.a)) / a,
        g: (src.g * src.a + dst.g * dst.a * (1.0 - src.a)) / a,
        b: (src.b * src.a + dst.b * dst.a * (1.0 - src.a)) / a,
        a,
    }
}

fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

/// 用像素手绘一个番茄图标（无需外部图片资源）
pub fn tomato_icon(size: i32) -> HICON {
    let size = size.max(16);
    unsafe {
        let screen = GetDC(None);
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(color) = CreateDIBSection(Some(screen), &info, DIB_RGB_COLORS, &mut bits, None, 0) else {
            ReleaseDC(None, screen);
            return HICON(std::ptr::null_mut());
        };
        let s = size as f32;
        let pixels = bits as *mut u32;
        for y in 0..size {
            for x in 0..size {
                let fx = (x as f32 + 0.5) / s;
                let fy = (y as f32 + 0.5) / s;
                let mut pixel = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

                // 果身
                let cx = 0.5;
                let cy = 0.58;
                let body_r = 0.40;
                let d = ((fx - cx).powi(2) + ((fy - cy) * 1.08).powi(2)).sqrt();
                let body_alpha = coverage((d - body_r) / 0.02);
                if body_alpha > 0.0 {
                    let shade = 1.0 - ((fx - 0.34).powi(2) + (fy - 0.30).powi(2)).sqrt() * 0.55;
                    let shade = shade.clamp(0.62, 1.12);
                    pixel = blend(
                        pixel,
                        Rgba { r: 0.85 * shade, g: 0.27 * shade, b: 0.21 * shade, a: body_alpha },
                    );
                }

                // 高光
                let hd = ((fx - 0.37).powi(2) + (fy - 0.45).powi(2)).sqrt();
                let hl = coverage((hd - 0.10) / 0.03) * 0.40;
                if hl > 0.0 {
                    pixel = blend(pixel, Rgba { r: 1.0, g: 0.85, b: 0.82, a: hl });
                }

                // 叶片（三片）
                for (lx, ly, lr) in [(0.30_f32, 0.22_f32, 0.15_f32), (0.70, 0.22, 0.15), (0.50, 0.14, 0.17)] {
                    let ld = ((fx - lx).powi(2) + ((fy - ly) * 1.6).powi(2)).sqrt();
                    let la = coverage((ld - lr) / 0.02);
                    if la > 0.0 {
                        let shade = 1.0 - (fx - 0.5).abs() * 0.4;
                        pixel = blend(pixel, Rgba { r: 0.24 * shade, g: 0.55 * shade, b: 0.30 * shade, a: la });
                    }
                }

                // 果蒂
                let sd = ((fx - 0.5).powi(2) + (fy - 0.09).powi(2)).sqrt();
                let sa = coverage((sd - 0.045) / 0.02);
                if sa > 0.0 {
                    pixel = blend(pixel, Rgba { r: 0.30, g: 0.36, b: 0.24, a: sa });
                }

                let a = (pixel.a.clamp(0.0, 1.0) * 255.0).round() as u32;
                let r = (pixel.r.clamp(0.0, 1.0) * 255.0).round() as u32;
                let g = (pixel.g.clamp(0.0, 1.0) * 255.0).round() as u32;
                let b = (pixel.b.clamp(0.0, 1.0) * 255.0).round() as u32;
                *pixels.add((y * size + x) as usize) = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }

        let mask = CreateBitmap(size, size, 1, 1, None);
        let icon_info = ICONINFO {
            fIcon: BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&icon_info).unwrap_or(HICON(std::ptr::null_mut()));
        let _ = DeleteObject(color.into());
        let _ = DeleteObject(mask.into());
        ReleaseDC(None, screen);
        icon
    }
}


