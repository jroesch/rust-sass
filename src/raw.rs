use libc::{c_void, c_int, c_char};

#[repr(C)]
// Please ensure there are no null values.
// Thar be dragons.
pub struct sass_options {
  // Output style for the generated css code
  // A value from above SASS_STYLE_* constants
  pub output_style: c_int,
  // If you want inline source comments
  pub source_comments: c_int,
  // Path to source map file
  // Enables the source map generating
  // Used to create sourceMappingUrl
  pub source_map_file: *const c_char,
  // Disable sourceMappingUrl in css outputd
  pub omit_source_map_url: c_int,
  // Treat source_string as sass (as opposed to scss)
  pub is_indented_syntax_src: c_int,
  // Colon-separated list of paths
  // Semicolon-separated on Windows
  pub include_paths: *const c_char,
  // For the image-url Sass function
  pub image_path: *const c_char,
  // Precision for outputting fractional numbers
  pub precision: c_int
}

#[repr(C)]
pub struct sass_context {
  pub input_path: *const c_char,
  pub output_path: *const c_char,
  pub source_string: *const c_char,
  pub output_string: *mut c_char,
  pub source_map_string: *mut c_char,
  pub options: sass_options,
  pub error_status: c_int,
  pub error_message: *mut c_char,
  pub c_functions: *mut Sass_C_Function_Descriptor,
  pub included_files: *mut *mut c_char,
  pub num_included_files: c_int
}

#[repr(C)]
pub struct sass_file_context {
  input_path: *const c_char,
  output_path: *const c_char,
  output_string: *mut c_char,
  source_map_string: *mut c_char,
  options: sass_options,
  error_status: c_int,
  error_message: *mut char,
  c_functions: *mut Sass_C_Function_Descriptor,
  pub included_files: *mut *mut c_char,
  num_included_files: c_int
}

#[repr(C)]
pub struct sass_folder_context {
  search_path: *const c_char,
  output_path: *const c_char,
  options: sass_options,
  error_status: c_int,
  error_message: *mut c_char,
  c_functions: *mut c_void,
  included_files: *mut *mut c_char,
  num_included_files: c_int
}

type Sass_C_Function = extern fn (value: Sass_Value, cookie: *mut c_void) -> Sass_Value;
type Sass_Value = ();

#[repr(C)]
struct Sass_C_Function_Descriptor {
    signature: *const c_char,
    function: Sass_C_Function,
    void: *mut c_void
}

#[link(name = "sass")]
extern {
    pub fn sass_new_context() -> *mut sass_context;
    pub fn sass_new_file_context() -> *mut sass_file_context;
    pub fn sass_new_folder_context() -> *mut sass_folder_context;
    pub fn sass_free_context(ctx: *mut sass_context);
    pub fn sass_free_file_context(ctx: *mut sass_file_context);
    pub fn sass_free_folder_context(ctx: *mut sass_folder_context);
    pub fn sass_compile(ctx: *mut sass_context) -> c_int;
    pub fn sass_compile_file(ctx: *mut sass_file_context) -> c_int;
    pub fn sass_compile_folder(ctx: *mut sass_folder_context) -> c_int;
    pub fn quote(string: *const c_char, quotemark: *const c_char) -> *const c_char;
    pub fn unquote(string: *const c_char) -> *const c_char;
}
