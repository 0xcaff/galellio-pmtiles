//! This exmpale shows how to create and work with vector tile layers.

use std::sync::Arc;
use eframe::CreationContext;
use galileo::control::{EventPropagation, MouseButton, UserEvent, UserEventHandler};
use galileo::layer::vector_tile_layer::tile_provider::loader::{TileLoadError, VectorTileLoader};
use galileo::layer::vector_tile_layer::tile_provider::VectorTileProvider;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::platform::native::vt_processor::ThreadVtProcessor;
use galileo::tile_schema::{TileIndex, TileSchema, VerticalDirection};
use galileo::{Lod, Map, MapBuilder};
use galileo_egui::{EguiMap, EguiMapState};
use galileo_mvt::MvtTile;
use galileo_types::cartesian::{Point2, Rect};
use galileo_types::geo::Crs;
use parking_lot::RwLock;
use pmtiles::async_reader::{AsyncBackend, AsyncPmTilesReader};
use pmtiles::cache::DirectoryCache;
use pmtiles::reqwest::Client;
use pmtiles::{Compression, TileType};
use tokio::io::AsyncReadExt;

struct ProtomapVectorTileLoader<B, C> {
    reader: AsyncPmTilesReader<B, C>,
}

#[async_trait::async_trait]
impl<B, C> VectorTileLoader for ProtomapVectorTileLoader<B, C>
where
    B: AsyncBackend + Sync + Send,
    C: DirectoryCache + Sync + Send,
{
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError> {
        let Ok(result) = self.reader
                    .get_tile(index.z as _, index.x as _, index.y as _).await else {
            return Err(TileLoadError::Network);
        };

        let Some(bytes) = result else {
            return Err(TileLoadError::DoesNotExist);
        };

        let mut decompressed_bytes = Vec::new();
        let Ok(..) = async_compression::tokio::bufread::GzipDecoder::new(&bytes[..])
            .read_to_end(&mut decompressed_bytes)
            .await else {
            return Err(TileLoadError::Decoding);
        };

        let Ok(it) = MvtTile::decode(&*decompressed_bytes, true) else {
            return Err(TileLoadError::Decoding);
        };

        Ok(it)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = Client::new();
    let backend =
        AsyncPmTilesReader::new_with_url(client, "https://demo-bucket.protomaps.com/v4.pmtiles")
            .await?;

    let header = backend.get_header();
    assert_eq!(header.tile_compression, Compression::Gzip);
    assert_eq!(header.tile_type, TileType::Mvt);

    let metadata = backend.get_metadata().await?;
    println!("metadata: {}", metadata);

    assert_eq!(header.min_zoom, 0);

    let schema = TileSchema::pmtiles(header.max_zoom as _);
    let layer = VectorTileLayerBuilder::new_with_provider(VectorTileProvider::new(
        Arc::new(ProtomapVectorTileLoader { reader: backend }),
        Arc::new(ThreadVtProcessor::new(schema.clone())),
    ))
    .with_tile_schema(schema)
    .build()
    .expect("failed to create layer");

    let layer = Arc::new(RwLock::new(layer));

    let layer_copy = layer.clone();
    let handler = move |ev: &UserEvent, map: &mut Map| {
        match ev {
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
        }
    };

    let map = MapBuilder::default()
        .with_latlon(0., 0.)
        .with_z_level(0)
        .with_layer(layer.clone()).build();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 1000.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native("Galileo Dev Map", native_options, Box::new(|cc| {
        Ok(Box::new(App::new(map, cc, handler)))
    })).expect("failed to create eframe app");

    Ok(())
}

struct App {
    map: EguiMapState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            EguiMap::new(&mut self.map).show_ui(ui);
        });
    }
}

impl App {
    fn new(
        map: Map,
        cc: &CreationContext,
        handler: impl UserEventHandler + 'static,
    ) -> Self {
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

trait TileSchemaExt {
    fn pmtiles(lods: u32) -> Self;
}

impl TileSchemaExt for TileSchema {
    fn pmtiles(lods: u32) -> TileSchema {
        let max_resolution = 156543.03392804097;
        let lods = (0..lods)
            .map(|z| Lod::new(
                max_resolution / 2f64.powi(z as i32),
                z,
            ).unwrap())
            .collect();

        TileSchema {
            origin: Point2::new(-20037508.342789, 20037508.342789),
            bounds: Rect::new(
                -20037508.342789,
                -20037508.342789,
                20037508.342789,
                20037508.342789,
            ),
            lods,
            tile_width: 256,
            tile_height: 256,
            y_direction: VerticalDirection::TopToBottom,
            crs: Crs::EPSG3857,
        }
    }
}

