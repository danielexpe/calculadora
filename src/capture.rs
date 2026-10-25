use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use thiserror::Error;
use tiny_skia::Pixmap;

use crate::cli::Cli;
use crate::core::Calculator;
use crate::i18n::SoundStatus;
use crate::render::fb::{capture_frame, straight_rgba};
use crate::render::layout::{CANVAS_H, CANVAS_W};
use crate::render::scene::{calculator_scene, CalculatorView};
use crate::render::text::TextEngine;
use crate::theme::Theme;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("no capture path was provided")]
    MissingPath,
    #[error("cannot load embedded fonts: {0}")]
    Fonts(&'static str),
    #[error("cannot create {path}: {source}")]
    Create {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot encode PNG {path}: {source}")]
    Encode {
        path: PathBuf,
        source: image::ImageError,
    },
    #[error("frame geometry {width}x{height} is not {CANVAS_W}x{CANVAS_H}")]
    Geometry { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureInfo {
    pub width: u32,
    pub height: u32,
    pub bytes: usize,
}

pub fn render_capture(cli: &Cli) -> Result<Pixmap, CaptureError> {
    let engine = TextEngine::new().map_err(CaptureError::Fonts)?;
    let theme = Theme::new(cli.effective_color_from_env(), 32);
    let calculator = Calculator::new();
    let locale = cli.lang.number_locale();
    let sound = if cli.no_audio {
        SoundStatus::Off
    } else {
        SoundStatus::On
    };

    let view = CalculatorView {
        lang: cli.lang,
        theme: &theme,
        display: calculator.display(locale),
        expression: calculator.expression(locale),
        history: calculator.history(),
        selection: None,
        sound,
        pressed: None,
    };

    let pixmap = capture_frame(&calculator_scene(&view), &engine);
    if pixmap.width() != CANVAS_W || pixmap.height() != CANVAS_H {
        return Err(CaptureError::Geometry {
            width: pixmap.width(),
            height: pixmap.height(),
        });
    }
    Ok(pixmap)
}

pub fn write_frame(cli: &Cli) -> Result<CaptureInfo, CaptureError> {
    let path = cli.capture_frame.clone().ok_or(CaptureError::MissingPath)?;
    let pixmap = render_capture(cli)?;
    let width = pixmap.width();
    let height = pixmap.height();
    let rgba = straight_rgba(&pixmap);
    let buffer = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or(CaptureError::Geometry { width, height })?;

    let file = File::create(&path).map_err(|source| CaptureError::Create {
        path: path.clone(),
        source,
    })?;
    let mut writer = BufWriter::new(file);
    buffer
        .write_to(&mut writer, image::ImageFormat::Png)
        .map_err(|source| CaptureError::Encode {
            path: path.clone(),
            source,
        })?;

    Ok(CaptureInfo {
        width,
        height,
        bytes: std::fs::metadata(&path)
            .map(|meta| meta.len() as usize)
            .unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    fn cli(arguments: &[&str]) -> Cli {
        Cli::try_parse_from(arguments).unwrap()
    }

    #[test]
    fn renders_canvas_sized_frame() {
        let pixmap = render_capture(&cli(&["calculadora"])).unwrap();
        assert_eq!((pixmap.width(), pixmap.height()), (CANVAS_W, CANVAS_H));
    }

    #[test]
    fn capture_is_deterministic_and_non_empty() {
        let first = render_capture(&cli(&["calculadora"])).unwrap();
        let second = render_capture(&cli(&["calculadora"])).unwrap();
        assert_eq!(first.data(), second.data());
        let unique = first
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<std::collections::HashSet<_>>();
        assert!(unique.len() > 10);
    }

    #[test]
    fn language_changes_capture() {
        let pt = render_capture(&cli(&["calculadora", "--lang", "pt"])).unwrap();
        let en = render_capture(&cli(&["calculadora", "--lang", "en"])).unwrap();
        assert_ne!(pt.data(), en.data());
    }

    #[test]
    fn mono_capture_differs_from_truecolor() {
        let color = render_capture(&cli(&["calculadora", "--color", "truecolor"])).unwrap();
        let mono = render_capture(&cli(&["calculadora", "--color", "never"])).unwrap();
        assert_ne!(color.data(), mono.data());
    }

    #[test]
    fn writes_valid_png_file() {
        let mut path = std::env::temp_dir();
        path.push(format!("calculadora-capture-{}.png", std::process::id()));
        let info = write_frame(&cli(&[
            "calculadora",
            "--capture-frame",
            path.to_str().unwrap(),
        ]))
        .unwrap();
        assert_eq!((info.width, info.height), (CANVAS_W, CANVAS_H));
        let decoded = image::open(&path).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (CANVAS_W, CANVAS_H));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_path_is_reported() {
        assert!(matches!(
            write_frame(&cli(&["calculadora"])),
            Err(CaptureError::MissingPath)
        ));
    }
}
