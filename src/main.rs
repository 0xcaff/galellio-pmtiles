use eframe::CreationContext;
use galileo::control::{EventPropagation, MouseButton, UserEvent, UserEventHandler};
use galileo::layer::vector_tile_layer::style::{
    StyleRule, VectorTileLineSymbol, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::layer::vector_tile_layer::tile_provider::loader::{TileLoadError, VectorTileLoader};
use galileo::layer::vector_tile_layer::tile_provider::VectorTileProvider;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::platform::native::vt_processor::ThreadVtProcessor;
use galileo::tile_schema::{TileIndex, TileSchema, VerticalDirection};
use galileo::{Color, Lod, Map, MapBuilder};
use galileo_egui::{EguiMap, EguiMapState};
use galileo_mvt::MvtTile;
use galileo_types::cartesian::{Point2, Rect};
use galileo_types::geo::Crs;
use parking_lot::RwLock;
use pmtiles::async_reader::{AsyncBackend, AsyncPmTilesReader};
use pmtiles::cache::{DirectoryCache, NoCache};
use pmtiles::{Compression, HttpBackend, TileType};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
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
        let Ok(result) = self
            .reader
            .get_tile(index.z as _, index.x as _, index.y as _)
            .await
        else {
            return Err(TileLoadError::Network);
        };

        let Some(bytes) = result else {
            return Err(TileLoadError::DoesNotExist);
        };

        let mut decompressed_bytes = Vec::new();
        let Ok(..) = async_compression::tokio::bufread::GzipDecoder::new(&bytes[..])
            .read_to_end(&mut decompressed_bytes)
            .await
        else {
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
    let backend = HttpBackend::try_from(client, "https://demo-bucket.protomaps.com/v4.pmtiles")?;
    let backend = AsyncPmTilesReader::try_from_cached_source(backend, NoCache).await?;

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
    .with_style(VectorTileStyle {
        rules: vec![
            StyleRule {
                layer_name: Some("water".to_string()),
                properties: HashMap::new(),
                symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
                    fill_color: Color::from_hex("#80deea"),
                }),
            },
            style_rule_kind("landcover", "grassland", (210, 239, 207, u8::MAX)),
            style_rule_kind("landcover", "barren", (255, 243, 215, u8::MAX)),
            style_rule_kind("landcover", "urban_area", (230, 230, 230, u8::MAX)),
            style_rule_kind("landcover", "farmland", (216, 239, 210, u8::MAX)),
            style_rule_kind("landcover", "glacier", (255, 255, 255, u8::MAX)),
            style_rule_kind("landcover", "scrub", (234, 239, 210, u8::MAX)),
            style_rule("landcover", (196, 231, 210, u8::MAX)),
            // style_rule_kind("landuse", "national_park", (156, 211, 180, u8::MAX)),
            // style_rule_kind("landuse", "park", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "cemetery", (156, 211, 180, u8::MAX)),
            // style_rule_kind("landuse", "protected_area", (156, 211, 180, u8::MAX)),
            // style_rule_kind("landuse", "nature_reserve", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "forest", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "golf_course", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "wood", (160, 217, 160, u8::MAX)),
            style_rule_kind("landuse", "scrub", (153, 210, 187, u8::MAX)),
            style_rule_kind("landuse", "grassland", (153, 210, 187, u8::MAX)),
            style_rule_kind("landuse", "grass", (153, 210, 187, u8::MAX)),
            style_rule_kind("landuse", "glacier", (231, 231, 231, u8::MAX)),
            style_rule_kind("landuse", "sand", (226, 224, 215, u8::MAX)),
            style_rule_kind("landuse", "military", (198, 220, 220, u8::MAX)),
            style_rule_kind("landuse", "naval_base", (198, 220, 220, u8::MAX)),
            style_rule_kind("landuse", "airfield", (198, 220, 220, u8::MAX)),
            style_rule_kind("landuse", "allotments", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "village_green", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "playground", (156, 211, 180, u8::MAX)),
            style_rule_kind("landuse", "hospital", (228, 218, 217, u8::MAX)),
            style_rule_kind("landuse", "industrial", (209, 221, 225, u8::MAX)),
            style_rule_kind("landuse", "school", (228, 222, 215, u8::MAX)),
            style_rule_kind("landuse", "university", (228, 222, 215, u8::MAX)),
            style_rule_kind("landuse", "college", (228, 222, 215, u8::MAX)),
            style_rule_kind("landuse", "beach", (232, 228, 208, u8::MAX)),
            style_rule_kind("landuse", "zoo", (198, 220, 220, u8::MAX)),
            style_rule_kind("landuse", "aerodrome", (218, 219, 223, u8::MAX)),
            style_rule_kind("landuse", "runway", (233, 233, 237, u8::MAX)),
            style_rule_kind("landuse", "taxiway", (233, 233, 237, u8::MAX)),
            style_rule_kind("landuse", "pedestrian", (227, 224, 212, u8::MAX)),
            style_rule_kind("landuse", "dam", (227, 224, 212, u8::MAX)),
            style_rule_kind("landuse", "pier", (224, 224, 224, u8::MAX)),
            boundary_style(0, 0.7), // International boundaries
            boundary_style(1, 0.7), // First-level admin (states/provinces)
            boundary_style(2, 0.7), // Second-level admin (counties/districts)
            boundary_style(3, 0.4), // Third-level admin (municipalities/townships)
            boundary_style(4, 0.4), // Lower administrative divisions (neighborhoods, city districts)
            boundary_style(5, 0.4), // Additional finer levels if applicable
            road_style("highway", 3.0, Color::from_hex("#ffffff")), // Highway road style (widest, white)
            road_style("major_road", 2.0, Color::from_hex("#ffffff")), // Major road style (wide, white)
            road_style("minor_road", 1.0, Color::from_hex("#ebebeb")), // Minor road style (medium, light gray to white)
            road_style("other", 0.5, Color::from_hex("#ebebeb")), // Other road style (narrow, light gray)
            road_style("path", 0.5, Color::from_hex("#ebebeb")),  // Path style (narrow, light gray)
            road_style("rail", 1.0, Color::from_hex("#a7b1b3")), // Rail style (narrow, grayish blue)
        ],
        default_symbol: Default::default(),
        background: Color::from_hex("#cccccc"),
    })
    .with_tile_schema(schema)
    .build()
    .expect("failed to create layer");

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
        ..Default::default()
    };

    eframe::run_native(
        "Galileo Dev Map",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(map, cc, handler)))),
    )
    .expect("failed to create eframe app");

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

fn style_rule_kind(layer: &str, kind: &str, color: (u8, u8, u8, u8)) -> StyleRule {
    StyleRule {
        layer_name: Some(layer.to_string()),
        properties: HashMap::from([("kind".to_string(), kind.to_string())]),
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: Color::rgba(color.0, color.1, color.2, color.3),
        }),
    }
}

fn style_rule(layer: &str, color: (u8, u8, u8, u8)) -> StyleRule {
    StyleRule {
        layer_name: Some(layer.to_string()),
        properties: HashMap::new(),
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: Color::rgba(color.0, color.1, color.2, color.3),
        }),
    }
}

fn boundary_style(kind_detail: u8, width: f64) -> StyleRule {
    StyleRule {
        layer_name: Some("boundaries".to_string()),
        properties: HashMap::from([("kind_detail".to_string(), kind_detail.to_string())]),
        symbol: VectorTileSymbol::Line(VectorTileLineSymbol {
            width,
            stroke_color: Color::from_hex("#adadad"),
        }),
    }
}

fn road_style(kind: &str, width: f64, stroke_color: Color) -> StyleRule {
    StyleRule {
        layer_name: Some("roads".to_string()),
        properties: HashMap::from([("kind".to_string(), kind.to_string())]),
        symbol: VectorTileSymbol::Line(VectorTileLineSymbol {
            width,
            stroke_color,
        }),
    }
}

trait TileSchemaExt {
    fn pmtiles(lods: u32) -> Self;
}

impl TileSchemaExt for TileSchema {
    fn pmtiles(lods: u32) -> TileSchema {
        let max_resolution = 156543.03392804097;
        let lods = (0..lods)
            .map(|z| Lod::new(max_resolution / 2f64.powi(z as i32), z).unwrap())
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
