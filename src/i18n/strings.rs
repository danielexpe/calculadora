#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strings {
    pub app_name: &'static str,
    pub history_title: &'static str,
    pub history_empty: &'static str,
    pub footer_hints: &'static str,
    pub lang_pt: &'static str,
    pub lang_en: &'static str,
    pub sound_on: &'static str,
    pub sound_off: &'static str,
    pub sound_unavailable: &'static str,
    pub error_title: &'static str,
    pub error_div_zero: &'static str,
    pub error_overflow: &'static str,
    pub error_other: &'static str,
    pub lang_name_pt: &'static str,
    pub lang_name_en: &'static str,
}

impl Strings {
    pub fn values(&self) -> [&'static str; 15] {
        [
            self.app_name,
            self.history_title,
            self.history_empty,
            self.footer_hints,
            self.lang_pt,
            self.lang_en,
            self.sound_on,
            self.sound_off,
            self.sound_unavailable,
            self.error_title,
            self.error_div_zero,
            self.error_overflow,
            self.error_other,
            self.lang_name_pt,
            self.lang_name_en,
        ]
    }
}

pub static PT: Strings = Strings {
    app_name: "Calculadora",
    history_title: "Histórico",
    history_empty: "Sem operações ainda",
    footer_hints: "F2 Idioma · M Som · Q Sair · ↑↓ Histórico",
    lang_pt: "PT",
    lang_en: "EN",
    sound_on: "Som: ligado",
    sound_off: "Som: desligado",
    sound_unavailable: "Som: indisponível",
    error_title: "Erro",
    error_div_zero: "Divisão por zero",
    error_overflow: "Número muito grande",
    error_other: "Erro de cálculo",
    lang_name_pt: "Português (BR)",
    lang_name_en: "Inglês",
};

pub static EN: Strings = Strings {
    app_name: "Calculadora",
    history_title: "History",
    history_empty: "No operations yet",
    footer_hints: "F2 Language · M Sound · Q Quit · ↑↓ History",
    lang_pt: "PT",
    lang_en: "EN",
    sound_on: "Sound: on",
    sound_off: "Sound: off",
    sound_unavailable: "Sound: unavailable",
    error_title: "Error",
    error_div_zero: "Division by zero",
    error_overflow: "Number too large",
    error_other: "Calculation error",
    lang_name_pt: "Portuguese (BR)",
    lang_name_en: "English",
};
