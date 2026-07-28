use binspec::{
    ByteSource,
    View,
    array,
    assert_spec,
    assert_spec_eq,
    spec_error,
    default_main,
    errors::SResult,
    specs::{
        Spec,
        U8, U16, U32
    }
};

#[derive(Debug)]
pub struct Color {
    pub red: U8,
    pub green: U8,
    pub blue: U8,
}

impl Spec for Color {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        let blue = U8::read_from_view(&mut view,  ())?;
        let green = U8::read_from_view(&mut view,  ())?;
        let red = U8::read_from_view(&mut view,  ())?;
        let reserved = U8::read_from_view(&mut view, ())?;
        assert_spec_eq!(reserved.value, 0);
        Ok((Color {red, green, blue}, view.cursor.offset))
    }
}

#[derive(Debug)]
pub struct FileHeader {
    pub filesize: U32,
    pub data_offset: U32
}

impl Spec for FileHeader {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        let signature = view.consume_n(2)?;
        assert_spec_eq!(signature, b"BM", "Not a BMP file");
        let filesize = U32::LE(&mut view)?;
        let reserved = U32::LE(&mut view)?;
        assert_spec_eq!(reserved.value, 0);
        let data_offset = U32::LE(&mut view)?;
        Ok((FileHeader { filesize, data_offset }, view.cursor.offset))
    }
}

#[derive(Debug)]
pub struct InfoHeader {
    pub size: U32,
    pub width: U32,
    pub height: U32,
    pub bit_count: U16,
    pub compression: InfoHeader_CompressionType,
    pub image_size: U32,
    pub x_pixels_per_m: U32,
    pub y_pixels_per_m: U32,
    pub colors_used: U32,
    pub colors_important: U32,
    pub color_table: Option<Vec<Color>>
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum InfoHeader_CompressionType {
    BI_RGB = 0,
    BI_RLE4 = 1,
    BI_RLE8 = 2,
}
impl Spec for InfoHeader_CompressionType {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        let spec = U32::LE(&mut view)?;
        Ok((match spec.value {
            0 => InfoHeader_CompressionType::BI_RGB,
            1 => InfoHeader_CompressionType::BI_RLE4,
            2 => InfoHeader_CompressionType::BI_RLE8,
            rest => return spec_error!("Unsupported compression type: {rest}")
        }, view.cursor.offset))
    }
}

impl Spec for InfoHeader {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        let size = U32::LE(&mut view)?;
        assert_spec!(size.value >= 36, "Size field is wrong");
        let width = U32::LE(&mut view)?;
        let height = U32::LE(&mut view)?;
        let planes =U16::LE(&mut view)?;
        assert_spec_eq!(planes.value, 1);
        let bit_count = U16::LE(&mut view)?;
        assert_spec_eq!(bit_count.value, 24, "non 24-bit BMP are not supported");
        assert_spec!(bit_count.value <= (usize::BITS as u16 - 1), "bits per pixel must be atleast {}", (usize::BITS as u16 - 1));
        let num_colors = if bit_count.value < 2 {1} else {(1 as usize) << bit_count.value};
        // enum defined above
        let compression = InfoHeader_CompressionType::read_from_view(&mut view, ())?;
        let image_size = U32::LE(&mut view)?;
        assert_spec!(image_size.value != 0 || compression == InfoHeader_CompressionType::BI_RGB, "The field image_size is zero but compression is used");
        let x_pixels_per_m = U32::LE(&mut view)?;
        let y_pixels_per_m = U32::LE(&mut view)?;
        let colors_used = U32::LE(&mut view)?;
        let colors_important = U32::LE(&mut view)?;
        let color_table = if bit_count.value <= 8 {
            Some(array![Color::read_from_view(&mut view, ()); num_colors]?)
        } else {None};
        Ok((InfoHeader { size, width, height, bit_count, compression, image_size, x_pixels_per_m, y_pixels_per_m, colors_used, colors_important, color_table },view.cursor.offset))
    }
}


// lifetime issues from here on
#[derive(Debug)]
pub struct RasterData {
    pub pixel_lines: Vec<Vec<Color>>
}

#[derive(Debug, Clone, Copy)]
pub struct RasterDataParams {
    pub width: U32,
    pub height: U32,
    pub bit_count: U16,
    pub compression: InfoHeader_CompressionType
}

impl From<&InfoHeader> for RasterDataParams {
    fn from(value: &InfoHeader) -> Self {
        Self { width: value.width, height: value.height, bit_count: value.bit_count, compression: value.compression }
    }
}

impl Spec for RasterData {
    type Params = RasterDataParams;
    fn read_all<S: ByteSource>(data: &S, params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        assert_spec!(
            params.bit_count.value == 24 && params.compression == InfoHeader_CompressionType::BI_RGB,
            "Only Truecolor Uncompressed images supported for now"
        );
        let pixel_lines = array![array![Color::read_from_view(&mut view, ()); params.width.value as usize]; params.height.value as usize]?;
        Ok((RasterData { pixel_lines },view.cursor.offset))
    }
}

#[derive(Debug)]
pub struct File {
    pub file_header: FileHeader,
    pub info_header: InfoHeader,
    pub raster_data: RasterData
}

impl Spec for File {
    type Params = ();
    fn read_all<S: ByteSource>(data: &S, _params: Self::Params) -> SResult<(Self, usize)> {
        let mut view = View::from(data);
        let file_header = FileHeader::read_from_view(&mut view, ())?;
        let info_header = InfoHeader::read_from_view(&mut view, ())?;
        let raster_data = RasterData::read_at(data, file_header.data_offset.value as usize, RasterDataParams::from(&info_header))?;
        Ok((File { file_header, info_header, raster_data }, view.cursor.offset))
    }
}

fn main() {
    default_main();
}