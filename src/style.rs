use galileo::layer::vector_tile_layer::style::{
    StyleRule, VectorTileLineSymbol, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::Color;
use std::collections::HashMap;

pub fn make_style() -> VectorTileStyle {
    VectorTileStyle {
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
