use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

///Function macro to track timing information of a function. Any provided attributes will be the key to obtain the [`TrackInfo`], otherwise the name in the function signature is used.
/// 
/// # Examples
    ///
    /// ```no_run
    /// use track::track;
    /// 
    /// #[track]
    /// fn foo() {
    ///     /.../
    /// }
    /// 
    /// #[track(bar)]
    /// fn foo_bar() {
    ///     /.../
    /// }
    ///
    /// fn main()  {
    ///     for _ in 0..10 {
    ///         foo();
    ///     }
    /// 
    ///     let t = track::get("foo").unwrap();
    /// 
    ///     assert_eq!(t.calls, 10);
    /// 
    ///     for _ in 0..20 {
    ///         foo_bar();
    ///     }
    /// 
    ///     let t = track::get_or_default("bar");
    ///     assert_eq!(t.calls, 20);
    /// }
    /// ```
#[proc_macro_attribute]
pub fn track(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    expand(attr.to_string(), input).into()
}

fn expand(attr: String, f: ItemFn) -> TokenStream {
    let attrs = &f.attrs;
    let vis = &f.vis;
    let sig = &f.sig;
    let block = &f.block;
    let name = sig.ident.to_string();
    let name = if attr.is_empty() {name.as_str()} else {attr.as_str()};
    
    
    let t = quote!  {
        #(#attrs)*
        #vis #sig {
            
            let __start = ::std::time::Instant::now();
            let __result = (|| #block)();
            let __end = __start.elapsed();
            ::track::record(#name, __end);

            __result
        }
    };
    t.into()

}