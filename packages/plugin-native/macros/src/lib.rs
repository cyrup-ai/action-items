use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

/// Attribute macro that generates the FFI export for a plugin
///
/// Example:
/// ```rust
/// #[plugin]
/// fn my_plugin() -> PluginBuilder {
///     PluginBuilder::new("my-plugin", "My Plugin").on_search(|query, ctx| async move {
///         // search logic
///     })
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let ident = &func.sig.ident;
    let wrapper = format_ident!("__{}_wrapper", ident);
    let vis = &func.vis;

    TokenStream::from(quote! {
        #func

        #vis fn #wrapper() -> Box<dyn ::action_items_plugin_native::ffi::LauncherPlugin> {
            #ident().build()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn _action_items_create_plugin() -> Box<dyn ::action_items_plugin_native::ffi::LauncherPlugin> {
            #wrapper()
        }
    })
}
