extern crate libc;
extern crate alloc;

use std::io::{IoResult};
use std::c_str::CString;
use std::ptr::{null, null_mut};

mod raw;

#[deriving(Copy, PartialEq, Eq, Show)]
pub enum Style {
    Nested,
    Expanded,
    Compact,
    Compressed
}

impl Style {
    fn to_int(&self) -> int {
        match self {
            &Style::Nested => 0,
            &Style::Expanded => 1,
            &Style::Compact => 2,
            &Style::Compressed => 3
        }
    }

    fn from_int(i: i32) -> Style {
        match i {
            0 => Style::Nested,
            1 => Style::Expanded,
            2 => Style::Compact,
            3 => Style::Compressed,
            _ => panic!("Unkown SASS_STYLE constant: {}", i)
        }
    }
}

pub struct Options {
    // Output style for the generated css code
    // A value from above SASS_STYLE_* constants
    pub output_style: Style,
    // If you want inline source comments
    pub source_comments: bool,
    // Path to source map file
    // Enables the source map generating
    // Used to create sourceMappingUrl
    pub source_map_file: Option<String>,
    // Treat source_string as sass (as opposed to scss)
    pub is_indented_syntax_src: bool,
    // Colon-separated list of paths
    // Semicolon-separated on Windows
    pub include_paths: String,
    // For the image-url Sass function
    pub image_path: String,
    // Precision for outputting fractional numbers
    pub precision: int
}

fn options_from_raw(opt: raw::sass_options) -> Options {
    fn to_bool(i: libc::c_int) -> bool {
        assert!(i == 0 || i == 1);
        if i == 0 { false } else { true }
    }

    fn to_string(s: *const libc::c_char) -> String {
        if (s.is_null()) {
            "".to_string()
        } else {
            unsafe { CString::new(s, false) }.to_string()
        }
    }

    let raw::sass_options {
        output_style,
        source_comments,
        source_map_file,
        omit_source_map_url,
        is_indented_syntax_src,
        include_paths,
        image_path,
        precision
    } = opt;

    let source_map_file = match to_bool(omit_source_map_url) {
        false => Some(to_string(source_map_file)),
        true => None
    };

    Options {
        output_style: Style::from_int(output_style),
        source_comments: to_bool(source_comments),
        source_map_file: source_map_file,
        is_indented_syntax_src: to_bool(is_indented_syntax_src),
        include_paths: to_string(include_paths),
        image_path: to_string(image_path),
        precision: precision as int
    }
}

fn options_to_raw(opt: Options) -> raw::sass_options {
    fn from_bool(b: bool) -> libc::c_int {
        match b {
            false => 0,
            true  => 1
        }
    }

    fn from_string(s: String) -> *const libc::c_char {
        raw_string(s)
    }

    let Options {
        output_style,
        source_comments,
        source_map_file,
        is_indented_syntax_src,
        include_paths,
        image_path,
        precision
    } = opt;

    let (omit_source_map_url, source_map_file) =
        match source_map_file {
            None => (false, "".to_string()),
            Some(file) => (true, file)
        };

    raw::sass_options {
        output_style: output_style.to_int() as i32,
        source_comments: from_bool(source_comments),
        source_map_file: from_string(source_map_file),
        omit_source_map_url: from_bool(omit_source_map_url),
        is_indented_syntax_src: from_bool(is_indented_syntax_src),
        include_paths: from_string(include_paths),
        image_path: from_string(image_path),
        precision: precision as i32
    }
}


fn raw_string(s: String) -> *const i8 {
    unsafe { s.to_c_str().into_inner() }
}

#[test]
fn test_convert_btween_options_representations() {
    // Emulates a CString, you are responsible for freeing this.
    let source_map_path = raw_string("/SourceMapPath".to_string());
    let include_paths = raw_string("/IncludePath".to_string());
    let image_path = raw_string("/ImagePath".to_string());

    let sass_options = raw::sass_options {
        output_style: 1,
        source_comments: 0,
        source_map_file: source_map_path,
        omit_source_map_url: 0,
        is_indented_syntax_src: 0,
        include_paths: include_paths,
        image_path: image_path,
        precision: 10
    };

    let options = options_from_raw(sass_options);
    assert_eq!(options.output_style, Style::Expanded);
    assert_eq!(options.source_comments, false);
    assert_eq!(options.source_map_file, Some("/SourceMapPath".to_string()));
    assert_eq!(options.is_indented_syntax_src, false);
    assert_eq!(options.include_paths, "/IncludePath".to_string());
    assert_eq!(options.image_path, "/ImagePath".to_string());
    assert_eq!(options.precision, 10);
    //assert_eq!(sass_options, options_to_raw(options));
}

pub struct StringContext {
    context: raw::sass_context
}

impl StringContext {
    pub fn new(source: String, output_path: String) -> StringContext {
        unsafe {
            let context = raw::sass_context {
                input_path: null(),
                output_path: null(),
                source_string: raw_string(source),
                output_string: null_mut(),
                options: options_to_raw(StringContext::default_options()),
                source_map_string: null_mut(),
                error_status: 0,
                error_message: null_mut(),
                c_functions: null_mut(),
                included_files: null_mut(),
                num_included_files: 0
            };

            StringContext { context: context }
        }
    }

    fn default_options() -> Options {
        Options {
            output_style: Style::Nested,
            source_comments: true,
            source_map_file: None,
            is_indented_syntax_src: false,
            include_paths: "/foo".to_string(),
            image_path: "/foo".to_string(),
            precision: 10
        }
    }

    pub fn compile(mut self) -> String {
        unsafe {
            raw::sass_compile(&mut self.context);
            assert_eq!(self.context.error_status, 0);
            CString::new(self.context.output_string, false).to_string()
        }
    }
}

impl Drop for StringContext {
    fn drop(&mut self) {
        // unsafe { raw::sass_free_context(self.context) };
    }
}

pub struct FileContext {
    context: *mut raw::sass_file_context
}

impl FileContext {
    fn new() -> FileContext {
        unsafe { FileContext { context: raw::sass_new_file_context() } }
    }
}

impl Drop for FileContext {
    fn drop(&mut self) {
        unsafe { raw::sass_free_file_context(self.context) };
    }
}

pub struct DirectoryContext {
    context:  *mut raw::sass_folder_context
}

impl DirectoryContext {
    fn new() -> DirectoryContext {
        unsafe { DirectoryContext { context: raw::sass_new_folder_context() } }
    }
}

impl Drop for DirectoryContext {
    fn drop(&mut self) {
        unsafe { raw::sass_free_folder_context(self.context) };
    }
}

pub fn from_string(s: String, output: String) -> IoResult<StringContext> {
    let result = StringContext::new(s, output);
    Ok(result)
}

pub fn from_path(p: Path) -> IoResult<FileContext> {
    let result = FileContext::new();
    Ok(result)
}

pub fn from_directory(p: Path) -> IoResult<DirectoryContext> {
    let result = DirectoryContext::new();
    Ok(result)
}

#[test]
fn test_from_string_should_produce_a_valid_context() {
    match from_string("#hello { color: red; }".to_string(), "/Users/jroesch/Desktop/foo.sass".to_string()) {
        Err(err) => panic!("Failed with: {}", err),
        Ok(context) => {
            assert_eq!("/* line 1, stdin */\n#hello {\n  color: red; }\n".to_string(), context.compile())
        }
    }
}
