use eframe::egui::{self, FontData, FontDefinitions};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::{
    array, cmp,
    fmt::{self, Display},
    str::FromStr,
};

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (Op::Add, Op::Mul) => cmp::Ordering::Less,
            (Op::Add, Op::Div) => cmp::Ordering::Less,
            (Op::Sub, Op::Mul) => cmp::Ordering::Less,
            (Op::Sub, Op::Div) => cmp::Ordering::Less,
            (Op::Mul, Op::Add) => cmp::Ordering::Greater,
            (Op::Mul, Op::Sub) => cmp::Ordering::Greater,
            (Op::Div, Op::Add) => cmp::Ordering::Greater,
            (Op::Div, Op::Sub) => cmp::Ordering::Greater,
            _ => cmp::Ordering::Equal,
        })
    }
}

impl Ord for Op {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => f.write_str("+"),
            Op::Sub => f.write_str("-"),
            Op::Mul => f.write_str("*"),
            Op::Div => f.write_str("/"),
        }
    }
}

struct App {
    operands: [(Decimal, String); 4],
    ops: [Op; 3],
    rounding_strategy: RoundingStrategy,
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
        let default = dec!(0.000000);
        let default_string = format_with_spaces(&default.to_string());
        Self {
            operands: array::from_fn(|_| (default.clone(), default_string.clone())),
            ops: [Op::Add; 3],
            rounding_strategy: RoundingStrategy::MidpointAwayFromZero,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Гельманов Артур Русланович");
            ui.heading("4 курс 4 группа 2022");
            let mut input = |index: usize| -> Result<(), ()> {
                ui.text_edit_singleline(&mut self.operands[index].1);
                let result = if let Err(err) = check_spaces(&self.operands[index].1) {
                    ui.label(err);
                    Err(())
                } else {
                    let normalized = self.operands[index].1.replace(',', ".").replace(' ', "");

                    if let Ok(value) = Decimal::from_str(&normalized) {
                        if let Err(err) = check_limits(&value, &normalized) {
                            ui.label(err);
                            Err(())
                        } else {
                            self.operands[index].0 = value;
                            Ok(())
                        }
                    } else {
                        ui.label("возмутительный ввод");
                        Err(())
                    }
                };
                if result.is_ok() && index < self.ops.len() {
                    ui.push_id(index + 1000, |ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{}", self.ops[index]))
                            .show_ui(ui, |ui| {
                                for op in [Op::Add, Op::Sub, Op::Mul, Op::Div] {
                                    ui.selectable_value(&mut self.ops[index], op, op.to_string());
                                }
                            });
                    });
                }
                result
            };

            if input(0).is_ok() && input(1).is_ok() && input(2).is_ok() && input(3).is_ok() {
                let mut execute = |lhs: Decimal, rhs: Decimal, op| match op {
                    Op::Add => Some(lhs + rhs),
                    Op::Sub => Some(lhs - rhs),
                    Op::Mul => Some(lhs * rhs),
                    Op::Div => {
                        let result = lhs.checked_div(rhs);
                        if result.is_none() {
                            ui.label("деление на ноль");
                        }
                        result
                    }
                };
                if let Some(mut y) = execute(self.operands[1].0, self.operands[2].0, self.ops[1]) {
                    y = mathematical_round(10, &y);
                    if let Some(mut output) = if self.ops[2] > self.ops[0] {
                        execute(y, self.operands[3].0, self.ops[2]).map(|z| {
                            execute(self.operands[0].0, mathematical_round(10, &z), self.ops[0])
                        })
                    } else {
                        execute(self.operands[0].0, y, self.ops[0]).map(|x| {
                            execute(mathematical_round(10, &x), self.operands[3].0, self.ops[2])
                        })
                    }
                    .flatten()
                    {
                        output = mathematical_round(6, &output);
                        let output_string = output.to_string();

                        if let Err(err) = check_limits(&output, &output_string) {
                            ui.label(err);
                        } else {
                            ui.label(&format_with_spaces(&output_string));
                        }

                        ui.push_id(100000, |ui| {
                            egui::ComboBox::from_label("округление")
                                .selected_text(rounding_strategy_to_str(self.rounding_strategy))
                                .show_ui(ui, |ui| {
                                    for s in [
                                        RoundingStrategy::MidpointNearestEven,
                                        RoundingStrategy::MidpointAwayFromZero,
                                        RoundingStrategy::ToZero,
                                    ] {
                                        ui.selectable_value(
                                            &mut self.rounding_strategy,
                                            s,
                                            rounding_strategy_to_str(s),
                                        );
                                    }
                                });
                        });

                        let rounded_output =
                            output.round_dp_with_strategy(0, self.rounding_strategy);
                        ui.label(&format_with_spaces(&rounded_output.to_string()));
                    }
                }
            };
        });
    }
}

fn rounding_strategy_to_str(s: RoundingStrategy) -> &'static str {
    match s {
        RoundingStrategy::MidpointNearestEven => "бухгалтерское",
        RoundingStrategy::MidpointAwayFromZero => "математическое",
        RoundingStrategy::ToZero => "усечение",
        _ => unreachable!(),
    }
}

fn mathematical_round(dp: u32, x: &Decimal) -> Decimal {
    x.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero)
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
