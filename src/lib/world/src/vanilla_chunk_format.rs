use ferrumc_macros::NBTDeserialize;
use macro_rules_attribute::{apply, attribute_alias};

attribute_alias! {
    #[apply(ChunkDerives)] = #[derive(
        NBTDeserialize,
        Debug
        /*NBTSerialize, NBTDeserialize,
    Debug,
    Clone,
    PartialEq,
    Encode,
    Serialize,
    Decode,
    Deserialize,
    Eq*/
)];
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
#[nbt(is_root)]
#[nbt(rename = "")]
pub(crate) struct Chunk<'a> {
    pub dimension: Option<&'a str>,
    #[nbt(rename = "Status")]
    pub status: &'a str,
    #[nbt(rename = "DataVersion")]
    pub data_version: i32,
    #[nbt(rename = "Heightmaps")]
    pub heightmaps: Option<Heightmaps<'a>>,
    #[nbt(rename = "isLightOn")]
    pub is_light_on: Option<i8>,
    #[nbt(rename = "InhabitedTime")]
    pub inhabited_time: Option<i64>,
    #[nbt(rename = "yPos")]
    pub y_pos: i32,
    #[nbt(rename = "xPos")]
    pub x_pos: i32,
    #[nbt(rename = "zPos")]
    pub z_pos: i32,
    pub(crate) structures: Option<Structures<'a>>,
    #[nbt(rename = "LastUpdate")]
    pub last_update: Option<i64>,
    pub sections: Option<&'a [Section<'a>]>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
#[nbt(net_encode)]
pub(crate) struct Heightmaps<'a> {
    #[nbt(rename = "MOTION_BLOCKING")]
    pub motion_blocking: Option<&'a [i64]>,
    #[nbt(rename = "WORLD_SURFACE")]
    pub world_surface: Option<&'a [i64]>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Structures<'a> {
    pub starts: Starts,
    #[nbt(rename = "References")]
    pub references: References<'a>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Starts {}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct References<'a> {}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Section<'a> {
    #[nbt(rename = "block_states")]
    pub block_states: Option<BlockStates<'a>>,
    pub biomes: Option<Biomes<'a>>,
    #[nbt(rename = "Y")]
    pub y: i8,
    #[nbt(rename = "BlockLight")]
    pub block_light: Option<&'a [i8]>,
    #[nbt(rename = "SkyLight")]
    pub sky_light: Option<&'a [i8]>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct BlockStates<'a> {
    pub data: Option<&'a [i64]>,
    pub palette: Option<&'a [Palette<'a>]>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Palette<'a> {
    #[nbt(rename = "Name")]
    pub name: &'a str,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Properties<'a> {
    pub snowy: Option<&'a str>,
    pub level: Option<&'a str>,
    pub east: Option<&'a str>,
    pub waterlogged: Option<&'a str>,
    pub north: Option<&'a str>,
    pub west: Option<&'a str>,
    pub up: Option<&'a str>,
    pub down: Option<&'a str>,
    pub south: Option<&'a str>,
    pub drag: Option<&'a str>,
    pub lit: Option<&'a str>,
    pub axis: Option<&'a str>,
}

#[apply(ChunkDerives)]
#[derive(deepsize::DeepSizeOf)]
pub(crate) struct Biomes<'a> {
    pub palette: Vec<&'a str>,
}
