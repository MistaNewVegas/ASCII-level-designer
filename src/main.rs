use std::fs::File;
use std::error::Error;
use csv::Writer;
use eframe::egui;
use eframe::IconData; // Import IconData from eframe
use std::sync::{Arc, Mutex};

const DEFAULT_SIZE: usize = 25; // 25x25 grid
const CELL_SIZE: f32 = 25.0;    // Each cell is 25x25 pixels
const GRID_MARGIN: f32 = 30.0;  // Extra margin added to the grid container

type GridType = Vec<Vec<char>>;

struct LevelEditor {
    textures: Arc<Mutex<GridType>>,
    entities: Arc<Mutex<GridType>>,
}

impl LevelEditor {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            textures: Arc::new(Mutex::new(vec![vec![' '; cols]; rows])),
            entities: Arc::new(Mutex::new(vec![vec![' '; cols]; rows])),
        }
    }

    fn export_to_csv(&self) -> Result<(), Box<dyn Error>> {
        let file_textures = File::create("textures.csv")?;
        let file_entities = File::create("entities.csv")?;
        let mut wtr_textures = Writer::from_writer(file_textures);
        let mut wtr_entities = Writer::from_writer(file_entities);

        for row in &*self.textures.lock().unwrap() {
            let record: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            wtr_textures.write_record(record)?;
        }
        for row in &*self.entities.lock().unwrap() {
            let record: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            wtr_entities.write_record(record)?;
        }

        wtr_textures.flush()?;
        wtr_entities.flush()?;
        Ok(())
    }
}

// New enum to represent brush types.
#[derive(PartialEq)]
enum Brush {
    Wall,
    Enemy,
    Checkpoint,
    Fruit,
    Clear,
}

struct App {
    editor: Arc<LevelEditor>,
    current_brush: Brush,
}

impl App {
    fn new(editor: Arc<LevelEditor>) -> Self {
        Self {
            editor,
            current_brush: Brush::Wall, // Default brush
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Remove extra spacing in the UI.
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
        style.spacing.button_padding = egui::vec2(0.0, 0.0);
        ctx.set_style(style);

        // Top toolbar with title, export button, and brush selection.
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("Daniel's ASCII Level Editor  ");
                if ui.button(" Export to CSV ").clicked() {
                    self.editor.export_to_csv().unwrap();
                }
                ui.separator();

                // Group brush selection buttons with reduced spacing and underlined text.
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                    ui.label("Brush:");
                    if ui.selectable_label(
                        self.current_brush == Brush::Wall,
                        egui::RichText::new("Wall # ").underline()
                    ).clicked() {
                        self.current_brush = Brush::Wall;
                    }
                    if ui.selectable_label(
                        self.current_brush == Brush::Enemy,
                        egui::RichText::new("Enemy @ ").underline()
                    ).clicked() {
                        self.current_brush = Brush::Enemy;
                    }
                    if ui.selectable_label(
                        self.current_brush == Brush::Checkpoint,
                        egui::RichText::new("Checkpoint ! ").underline()
                    ).clicked() {
                        self.current_brush = Brush::Checkpoint;
                    }
                    if ui.selectable_label(
                        self.current_brush == Brush::Fruit,
                        egui::RichText::new("Fruit $ ").underline()
                    ).clicked() {
                        self.current_brush = Brush::Fruit;
                    }
                    if ui.selectable_label(
                        self.current_brush == Brush::Clear,
                        egui::RichText::new("Clear ").underline()
                    ).clicked() {
                        self.current_brush = Brush::Clear;
                    }
                });
            });
        });

        // Central panel for instructions and grid.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Left-click or drag to paint cells with the selected brush; Shift+right-click clears.");
            ui.separator();

            // Compute grid dimensions: grid is now zero-spaced.
            let grid_width = DEFAULT_SIZE as f32 * CELL_SIZE + GRID_MARGIN;
            let grid_height = DEFAULT_SIZE as f32 * CELL_SIZE + GRID_MARGIN;
            // Allocate a container slightly larger than the grid.
            let (grid_rect, _grid_response) =
                ui.allocate_exact_size(egui::Vec2::new(grid_width, grid_height), egui::Sense::hover());
            let painter = ui.painter();

            // Calculate an offset so the grid is centered within the container.
            let offset = GRID_MARGIN / 2.0;

            // Create a custom FontId for bold, larger text.
            let font = egui::FontId::new(20.0, egui::FontFamily::Proportional);

            // Draw the grid manually.
            for y in 0..DEFAULT_SIZE {
                for x in 0..DEFAULT_SIZE {
                    let cell_x = grid_rect.min.x + offset + x as f32 * CELL_SIZE;
                    let cell_y = grid_rect.min.y + offset + y as f32 * CELL_SIZE;
                    let cell_rect = egui::Rect::from_min_size(egui::pos2(cell_x, cell_y), egui::vec2(CELL_SIZE, CELL_SIZE));

                    // Fetch current cell contents.
                    let tile = self.editor.textures.lock().unwrap()[y][x];
                    let entity = self.editor.entities.lock().unwrap()[y][x];
                    let label = if entity != ' ' {
                        format!("{}", entity)
                    } else if tile == ' ' {
                        "  ".to_string()
                    } else {
                        format!("{}", tile)
                    };

                    // Draw cell background and border.
                    painter.rect_filled(cell_rect, 0.0, egui::Color32::from_gray(30));
                    painter.rect_stroke(cell_rect, 0.0, (1.0, egui::Color32::DARK_GRAY));
                    
                    // Draw bold, larger text by drawing a shadow copy then the main text.
                    painter.text(
                        cell_rect.center() + egui::vec2(1.0, 1.0),
                        egui::Align2::CENTER_CENTER,
                        label.clone(),
                        font.clone(),
                        egui::Color32::from_black_alpha(200)
                    );
                    painter.text(
                        cell_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        font.clone(),
                        egui::Color32::WHITE
                    );
                }
            }

            // --- Handle Drag Painting Over the Entire Grid ---
            if ui.input(|i| i.pointer.primary_down()) {
                if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                    // Adjust the hit test by the offset.
                    let adjusted_rect = egui::Rect::from_min_size(
                        grid_rect.min + egui::vec2(offset, offset),
                        egui::vec2(DEFAULT_SIZE as f32 * CELL_SIZE, DEFAULT_SIZE as f32 * CELL_SIZE)
                    );
                    if adjusted_rect.contains(pos) {
                        // Compute which cell is hovered.
                        let relative_pos = pos - (grid_rect.min + egui::vec2(offset, offset));
                        let col = (relative_pos.x / CELL_SIZE).floor() as usize;
                        let row = (relative_pos.y / CELL_SIZE).floor() as usize;
                        if row < DEFAULT_SIZE && col < DEFAULT_SIZE {
                            let mut textures = self.editor.textures.lock().unwrap();
                            let mut entities = self.editor.entities.lock().unwrap();
                            match self.current_brush {
                                Brush::Wall => {
                                    textures[row][col] = '#';
                                }
                                Brush::Enemy => {
                                    entities[row][col] = '&';
                                }
                                Brush::Checkpoint => {
                                    entities[row][col] = '!';
                                }
                                Brush::Fruit => {
                                    entities[row][col] = '$';
                                }
                                Brush::Clear => {
                                    textures[row][col] = ' ';
                                    entities[row][col] = ' ';
                                }
                            }
                        }
                    }
                }
            }


            for y in 0..DEFAULT_SIZE {
                for x in 0..DEFAULT_SIZE {
                    let cell_x = grid_rect.min.x + offset + x as f32 * CELL_SIZE;
                    let cell_y = grid_rect.min.y + offset + y as f32 * CELL_SIZE;
                    let cell_rect = egui::Rect::from_min_size(egui::pos2(cell_x, cell_y), egui::vec2(CELL_SIZE, CELL_SIZE));
                    let response = ui.interact(cell_rect, ui.make_persistent_id(format!("cell_{}_{}", x, y)), egui::Sense::click());
                    if response.clicked() {
                        let mut textures = self.editor.textures.lock().unwrap();
                        let mut entities = self.editor.entities.lock().unwrap();
                        match self.current_brush {
                            Brush::Wall => {
                                textures[y][x] = '#';
                            }
                            Brush::Enemy => {
                                entities[y][x] = '&';
                            }
                            Brush::Checkpoint => {
                                entities[y][x] = '!';
                            }
                            Brush::Fruit => {
                                entities[y][x] = '$';
                            }
                            Brush::Clear => {
                                textures[y][x] = ' ';
                                entities[y][x] = ' ';
                            }
                        }
                    }
                    if response.secondary_clicked() && ui.input(|i| i.modifiers.shift) {
                        let mut textures = self.editor.textures.lock().unwrap();
                        let mut entities = self.editor.entities.lock().unwrap();
                        textures[y][x] = ' ';
                        entities[y][x] = ' ';
                    }
                }
            }
        });
    }
}

// load icon
fn load_icon() -> Option<IconData> {
    let icon_bytes = include_bytes!("daniel.ico");
    let image = image::load_from_memory(icon_bytes).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Some(IconData {
        rgba: image.into_raw(),
        width: width as u32,
        height: height as u32,
    })
}

fn main() -> eframe::Result<()> {
    let editor = LevelEditor::new(DEFAULT_SIZE, DEFAULT_SIZE);
    let editor_arc = Arc::new(editor);

    let grid_width = DEFAULT_SIZE as f32 * CELL_SIZE + GRID_MARGIN;
    let grid_height = DEFAULT_SIZE as f32 * CELL_SIZE + GRID_MARGIN;
    
    let toolbar_height = 40.0;
    let instructions_height = 30.0;

    let window_width = grid_width;
    let window_height = grid_height + toolbar_height + instructions_height;

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(window_width, window_height)),
        resizable: false,
        initial_window_pos: Some(egui::Pos2::new(100.0, 100.0)),
        icon_data: load_icon(), 
        ..Default::default()
    };

    eframe::run_native(
        "Daniel's ASCII Level Editor v1.0",
        native_options,
        Box::new(|_cc| Box::new(App::new(editor_arc))),
    )
}
