use galileo::layer::vector_tile_layer::tile_provider::loader::{TileLoadError, VectorTileLoader};
use galileo::tile_schema::{TileIndex, VerticalDirection};
use galileo::{Lod, TileSchema};
use galileo_mvt::MvtTile;
use galileo_types::cartesian::{Point2, Rect};
use galileo_types::geo::Crs;
use pmtiles::async_reader::{AsyncBackend, AsyncPmTilesReader};
use pmtiles::cache::DirectoryCache;
use tokio::io::AsyncReadExt;

pub struct ProtomapVectorTileLoader<B, C> {
    reader: AsyncPmTilesReader<B, C>,
}

impl<B, C> ProtomapVectorTileLoader<B, C> {
    pub fn new(reader: AsyncPmTilesReader<B, C>) -> Self {
        Self { reader }
    }
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

pub trait TileSchemaExt {
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
