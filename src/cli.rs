use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::i18n::Lang;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[default]
    Fb,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    #[default]
    Auto,
    Truecolor,
    #[value(name = "256")]
    Color256,
    #[value(name = "16")]
    Color16,
    Never,
}

impl ColorMode {
    pub const fn effective(self, no_color: bool) -> Self {
        if no_color {
            Self::Never
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "calculadora",
    version,
    about = "Calculadora decimal para console Linux via framebuffer"
)]
pub struct Cli {
    /// Initial UI language.
    #[arg(long, value_parser = parse_lang, default_value = "pt")]
    pub lang: Lang,

    /// Video backend. The v1 supports only fb.
    #[arg(long, value_enum, default_value = "fb")]
    pub backend: Backend,

    /// Framebuffer device path.
    #[arg(long, value_name = "PATH", default_value = "/dev/fb0")]
    pub fb_device: PathBuf,

    /// Color mode. NO_COLOR always wins.
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    /// Disable animations.
    #[arg(long)]
    pub no_animations: bool,

    /// Start muted.
    #[arg(long)]
    pub no_audio: bool,

    /// Render a deterministic PNG and exit without using fb, tty or audio.
    #[arg(long, value_name = "PATH")]
    pub capture_frame: Option<PathBuf>,
}

impl Cli {
    pub fn is_capture(&self) -> bool {
        self.capture_frame.is_some()
    }

    pub fn effective_color(&self, no_color_env: bool) -> ColorMode {
        self.color.effective(no_color_env)
    }

    pub fn effective_color_from_env(&self) -> ColorMode {
        self.effective_color(no_color_environment())
    }
}

pub fn no_color_environment() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

fn parse_lang(value: &str) -> Result<Lang, String> {
    value.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(arguments)
    }

    #[test]
    fn defaults_are_conservative() {
        let cli = parse(&["calculadora"]).unwrap();
        assert_eq!(cli.lang, Lang::Pt);
        assert_eq!(cli.backend, Backend::Fb);
        assert_eq!(cli.fb_device, PathBuf::from("/dev/fb0"));
        assert_eq!(cli.color, ColorMode::Auto);
        assert!(!cli.no_animations);
        assert!(!cli.no_audio);
        assert!(!cli.is_capture());
    }

    #[test]
    fn parses_all_flags() {
        let cli = parse(&[
            "calculadora",
            "--lang",
            "en",
            "--backend",
            "fb",
            "--fb-device",
            "/dev/fb1",
            "--color",
            "256",
            "--no-animations",
            "--no-audio",
            "--capture-frame",
            "/tmp/calculadora.png",
        ])
        .unwrap();

        assert_eq!(cli.lang, Lang::En);
        assert_eq!(cli.backend, Backend::Fb);
        assert_eq!(cli.fb_device, PathBuf::from("/dev/fb1"));
        assert_eq!(cli.color, ColorMode::Color256);
        assert!(cli.no_animations);
        assert!(cli.no_audio);
        assert_eq!(
            cli.capture_frame,
            Some(PathBuf::from("/tmp/calculadora.png"))
        );
        assert!(cli.is_capture());
    }

    #[test]
    fn parses_all_color_names() {
        for (name, expected) in [
            ("auto", ColorMode::Auto),
            ("truecolor", ColorMode::Truecolor),
            ("256", ColorMode::Color256),
            ("16", ColorMode::Color16),
            ("never", ColorMode::Never),
        ] {
            let cli = parse(&["calculadora", "--color", name]).unwrap();
            assert_eq!(cli.color, expected);
        }
    }

    #[test]
    fn no_color_environment_wins() {
        let cli = parse(&["calculadora", "--color", "truecolor"]).unwrap();
        assert_eq!(cli.effective_color(true), ColorMode::Never);
        assert_eq!(cli.effective_color(false), ColorMode::Truecolor);
    }

    #[test]
    fn rejects_tui_and_auto_backends() {
        assert!(parse(&["calculadora", "--backend", "tui"]).is_err());
        assert!(parse(&["calculadora", "--backend", "auto"]).is_err());
    }

    #[test]
    fn rejects_invalid_language() {
        assert!(parse(&["calculadora", "--lang", "fr"]).is_err());
    }

    #[test]
    fn rejects_removed_seed_flag() {
        assert!(parse(&["calculadora", "--seed", "42"]).is_err());
    }
}
