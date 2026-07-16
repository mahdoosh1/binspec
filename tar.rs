spec TarString(size: Int) {
    // [to_be_called; how_many_times]
    // [type; integer]
    // [|| TypeThatHasParameters(2); 5] is allowed
    // if rust has something that calls a function n times and returns array of return values, that should be used instead of this syntax
    hide let string = [U8; size-1]; // if rust has string type for ascii, that type should be used instead of arrays
    // call a type means -> read and return value of that type
    hide let end = U8();
    assert!(end == 0 || end == b' '); // not rust's assert
    let text = for i in string { // in this language for returns Option, None if broke without a value or if loop finished, Some if broke with a value
        if i != 0 && i != b' ' {break string}
    };
}

spec Oct(size = 8) { // implicitly of type Int
    hide let digits = TarString(size).text;
    let value = digits.and_then((digits) => {parse_int(digits, 8)});
}

spec Header() {
    hide let file_path_and_name = TarString(100).text;
    let file_mode = Oct().value;
    let UID = Oct().value;
    let GID = Oct().value;
    let file_size = Oct(12).value;
    let last_modified = Oct(12).value;
    let check_sum = Oct();
    enum FileType: U8 {
        b'0' | 0 => NormalFile,
        b'1' => HardLink,
        b'2' => SymbolicLink,
        b'3' => CharacterSpecial,
        b'4' => BlockSpecial,
        b'5' => Directory,
        b'6' => FIFO,
        b'7' => ContiguousFile,
        b'g' => GlobalExtendedHeaderWithMetadata,
        b'x' => ExtendedHeaderWithMetadataForTheNextFileInTheArchive,
        b'A'..=b'Z' as a_to_z => VendorSpecificExtention(a_to_z),
        rest => Reserved(rest)
    }
    let file_type = FileType();
    assert(file_type != FileType::Reserved, "Reserved filetype used");
    let name_of_linked_file = TarString(100).text;
    let unused = Try( // Try will use the first type, if it fails, it uses the second type
        spec { // anonymous spec
            let ustar_indicator = TarString(6).text;
            let ustar_version = TarString(2).text;
            let owner_user_name = TarString(32).text;
            let owner_group_name = TarString(32).text;
            let device_major_number = Oct().value;
            let device_minor_number = Oct().value;
            let filename_prefix = TarString(155).text;
            let unspecified = [U8; 12];
        },
        [U8; 255]
    ); // Try returns an enum with variants `Ok(type)` or `Fallback(type, error)`.
    // `value as type` -> take `value` as unparsed bytes and parse them as `type`
    assert(
        sum(self as [U8])
        - sum(check_sum as [U8]) + (check_sum.size * b' ') // sum of all headers with CheckSum replaced as spaces.
        == check_sum.value
    );
    // for use outside:
    let filename = match unused {
        Ok(ustar) => {
            if ustar.ustar_indicator != b"ustar" {
                break file_path_and_name
            }
            assert(ustar.ustar_version == b"00", "This file uses a newer version of UStar that is not yet supported");
            uStar.filename_prefix + file_path_and_name
        }
        Fallback => {
            file_path_and_name
        }
    }
}

spec TarEntry() {
    let header = Header();
    let file_data = [U8; Header.FileSize];
}
spec File() {
    let entries = [[0; Header.size]? TarEntry];
}