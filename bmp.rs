spec Color() {
    let red = U8();
    let green = U8();
    let blue = U8();
    hide let reserved = U8();
    assert(reserved == 0, "Reserved color part is not 0, Unsupported.");
}

spec FileHeader() {
    hide let signature = [U8; 2];
    assert(signature == b"BM", "Not a BMP file");
    let filesize = U32();
    hide let reserved = U32();
    assert(reserved == 0, "Unsupported new feature (reserved field set)");
    let data_offset = U32();
}

spec InfoHeader() {
    let size = U32();
    setsize(size);
    let width = U32();
    let height = U32();
    hide let planes = U16();
    assert(planes == 1, "Multiple planes not supported");
    let bit_count = U16();
    let num_colors = if bit_count < 2 {1} else {1 << bit_count};
    enum ComperssionType: U32 {
        0 => BI_RGB,
        1 => BI_RLE4,
        2 => BI_RLE8,
        rest => error("Unsupported compression type: {}", rest)
    }
    let compression = ComperssionType();
    let image_size = U32();
    assert(image_size != 0 || compression == CompressionType::BI_RGB, "The field image_size is zero but compression is used");
    let x_pixels_per_m = U32();
    let y_pixels_per_m = U32();
    let colors_used = U32();
    let colors_important = U32();
    if bit_count <= 8 {
        let color_table = [Color; num_colors];
    }
}

spec RasterData(info_header: InfoHeader) {
    assert(
        info_header.bit_count == 24 && info_header.compression == InfoHeader::ComperssionType::BI_RGB,
        "Only Truecolor Uncompressed images supported for now"
    );
    let pixel_lines = [[Color; info_header.width]; info_header.height];
}

spec File {
    let file_header = FileHeader();
    assertsize(FileHeader.Filesize);
    let info_header = InfoHeader();
    let raster_data @file_header.data_offset = RasterData(info_header);

}