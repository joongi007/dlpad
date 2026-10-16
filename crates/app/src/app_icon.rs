//! Original geometric artwork. Deterministic RGBA and uncompressed multi-size ICO.
//! No dependency, font, downloaded artwork, or runtime file I/O.
pub const SIZES: [u32; 4] = [16, 32, 48, 256];

pub fn rgba(size: u32) -> Vec<u8> {
    assert!(SIZES.contains(&size));
    let mut out = vec![0; (size * size * 4) as usize];
    // Four by four coverage sampling; coordinates use a 256-unit design grid.
    for y in 0..size {
        for x in 0..size {
            let mut blue = 0u32;
            let mut white = 0u32;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = (x as f64 + (sx as f64 + 0.5) / 4.0) * 256.0 / size as f64;
                    let py = (y as f64 + (sy as f64 + 0.5) / 4.0) * 256.0 / size as f64;
                    let dx = (px - 128.0).abs() - 76.0;
                    let dy = (py - 128.0).abs() - 76.0;
                    let inside = dx.max(0.0).powi(2) + dy.max(0.0).powi(2) <= 32.0f64.powi(2)
                        && (20.0..=236.0).contains(&px)
                        && (20.0..=236.0).contains(&py);
                    if inside {
                        blue += 1;
                        let stem = (112.0..=144.0).contains(&px) && (62.0..=128.0).contains(&py);
                        let arrow =
                            (116.0..=166.0).contains(&py) && (px - 128.0).abs() <= 166.0 - py;
                        let tray = (76.0..=180.0).contains(&px) && (182.0..=198.0).contains(&py)
                            || ((76.0..=92.0).contains(&px) || (164.0..=180.0).contains(&px))
                                && (162.0..=198.0).contains(&py);
                        if stem || arrow || tray {
                            white += 1;
                        }
                    }
                }
            }
            let i = ((y * size + x) * 4) as usize;
            if blue != 0 {
                for (c, b) in [0u32, 111, 190].into_iter().enumerate() {
                    out[i + c] = ((b * (blue - white) + 255 * white) / blue) as u8;
                }
                out[i + 3] = ((blue * 255 + 8) / 16) as u8;
            }
        }
    }
    out
}

#[allow(dead_code)]
pub fn ico() -> Vec<u8> {
    let mut images = Vec::new();
    for size in SIZES {
        let pixels = rgba(size);
        let mut dib = Vec::new();
        for v in [40u32, size, size * 2] {
            dib.extend(v.to_le_bytes());
        }
        dib.extend(1u16.to_le_bytes());
        dib.extend(32u16.to_le_bytes());
        for v in [0u32, size * size * 4, 0, 0, 0, 0] {
            dib.extend(v.to_le_bytes());
        }
        for y in (0..size).rev() {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                dib.extend([pixels[i + 2], pixels[i + 1], pixels[i], pixels[i + 3]]);
            }
        }
        let stride = size.div_ceil(32) * 4;
        for y in (0..size).rev() {
            let mut mask = vec![0u8; stride as usize];
            for x in 0..size {
                if pixels[((y * size + x) * 4 + 3) as usize] == 0 {
                    mask[(x / 8) as usize] |= 0x80 >> (x % 8);
                }
            }
            dib.extend(mask);
        }
        images.push(dib);
    }
    let mut out = vec![0, 0, 1, 0, 4, 0];
    let mut offset = 6 + 16 * SIZES.len();
    for (size, img) in SIZES.into_iter().zip(&images) {
        out.extend([
            if size == 256 { 0 } else { size as u8 },
            if size == 256 { 0 } else { size as u8 },
            0,
            0,
        ]);
        out.extend(1u16.to_le_bytes());
        out.extend(32u16.to_le_bytes());
        out.extend((img.len() as u32).to_le_bytes());
        out.extend((offset as u32).to_le_bytes());
        offset += img.len();
    }
    for img in images {
        out.extend(img);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_artwork_matches_executable_resource_source() {
        assert_eq!(super::ico(), include_bytes!("../assets/dlpad.ico"));
    }
}
