use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use calculadora::app::App;
use calculadora::capture::render_frame;
use calculadora::core::Operator;
use calculadora::i18n::{Lang, SoundStatus};
use calculadora::input::Action;
use calculadora::render::fb::straight_rgba;
use calculadora::render::layout::{CANVAS_H, CANVAS_W};
use calculadora::render::text::TextEngine;
use calculadora::theme::Theme;

const STEP: Duration = Duration::from_millis(200);

fn snapshots_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
}

fn digits(actions: &mut Vec<Action>, text: &str) {
    for digit in text.chars() {
        actions.push(Action::Digit(digit));
    }
}

fn operation(actions: &mut Vec<Action>, operator: Operator) {
    actions.push(Action::Operator(operator));
}

fn play(app: &mut App, actions: &[Action]) {
    let mut now = Instant::now();
    for action in actions {
        app.handle_at(*action, now);
        now += STEP;
    }
    app.update(now);
}

fn play_keeping_last_flash(app: &mut App, actions: &[Action]) {
    let mut now = Instant::now();
    for action in actions {
        app.handle_at(*action, now);
        now += STEP;
    }
    app.update(now - STEP + Duration::from_millis(50));
}

fn history_app(lang: Lang, count: usize) -> App {
    let mut app = App::new(lang, false, true);
    let mut actions = Vec::new();
    for index in 1..=count {
        digits(&mut actions, &index.to_string());
        operation(&mut actions, Operator::Add);
        digits(&mut actions, "1");
        actions.push(Action::Equals);
    }
    play(&mut app, &actions);
    app
}

fn long_state(lang: Lang) -> App {
    let mut app = App::new(lang, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "123456789012");
    operation(&mut actions, Operator::Add);
    digits(&mut actions, "987654321098");
    actions.push(Action::Equals);
    play(&mut app, &actions);
    app
}

fn cases() -> Vec<(&'static str, App, Theme)> {
    let mut cases = Vec::new();

    cases.push((
        "01_initial_pt.png",
        App::new(Lang::Pt, false, true),
        Theme::truecolor(),
    ));
    cases.push((
        "02_initial_en.png",
        App::new(Lang::En, false, true),
        Theme::truecolor(),
    ));

    let mut pending = App::new(Lang::Pt, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "200");
    operation(&mut actions, Operator::Add);
    play(&mut pending, &actions);
    cases.push(("03_pending_200_plus.png", pending, Theme::truecolor()));

    let mut result = App::new(Lang::Pt, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "200");
    operation(&mut actions, Operator::Add);
    digits(&mut actions, "20");
    actions.push(Action::Equals);
    play(&mut result, &actions);
    cases.push(("04_result_220_history.png", result, Theme::truecolor()));

    let mut percent = App::new(Lang::Pt, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "200");
    operation(&mut actions, Operator::Add);
    digits(&mut actions, "10");
    actions.push(Action::Percent);
    play(&mut percent, &actions);
    cases.push(("05_percent_200_plus_10.png", percent, Theme::truecolor()));

    let mut error = App::new(Lang::Pt, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "1");
    operation(&mut actions, Operator::Div);
    digits(&mut actions, "0");
    actions.push(Action::Equals);
    play(&mut error, &actions);
    cases.push(("06_error_division_by_zero.png", error, Theme::truecolor()));

    cases.push((
        "07_history_ten.png",
        history_app(Lang::Pt, 12),
        Theme::truecolor(),
    ));

    let mut selected = history_app(Lang::Pt, 12);
    let now = Instant::now();
    selected.handle_at(Action::HistoryPrev, now);
    selected.handle_at(Action::HistoryPrev, now);
    cases.push(("08_history_selected.png", selected, Theme::truecolor()));

    let mut flash = App::new(Lang::Pt, false, true);
    let mut actions = Vec::new();
    digits(&mut actions, "2");
    operation(&mut actions, Operator::Add);
    digits(&mut actions, "3");
    actions.push(Action::Equals);
    play_keeping_last_flash(&mut flash, &actions);
    cases.push(("09_equals_flash.png", flash, Theme::truecolor()));

    cases.push((
        "10_palette_256.png",
        App::new(Lang::Pt, false, true),
        Theme::color256(),
    ));
    cases.push((
        "11_palette_16.png",
        App::new(Lang::Pt, false, true),
        Theme::color16(),
    ));
    cases.push((
        "12_mono.png",
        App::new(Lang::Pt, false, true),
        Theme::mono(),
    ));
    cases.push((
        "13_sound_off.png",
        App::new(Lang::Pt, true, true),
        Theme::truecolor(),
    ));

    let mut unavailable = App::new(Lang::Pt, false, true);
    unavailable.set_sound_status(SoundStatus::Unavailable);
    cases.push(("14_sound_unavailable.png", unavailable, Theme::truecolor()));

    cases.push(("15_long_pt.png", long_state(Lang::Pt), Theme::truecolor()));
    cases.push(("16_long_en.png", long_state(Lang::En), Theme::truecolor()));

    cases
}

#[test]
fn golden_frames_match() {
    let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();
    let engine = TextEngine::new().expect("embedded fonts must load");
    let directory = snapshots_dir();
    if update {
        std::fs::create_dir_all(&directory).expect("create snapshots directory");
    }

    let mut failures = Vec::new();
    for (name, app, theme) in cases() {
        let rendered = straight_rgba(&render_frame(&app, &theme, &engine));
        assert_eq!(rendered.len(), (CANVAS_W * CANVAS_H * 4) as usize);

        let path = directory.join(name);
        if update {
            let image = image::RgbaImage::from_raw(CANVAS_W, CANVAS_H, rendered)
                .expect("frame has the expected size");
            image.save(&path).expect("write golden snapshot");
            continue;
        }

        if !path.is_file() {
            failures.push(format!(
                "{name}: missing; run UPDATE_SNAPSHOTS=1 cargo test --test golden"
            ));
            continue;
        }

        let golden = image::open(&path)
            .expect("decode golden snapshot")
            .to_rgba8()
            .into_raw();
        if golden != rendered {
            let differing = golden
                .as_chunks::<4>()
                .0
                .iter()
                .zip(rendered.as_chunks::<4>().0)
                .filter(|(left, right)| left != right)
                .count();
            failures.push(format!("{name}: {differing} differing pixels"));
        }
    }

    assert!(
        failures.is_empty(),
        "golden mismatches:\n{}\nrun UPDATE_SNAPSHOTS=1 cargo test --test golden after reviewing",
        failures.join("\n")
    );
}
