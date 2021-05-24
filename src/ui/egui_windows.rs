//! Sets up the windows that permit more advanced settings.

use crate::{
    geometry::{Hypersphere, Point},
    polytope::concrete::Concrete,
    Float,
};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CtxRef, Layout, TextStyle, Ui},
    EguiContext,
};

pub struct EguiWindowPlugin;

impl Plugin for EguiWindowPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.insert_resource(EguiWindows::default())
            .add_system_to_stage(CoreStage::Update, show_windows.system())
            .add_system_to_stage(CoreStage::PostUpdate, update_windows.system());
    }
}

fn ok_reset(ui: &mut Ui) -> ShowResult {
    let mut result = ShowResult::None;

    ui.allocate_ui_with_layout(ui.min_size(), Layout::right_to_left(), |ui| {
        if ui.button("Ok").clicked() {
            result = ShowResult::Ok;
        } else if ui.button("Reset").clicked() {
            result = ShowResult::Reset;
        }
    });

    result
}

pub trait WindowType: Clone + Into<WindowTypeId> {
    /// The number of dimensions of the polytope on screen, used to set up the
    /// window.
    fn dim(&self) -> usize;

    /// The default state of the window.
    fn default(dim: usize) -> Self;

    /// Resets a window to its default state.
    fn reset(&mut self) {
        *self = Self::default(self.dim())
    }

    /// Shows the window on screen.
    fn show(&mut self, ctx: &CtxRef) -> ShowResult;

    /// Updates the window's settings after the polytope's dimension is updated.
    fn update(&mut self, dim: usize);
}

#[derive(Clone)]
pub struct DualWindow {
    center: Point,
    radius: Float,
}

impl WindowType for DualWindow {
    fn dim(&self) -> usize {
        self.center.len()
    }

    fn default(dim: usize) -> Self {
        Self {
            center: Point::zeros(dim),
            radius: 1.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::default(self.center.len());
    }

    fn show(&mut self, ctx: &CtxRef) -> ShowResult {
        let mut open = true;
        let mut result = ShowResult::None;

        egui::Window::new("Dual")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Center:");
                    for c in self.center.iter_mut() {
                        ui.add(egui::DragValue::new(c).speed(0.01));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(
                        egui::DragValue::new(&mut self.radius)
                            .speed(0.01)
                            .clamp_range(0.0..=Float::MAX),
                    );
                });

                result = ok_reset(ui);
            });

        if !open {
            ShowResult::Close
        } else {
            result
        }
    }

    fn update(&mut self, dim: usize) {
        self.center = self.center.clone().resize_vertically(dim, 0.0);
    }
}

impl From<DualWindow> for WindowTypeId {
    fn from(dual: DualWindow) -> Self {
        WindowTypeId::Dual(dual)
    }
}

#[derive(Clone)]
pub struct AntiprismWindow {
    dual: DualWindow,
    height: Float,
    central_inversion: bool,
}

impl WindowType for AntiprismWindow {
    fn dim(&self) -> usize {
        self.dual.dim()
    }

    fn default(dim: usize) -> Self {
        Self {
            dual: DualWindow::default(dim),
            height: 1.0,
            central_inversion: false,
        }
    }

    fn reset(&mut self) {
        *self = Self::default(self.dual.center.len());
    }

    fn show(&mut self, ctx: &CtxRef) -> ShowResult {
        let mut open = true;
        let mut result = ShowResult::None;

        egui::Window::new("Antiprism")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Center:");
                    for c in self.dual.center.iter_mut() {
                        ui.add(egui::DragValue::new(c).speed(0.01));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Radius:");
                    ui.add(
                        egui::DragValue::new(&mut self.dual.radius)
                            .speed(0.01)
                            .clamp_range(0.0..=Float::MAX),
                    );

                    ui.label("Height:");
                    ui.add(egui::DragValue::new(&mut self.height).speed(0.01));

                    ui.add(
                        egui::Checkbox::new(&mut self.central_inversion, "Central inversion:")
                            .text_style(TextStyle::Body),
                    );
                });

                result = ok_reset(ui);
            });

        if open {
            result
        } else {
            ShowResult::Close
        }
    }

    fn update(&mut self, dim: usize) {
        self.dual.update(dim);
    }
}

impl From<AntiprismWindow> for WindowTypeId {
    fn from(antiprism: AntiprismWindow) -> Self {
        WindowTypeId::Antiprism(antiprism)
    }
}

/// Makes sure that every window type is associated a unique ID (its enum
/// discriminant), which we can then use to test whether it's already in the
/// list of windows.
#[derive(Clone)]
pub enum WindowTypeId {
    Dual(DualWindow),
    Antiprism(AntiprismWindow),
}

/// Compares by discriminant.
impl std::cmp::PartialEq for WindowTypeId {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::cmp::Eq for WindowTypeId {}

/// The result of showing the windows every frame.
pub enum ShowResult {
    /// Nothing special happens.
    None,

    /// A window is closed.
    Close,

    /// A window is reset to its default state.
    Reset,

    /// A window runs some action.
    Ok,
}

impl WindowTypeId {
    /// Shows a given window on a given context.
    pub fn show(&mut self, ctx: &CtxRef) -> ShowResult {
        match self {
            Self::Dual(window) => window.show(ctx),
            Self::Antiprism(window) => window.show(ctx),
        }
    }

    /// Updates the window after the amount of dimensions of the polytope on
    /// screen changes.
    pub fn update(&mut self, dim: usize) {
        match self {
            Self::Dual(window) => window.update(dim),
            Self::Antiprism(window) => window.update(dim),
        }
    }

    /// Resets the window to its default state.
    pub fn reset(&mut self) {
        match self {
            Self::Dual(window) => window.reset(),
            Self::Antiprism(window) => window.reset(),
        }
    }
}

/// The list of all windows currently shown on screen.
pub struct EguiWindows(Vec<WindowTypeId>);

impl std::default::Default for EguiWindows {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl EguiWindows {
    /// Adds a new window to the list.
    pub fn push<T: WindowType>(&mut self, value: T) {
        let value = value.into();
        if !self.0.contains(&value) {
            self.0.push(value);
        }
    }

    /// Removes a window with a given index and returns it.
    pub fn swap_remove(&mut self, idx: usize) -> WindowTypeId {
        self.0.swap_remove(idx)
    }

    /// The number of windows on the screen.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Mutably iterates over all windows.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<WindowTypeId> {
        self.0.iter_mut()
    }

    /// Shows all of the windows, and returns one if its action has to run.
    pub fn show(&mut self, ctx: &CtxRef) -> Option<WindowTypeId> {
        let mut action_window = None;
        let window_count = self.len();

        for idx in 0..window_count {
            let window = &mut self.0[idx];

            match window.show(ctx) {
                ShowResult::Close => {
                    println!("Close");
                    self.swap_remove(idx);
                    break;
                }
                ShowResult::Ok => {
                    action_window = Some(self.swap_remove(idx));
                    break;
                }
                ShowResult::Reset => window.reset(),
                ShowResult::None => {}
            }
        }

        action_window
    }

    pub fn update(&mut self, dim: usize) {
        for window in self.iter_mut() {
            window.update(dim);
        }
    }
}

/// The system that shows the windows on screen.
fn show_windows(
    egui_ctx: Res<EguiContext>,
    mut query: Query<&mut Concrete>,
    mut egui_windows: ResMut<EguiWindows>,
) {
    if let Some(result) = egui_windows.show(egui_ctx.ctx()) {
        match result {
            WindowTypeId::Dual(DualWindow { center, radius }) => {
                let sphere = Hypersphere::new(center, radius * radius);

                for mut p in query.iter_mut() {
                    if let Err(err) = p.try_dual_mut_with(&sphere) {
                        println!("{:?}", err);
                    }
                }
            }
            WindowTypeId::Antiprism(AntiprismWindow {
                dual: DualWindow { center, radius },
                height,
                central_inversion,
            }) => {
                let mut squared_radius = radius * radius;
                if central_inversion {
                    squared_radius *= -1.0;
                }

                let sphere = Hypersphere::new(center, squared_radius);

                for mut p in query.iter_mut() {
                    match p.try_antiprism_with(&sphere, height) {
                        Ok(q) => *p = q,
                        Err(err) => println!("{:?}", err),
                    }
                }
            }
        }
    }
}

/// Updates the windows after the polytopes change.
pub fn update_windows(
    polies: Query<(&Concrete, &Handle<Mesh>, &Children), Changed<Concrete>>,
    mut egui_windows: ResMut<EguiWindows>,
) {
    if let Some((poly, _, _)) = polies.iter().next() {
        egui_windows.update(poly.dim().unwrap_or(0));
    }
}
