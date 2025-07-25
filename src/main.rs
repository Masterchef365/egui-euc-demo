use egui::{pos2, DragValue, Vec2};
use egui_euc::SoftwareGui;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

struct App {
    tex: egui::TextureId,
    sub: SoftwareGui,
    demo: egui_demo_lib::DemoWindows,
    dims: [usize; 2],
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let options = egui::TextureOptions::NEAREST;
        let dims = [640, 480];
        let image = egui::ImageData::from(egui::ColorImage::filled(
            dims,
            egui::Color32::RED,
        ));

        let tex = cc
            .egui_ctx
            .tex_manager()
            .write()
            .alloc("sub-gui".into(), image, options);

        Self {
            demo: Default::default(),
            tex,
            dims,
            sub: SoftwareGui::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.strong("Resolution:");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.dims[0]));
                ui.label("x");
                ui.add(DragValue::new(&mut self.dims[1]));
            });
            let [width, height] = self.dims;

            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(width as _, height as _) / ui.pixels_per_point(),
                egui::Sense::click_and_drag(),
            );

            ui.painter().image(
                self.tex,
                rect,
                egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE), egui::StrokeKind::Outside);

            let mut raw_input = ctx.input(|r| r.raw.clone());

            raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(width as f32, height as f32) / ui.pixels_per_point()));

            for event in &mut raw_input.events {
                match event {
                    egui::Event::PointerMoved(pos) => {
                        *pos -= rect.min.to_vec2();
                    },
                    egui::Event::PointerButton { pos, .. } => {
                        *pos -= rect.min.to_vec2();
                    }
                    _ => (),
                }
            }

            //self.sub.egui_ctx.set_pixels_per_point(ui.pixels_per_point());
            let new_image = self.sub.update(raw_input, self.dims, |ctx| {
                self.demo.ui(ctx);
            });

            ui.ctx().tex_manager().write().set(
                self.tex,
                egui::epaint::ImageDelta::full(new_image, egui::TextureOptions::NEAREST),
            );

            ui.hyperlink_to("Source code", "https://github.com/Masterchef365/egui-euc");
        });
    }
}

