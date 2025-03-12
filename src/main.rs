mod protomap_loader;
mod style;

use crate::protomap_loader::{ProtomapVectorTileLoader, TileSchemaExt};
use crate::style::make_style;
use eframe::CreationContext;
use egui::{Frame, Margin};
use galileo::control::{EventPropagation, MouseButton, UserEvent, UserEventHandler};
use galileo::layer::vector_tile_layer::tile_provider::VectorTileProvider;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::platform::native::vt_processor::ThreadVtProcessor;
use galileo::tile_schema::TileSchema;
use galileo::{Map, MapBuilder};
use galileo_egui::{EguiMap, EguiMapState};
use parking_lot::RwLock;
use pmtiles::async_reader::AsyncPmTilesReader;
use pmtiles::cache::NoCache;
use pmtiles::{Compression, HttpBackend, TileType};
use reqwest::Client;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = Client::new();
    let backend = HttpBackend::try_from(client, "https://demo-bucket.protomaps.com/v4.pmtiles")?;
    let backend = AsyncPmTilesReader::try_from_cached_source(backend, NoCache).await?;

    let header = backend.get_header();
    assert_eq!(header.tile_compression, Compression::Gzip);
    assert_eq!(header.tile_type, TileType::Mvt);
    assert_eq!(header.min_zoom, 0);

    let schema = TileSchema::pmtiles(header.max_zoom as _);
    let layer = VectorTileLayerBuilder::new_with_provider(VectorTileProvider::new(
        Arc::new(ProtomapVectorTileLoader::new(backend)),
        Arc::new(ThreadVtProcessor::new(schema.clone())),
    ))
    .with_style(make_style())
    .with_tile_schema(schema)
    .build()?;

    let layer = Arc::new(RwLock::new(layer));

    let layer_copy = layer.clone();
    let handler = move |ev: &UserEvent, map: &mut Map| match ev {
        UserEvent::Click(MouseButton::Left, mouse_event) => {
            let view = map.view().clone();
            if let Some(position) = map
                .view()
                .screen_to_map(mouse_event.screen_pointer_position)
            {
                let features = layer_copy.read().get_features_at(&position, &view);

                for (layer, feature) in features {
                    println!("{layer}, {:?}", feature.properties);
                }
            }

            EventPropagation::Stop
        }
        _ => EventPropagation::Propagate,
    };

    let map = MapBuilder::default()
        .with_latlon(0., 0.)
        .with_z_level(0)
        .with_layer(layer.clone())
        .build();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 1000.0])
            .with_min_inner_size([300.0, 220.0]),
        window_builder: Some(Box::new(|it| {
            it.with_fullsize_content_view(true)
                .with_titlebar_shown(false)
                .with_title_shown(false)
        })),
        ..Default::default()
    };

    eframe::run_native(
        "Galileo Dev Map",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(map, cc, handler)))),
    )?;

    Ok(())
}

struct App {
    map: EguiMapState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(Margin::ZERO))
            .show(ctx, |ui| {
                EguiMap::new(&mut self.map).show_ui(ui);
            });
    }
}

impl App {
    fn new(map: Map, cc: &CreationContext, handler: impl UserEventHandler + 'static) -> Self {
        Self {
            map: EguiMapState::new(
                map,
                cc.egui_ctx.clone(),
                cc.wgpu_render_state.clone().expect("no render state"),
                [Box::new(handler) as Box<dyn UserEventHandler>],
            ),
        }
    }
}
