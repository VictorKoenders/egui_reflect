#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

pub use egui_reflect_derive::EguiReflect;

use egui::Ui;

pub fn reflect<R>(ui: &mut Ui, value: &mut R)
where
    R: Reflectable,
{
    let id = ui
        .id()
        .with("egui_reflect")
        .with(std::any::type_name::<R>());

    for field in value.reflect() {
        field
            .value
            .editor(ui, field.name, id.with(field.name), field.opts);
    }
}

pub trait Reflectable {
    fn reflect(&mut self) -> Vec<ReflectField>;
}

pub struct ReflectField<'a> {
    pub name: &'static str,
    pub value: &'a mut dyn ReflectValue,
    pub opts: FieldOptions,
}

pub trait ReflectValue {
    fn editor(&mut self, ui: &mut Ui, name: &str, id: egui::Id, options: FieldOptions);
}

macro_rules! impl_reflect_for_integers {
    ($($t:ty),*) => {
        $(
            impl ReflectValue for $t {
                fn editor(&mut self, ui: &mut Ui, name: &str, id: egui::Id, options: FieldOptions) {

                    ui.horizontal(|ui| {
                        ui.label(name);

                        if let Some((min, max)) = options.range {
                            ui.add(egui::Slider::new(self, (min as $t)..=(max as $t)));
                        } else {
                            let mut state = ui.ctx().data_mut(|w| {
                                return w.get_temp_mut_or_insert_with::<(String, bool)>(id, || {
                                    (self.to_string(), false)
                                }).clone();
                            });
                            ui.text_edit_singleline(&mut state.0);
                            if state.0 != self.to_string() {
                                if let Ok(value) = state.0.parse::<$t>() {
                                    *self = value;
                                    state.1 = false;
                                } else {
                                    state.1 = true;
                                }
                            }
                            ui.ctx().data_mut(|w| {
                                w.insert_temp(id, state);
                            });
                        }
                    });

                }
            }
        )*
    };
}

impl_reflect_for_integers!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

impl ReflectValue for bool {
    fn editor(&mut self, ui: &mut Ui, name: &str, _id: egui::Id, _: FieldOptions) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.checkbox(self, "");
        });
    }
}

impl ReflectValue for String {
    fn editor(&mut self, ui: &mut Ui, name: &str, _id: egui::Id, _: FieldOptions) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.text_edit_singleline(self);
        });
    }
}

impl<T> ReflectValue for T
where
    T: Reflectable,
{
    fn editor(&mut self, ui: &mut Ui, name: &str, _id: egui::Id, _: FieldOptions) {
        ui.collapsing(name, |ui| {
            reflect(ui, self);
        });
    }
}

#[cfg(feature = "glam")]
impl ReflectValue for glam::Vec2 {
    fn editor(&mut self, ui: &mut Ui, name: &str, _id: egui::Id, _: FieldOptions) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.columns(2, |columns| {
                columns[0].add(egui::DragValue::new(&mut self.x));
                columns[1].add(egui::DragValue::new(&mut self.y));
            });
        });
    }
}

#[derive(Default)]
pub struct FieldOptions {
    pub range: Option<(i32, i32)>,
}
