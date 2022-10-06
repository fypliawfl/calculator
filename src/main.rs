use eframe::egui::{self, FontData, FontDefinitions};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

const MAX: Decimal = dec!(1000000000000.000000);
const MIN: Decimal = dec!(-1000000000000.000000);
const MAX_FRACT_LEN: usize = 6;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Калькулятор",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

struct App {
    lhs: (Decimal, String),
    rhs: (Decimal, String),
    op: Op,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "stalinist".into(),
            FontData::from_static(include_bytes!("StalinistOne-Regular.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "stalinist".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        let default = dec!(100000000000.000000);
        let default_string = format_with_spaces(&default.to_string());
        Self {
            lhs: (default, default_string.clone()),
            rhs: (default, default_string),
            op: Op::Add,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Гельманов Артур Русланович");
                ui.heading("4 курс 4 группа 2022");
                let mut input = |operand: &mut (Decimal, String)| -> Result<(), ()> {
                    ui.text_edit_singleline(&mut operand.1);
                    if let Err(err) = check_spaces(&operand.1) {
                        ui.label(err);
                        Err(())
                    } else {
                        let normalized = operand.1.replace(',', ".").replace(' ', "");

                        if let Ok(value) = Decimal::from_str(&normalized) {
                            if let Err(err) = check_limits(&value, &normalized) {
                                ui.label(err);
                                Err(())
                            } else {
                                operand.0 = value;
                                Ok(())
                            }
                        } else {
                            ui.label("возмутительный ввод");
                            Err(())
                        }
                    }
                };

                if input(&mut self.lhs).is_ok() && input(&mut self.rhs).is_ok() {
                    if ui.small_button("+").clicked() {
                        self.op = Op::Add;
                    } else if ui.small_button("-").clicked() {
                        self.op = Op::Sub;
                    } else if ui.small_button("*").clicked() {
                        self.op = Op::Mul;
                    } else if ui.small_button("/").clicked() {
                        self.op = Op::Div;
                    }

                    if let Some(output) = match self.op {
                        Op::Add => Some(self.lhs.0 + self.rhs.0),
                        Op::Sub => Some(self.lhs.0 - self.rhs.0),
                        Op::Mul => Some(self.lhs.0 * self.rhs.0),
                        Op::Div => self.lhs.0.checked_div(self.rhs.0).map(|x| x.round_dp(6)),
                    } {
                        let output_string = output.to_string();

                        if let Err(err) = check_limits(&output, &output_string) {
                            ui.label(err);
                        } else {
                            ui.label(&format_with_spaces(&output_string));
                        }
                    } else {
                        ui.label("деление на ноль");
                    }
                };
            });
        });
    }
}

fn check_limits(x: &Decimal, s: &str) -> Result<(), &'static str> {
    if x < &MIN {
        Err("число меньше минимума")
    } else if x > &MAX {
        Err("число больше максимума")
    } else {
        let fract_len = s.chars().try_fold(0, |fract_len, ch| {
            if fract_len > MAX_FRACT_LEN {
                None
            } else if fract_len > 0 {
                Some(fract_len + 1)
            } else if ch == '.' {
                Some(1)
            } else {
                Some(0)
            }
        });
        if fract_len.is_none() {
            Err("много знаков после точки")
        } else {
            Ok(())
        }
    }
}

fn check_spaces(s: &str) -> Result<(), &'static str> {
    let mut chars = s.chars();
    let mut offset = 0;
    let mut is_first_offset = true;
    let mut is_prev_whitespace = false;
    if s.starts_with(' ') {
        return Err("ввод начинается c пробела");
    }
    for ch in &mut chars {
        if ch.is_whitespace() {
            if is_prev_whitespace {
                return Err("много пробелов подряд");
            } else if is_first_offset {
                if offset <= 3 {
                    is_first_offset = false;
                } else {
                    return Err("много цифр перед первым пробелом");
                }
            } else if offset != 3 {
                return Err("между пробелами в середине числа не по три цифры");
            }
            offset = 0;
            is_prev_whitespace = true;
        } else if ch == '.' {
            if offset != 3 && !is_first_offset {
                return Err("последний разделитель перед точкой в неположенном месте");
            }
            break;
        } else {
            offset += 1;
            is_prev_whitespace = false;
        }
    }

    for ch in chars {
        if ch.is_whitespace() {
            return Err("пробелы после точки");
        }
    }

    Ok(())
}

fn format_with_spaces(s: &str) -> String {
    let trunc_len = s
        .chars()
        .position(|ch| ch == '.')
        .unwrap_or_else(|| s.chars().count());
    let mut offset = trunc_len % 3;
    let mut string = String::with_capacity(s.len() + 6);
    for ch in s.chars().take(trunc_len) {
        if offset == 0 {
            if !string.is_empty() {
                string.push(' ');
            }
            offset = 3;
        }
        offset -= 1;
        string.push(ch);
    }
    string.extend(s.chars().skip(trunc_len));
    string
}
