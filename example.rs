struct Color { // specs are flexible, but structs make our job easier by not having to check the size of ifs and fors.
    Red: u8,
    Green: u8,
    Blue: u8,
    Reserved: u8
    assert(Reserved == 0, "Reserved color part is not 0, Unsupported.");
} // size calculated just like how rust calculates it.

spec FileHeader {
    Signature: [u8; 2]; // length constant
    assert(signature == 'BM', "Not a BMP file"); // '' = bytes "" = unicode encoded
    Filesize: u32;
    Reserved: u32;
    assert(reserved == 0, "Unsupported new feature (reserved field set)");
    DataOffset: u32;
}

spec InfoHeader {
    Size: u32;
    Width: u32;
    Height: u32;
    Planes: u16;
    assert(planes == 1, "Multiple planes not supported");
    BitCount: u16;
    let num_colors = if BitCount < 2 {1} else {1 << BitCount};
    Compression: enum<u32> {
        0 => BI_RGB,
        1 => BI_RLE8,
        2 => BI_RLE4,
        _ => error("Unsupported new feature (field in invalid state)")
    }
    use Compression::BI_RGB;
    ImageSize: u32;
    assert(ImageSize != 0 || Compression == Compression::BI_RGB, "ImageSize undefined");
    XpixelsPerM: u32;
    YpixelsPerM: u32;
    ColorsUsed: u32;
    ColorsImportant: u32;
    if BitCount <= 8 { // if without else, so size can be either 0 or size of the block
        ColorTable: [Color; num_colors]; // size = size(color) * num_colors
    } // size = size(block) or 0; = size(color) * num_colors or 0.
    assertsize(Size); // we can't allow unsafe use of Size without checking, we can however check it
} // size is sum of sizes of all elements

// syntax change, values are passed like functions
spec RasterData(InfoHeader: InfoHeader) { // generic value, fields can be used freely. i.e. dependency on a generic doesn't make this variable size
    use InfoHeader::Compression::BI_RGB;
    assert(
        InfoHeader.BitCount == 24 && InfoHeader.Compression == BI_RGB,
        "Only Truecolor Uncompressed images supported for now"
    ); // TODO
    PixelLines: [[Color; InfoHeader.Width];InfoHeader.Height]; // constant size because fields from a generic value are used, not from this spec itself
}

spec File {
    FileHeader: FileHeader;
    assertsize(FileHeader.Filesize);
    InfoHeader: InfoHeader; // size calcualted manually
    RasterData: RasterData(InfoHeader=InfoHeader) @FileHeader.DataOffset; // setting an offset tells us that the size is atleast size(before) + offset.
}