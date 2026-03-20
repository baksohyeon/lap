use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, Rgb, Rgba};
use rsraw::{ImageFormat as RsRawImageFormat, RawImage, ThumbFormat, ThumbnailImage, BIT_DEPTH_8};
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RawFormat {
    NefLike,
    Dng,
    TiffLike,
    GenericRaw,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreviewStrategy {
    // Prefer the camera-embedded preview. This matches mainstream RAW viewers:
    // fast to display, preserves the in-camera rendering, and is usually the
    // most stable path for common RAW formats when the embedded JPEG is valid.
    EmbeddedFirst,
    // Use the embedded preview only when it is already large enough; otherwise
    // fall back to a rendered preview from the RAW data. This is safer for
    // formats that often expose a small or inconsistent embedded preview.
    ProcessedWhenEmbeddedTooSmall,
}

#[derive(Clone, Copy, Debug)]
struct RawProfile {
    preview_strategy: PreviewStrategy,
    prefer_jpeg_thumbnail: bool,
}

fn resolve_raw_format(file_path: &str) -> RawFormat {
    match file_extension(file_path).as_deref() {
        Some("nef") | Some("nrw") => RawFormat::NefLike,
        Some("dng") => RawFormat::Dng,
        Some("tif") | Some("tiff") => RawFormat::TiffLike,
        Some(ext) if crate::t_common::RAW_IMGS.contains(&ext) => RawFormat::GenericRaw,
        _ => RawFormat::Unknown,
    }
}

fn resolve_raw_profile(file_path: &str) -> RawProfile {
    match resolve_raw_format(file_path) {
        // Default policy for formats listed in RAW_IMGS:
        // prefer the embedded preview first. This keeps preview loading fast
        // and avoids unnecessarily decoding full RAW data for formats that
        // usually carry a good camera-generated JPEG preview. This also covers
        // older/less common RAW families in the sample set (MRW, 3FR, MOS,
        // X3F, KDC/DCR, ERF, MEF, MDC, RAW) unless we later observe a format-
        // specific preview failure that justifies a dedicated policy.
        RawFormat::GenericRaw => RawProfile {
            preview_strategy: PreviewStrategy::EmbeddedFirst,
            prefer_jpeg_thumbnail: true,
        },
        // Nikon RAWs, DNG, and TIFF-like RAW containers are treated more
        // conservatively: if the embedded preview is too small, render from RAW.
        // This reflects the sample failures we saw in the app and aligns with
        // how mainstream RAW tools fall back to decoded previews when needed.
        RawFormat::NefLike | RawFormat::Dng | RawFormat::TiffLike => RawProfile {
            preview_strategy: PreviewStrategy::ProcessedWhenEmbeddedTooSmall,
            prefer_jpeg_thumbnail: true,
        },
        // Unknown should be rare here, but keep the safer fallback policy.
        RawFormat::Unknown => RawProfile {
            preview_strategy: PreviewStrategy::ProcessedWhenEmbeddedTooSmall,
            prefer_jpeg_thumbnail: true,
        },
    }
}

fn file_extension(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

fn open_raw_image(file_path: &str) -> Result<RawImage, String> {
    let bytes = fs::read(file_path).map_err(|e| format!("Failed to read RAW file: {}", e))?;
    RawImage::open(&bytes).map_err(|e| format!("Failed to open RAW file with rsraw: {}", e))
}

fn encode_as_jpeg(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    DynamicImage::ImageRgb8(img.to_rgb8())
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode image as JPEG: {}", e))?;
    Ok(buf)
}

fn orient_image(img: DynamicImage, orientation: i32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn thumbnail_max_edge(thumb: &ThumbnailImage) -> u32 {
    thumb.width.max(thumb.height)
}

fn is_preview_thumb_large_enough(thumb: &ThumbnailImage, raw_width: u32, raw_height: u32) -> bool {
    let thumb_edge = thumbnail_max_edge(thumb);
    let raw_edge = raw_width.max(raw_height);
    thumb_edge >= 2048 || thumb_edge.saturating_mul(2) >= raw_edge
}

fn decode_thumb_image(thumb: &ThumbnailImage) -> Result<DynamicImage, String> {
    match thumb.format {
        ThumbFormat::Jpeg => image::load_from_memory(&thumb.data)
            .map_err(|e| format!("Failed to decode JPEG RAW thumbnail: {}", e)),
        ThumbFormat::Bitmap => match thumb.colors {
            1 => {
                let image = ImageBuffer::<Luma<u8>, _>::from_raw(
                    thumb.width,
                    thumb.height,
                    thumb.data.clone(),
                )
                .ok_or("Failed to create grayscale RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageLuma8(image))
            }
            3 => {
                let image = ImageBuffer::<Rgb<u8>, _>::from_raw(
                    thumb.width,
                    thumb.height,
                    thumb.data.clone(),
                )
                .ok_or("Failed to create RGB RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageRgb8(image))
            }
            4 => {
                let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
                    thumb.width,
                    thumb.height,
                    thumb.data.clone(),
                )
                .ok_or("Failed to create RGBA RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageRgba8(image))
            }
            _ => Err(format!(
                "Unsupported RAW thumbnail bitmap color count: {}",
                thumb.colors
            )),
        },
        ThumbFormat::Bitmap16 => match thumb.colors {
            1 => {
                let data = thumb
                    .data
                    .chunks_exact(2)
                    .map(|chunk| chunk[1])
                    .collect::<Vec<u8>>();
                let image = ImageBuffer::<Luma<u8>, _>::from_raw(thumb.width, thumb.height, data)
                    .ok_or("Failed to create 16-bit grayscale RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageLuma8(image))
            }
            3 => {
                let data = thumb
                    .data
                    .chunks_exact(2)
                    .map(|chunk| chunk[1])
                    .collect::<Vec<u8>>();
                let image = ImageBuffer::<Rgb<u8>, _>::from_raw(thumb.width, thumb.height, data)
                    .ok_or("Failed to create 16-bit RGB RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageRgb8(image))
            }
            4 => {
                let data = thumb
                    .data
                    .chunks_exact(2)
                    .map(|chunk| chunk[1])
                    .collect::<Vec<u8>>();
                let image = ImageBuffer::<Rgba<u8>, _>::from_raw(thumb.width, thumb.height, data)
                    .ok_or("Failed to create 16-bit RGBA RAW thumbnail buffer")?;
                Ok(DynamicImage::ImageRgba8(image))
            }
            _ => Err(format!(
                "Unsupported 16-bit RAW thumbnail color count: {}",
                thumb.colors
            )),
        },
        _ => Err(format!(
            "Unsupported RAW thumbnail format: {:?}",
            thumb.format
        )),
    }
}

fn select_preview_thumb(thumbs: &[ThumbnailImage]) -> Option<&ThumbnailImage> {
    thumbs
        .iter()
        .filter(|thumb| {
            matches!(
                thumb.format,
                ThumbFormat::Jpeg | ThumbFormat::Bitmap | ThumbFormat::Bitmap16
            )
        })
        .max_by_key(|thumb| thumbnail_max_edge(thumb))
}

fn select_thumbnail_thumb(
    thumbs: &[ThumbnailImage],
    thumbnail_size: u32,
    prefer_jpeg_thumbnail: bool,
) -> Option<&ThumbnailImage> {
    if prefer_jpeg_thumbnail {
        let jpeg_thumbs = thumbs
            .iter()
            .filter(|thumb| matches!(thumb.format, ThumbFormat::Jpeg))
            .collect::<Vec<_>>();

        if !jpeg_thumbs.is_empty() {
            let mut best_not_smaller: Option<&ThumbnailImage> = None;
            let mut best_smaller: Option<&ThumbnailImage> = None;

            for thumb in jpeg_thumbs {
                let edge = thumbnail_max_edge(thumb);
                if edge >= thumbnail_size {
                    match best_not_smaller {
                        Some(best) if edge >= thumbnail_max_edge(best) => {}
                        _ => best_not_smaller = Some(thumb),
                    }
                } else {
                    match best_smaller {
                        Some(best) if edge <= thumbnail_max_edge(best) => {}
                        _ => best_smaller = Some(thumb),
                    }
                }
            }

            return best_not_smaller.or(best_smaller);
        }
    }

    let mut best_not_smaller: Option<&ThumbnailImage> = None;
    let mut best_smaller: Option<&ThumbnailImage> = None;

    for thumb in thumbs.iter().filter(|thumb| {
        matches!(
            thumb.format,
            ThumbFormat::Jpeg | ThumbFormat::Bitmap | ThumbFormat::Bitmap16
        )
    }) {
        let edge = thumbnail_max_edge(thumb);
        if edge >= thumbnail_size {
            match best_not_smaller {
                Some(best) if edge >= thumbnail_max_edge(best) => {}
                _ => best_not_smaller = Some(thumb),
            }
        } else {
            match best_smaller {
                Some(best) if edge <= thumbnail_max_edge(best) => {}
                _ => best_smaller = Some(thumb),
            }
        }
    }

    best_not_smaller.or(best_smaller)
}

fn render_processed_preview(file_path: &str, max_edge: u32) -> Result<Vec<u8>, String> {
    let mut raw = open_raw_image(file_path)?;
    raw.unpack()
        .map_err(|e| format!("Failed to unpack RAW file: {}", e))?;
    let processed = raw
        .process::<BIT_DEPTH_8>()
        .map_err(|e| format!("Failed to process RAW preview: {}", e))?;

    let image = match processed.image_format() {
        RsRawImageFormat::Jpeg => image::load_from_memory(&processed.to_vec())
            .map_err(|e| format!("Failed to decode processed RAW JPEG preview: {}", e))?,
        RsRawImageFormat::Bitmap => {
            let width = processed.width();
            let height = processed.height();
            let colors = processed.colors();

            match colors {
                1 => {
                    let image =
                        ImageBuffer::<Luma<u8>, _>::from_raw(width, height, processed.to_vec())
                            .ok_or("Failed to create processed grayscale RAW image")?;
                    DynamicImage::ImageLuma8(image)
                }
                3 => {
                    let image =
                        ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, processed.to_vec())
                            .ok_or("Failed to create processed RGB RAW image")?;
                    DynamicImage::ImageRgb8(image)
                }
                4 => {
                    let image =
                        ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, processed.to_vec())
                            .ok_or("Failed to create processed RGBA RAW image")?;
                    DynamicImage::ImageRgba8(image)
                }
                _ => {
                    return Err(format!(
                        "Unsupported processed RAW channel count: {}",
                        colors
                    ));
                }
            }
        }
    };

    let image = if max_edge > 0 {
        image.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };

    encode_as_jpeg(&image)
}

fn render_oriented_processed_preview(file_path: &str, orientation: i32) -> Result<Vec<u8>, String> {
    let image = image::load_from_memory(&render_processed_preview(file_path, 4096)?)
        .map_err(|e| format!("Failed to decode processed RAW preview: {}", e))?;
    let image = orient_image(image, orientation);
    encode_as_jpeg(&image)
}

pub fn get_raw_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    let raw = open_raw_image(file_path)?;
    let width = raw.width();
    let height = raw.height();
    if width > 0 && height > 0 {
        Ok((width, height))
    } else {
        Err("rsraw resolved empty RAW dimensions".to_string())
    }
}

pub fn get_raw_preview_image(file_path: &str, orientation: i32) -> Result<Option<Vec<u8>>, String> {
    let profile = resolve_raw_profile(file_path);
    let mut raw = open_raw_image(file_path)?;
    let raw_width = raw.width();
    let raw_height = raw.height();

    if let Ok(thumbs) = raw.extract_thumbs() {
        if let Some(thumb) = select_preview_thumb(&thumbs) {
            let should_use_embedded = match profile.preview_strategy {
                PreviewStrategy::EmbeddedFirst => true,
                PreviewStrategy::ProcessedWhenEmbeddedTooSmall => {
                    is_preview_thumb_large_enough(thumb, raw_width, raw_height)
                }
            };

            if should_use_embedded {
                if let Ok(image) = decode_thumb_image(thumb) {
                    let image = orient_image(image, orientation);
                    return encode_as_jpeg(&image).map(Some);
                }
            }
        }

        if profile.preview_strategy == PreviewStrategy::ProcessedWhenEmbeddedTooSmall {
            if let Some(thumb) = select_preview_thumb(&thumbs) {
                if !is_preview_thumb_large_enough(thumb, raw_width, raw_height) {
                    return render_oriented_processed_preview(file_path, orientation).map(Some);
                }
            } else {
                return render_oriented_processed_preview(file_path, orientation).map(Some);
            }
        }
    }

    render_oriented_processed_preview(file_path, orientation).map(Some)
}

pub fn get_raw_thumbnail(
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    let profile = resolve_raw_profile(file_path);
    let mut raw = open_raw_image(file_path)?;
    if let Ok(thumbs) = raw.extract_thumbs() {
        if let Some(thumb) =
            select_thumbnail_thumb(&thumbs, thumbnail_size, profile.prefer_jpeg_thumbnail)
        {
            if let Ok(image) = decode_thumb_image(thumb) {
                let image = orient_image(image, orientation);
                let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
                return encode_as_jpeg(&thumbnail).map(Some);
            }
        }
    }

    let preview = render_processed_preview(file_path, thumbnail_size)?;
    let image = image::load_from_memory(&preview)
        .map_err(|e| format!("Failed to decode processed RAW thumbnail preview: {}", e))?;
    let image = orient_image(image, orientation);
    let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
    encode_as_jpeg(&thumbnail).map(Some)
}

pub fn is_tiff_path(file_path: &str) -> bool {
    resolve_raw_format(file_path) == RawFormat::TiffLike
}
