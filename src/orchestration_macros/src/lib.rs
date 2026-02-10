// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, ItemStruct, LitStr, Token};

/// Macro to generate `extern "C"` FFI declarations for a C++ class.
///
/// This macro declares external C functions to interact with a C++ object via FFI.
///
/// # Usage
///
/// ```ignore
/// #[import_from_cpp_ffi("method1", "method2")]
/// pub struct MyClass;
/// ```
///
/// This generates:
/// - `create_MyClass() -> *mut c_void`
/// - `free_MyClass(ptr: *mut c_void)`
/// - `method1_MyClass(ptr: *mut c_void)`
/// - `method2_MyClass(ptr: *mut c_void)`
///
/// # Parameters
/// - `attr`: A comma-separated list of method names (as string literals).
/// - `item`: A Rust `struct` item to which the methods belong.
///
/// # Requirements
/// The C++ side must provide C bindings for these functions using the macro
/// EXPOSE_OBJECT_TO_ORCHESTRATION()
///
/// This macro does not generate any Rust wrapper logic. It only provides raw FFI bindings.
/// User need to implement Rust struct and methods to call these C++ methods via FFI.
/// Additionally, user shall implement new() method to initialize the struct and allocate memory for the C++ object and
/// Drop trait to free the memory when the struct goes out of scope.
/// The functions `create_<struct_name>()` and `free_<struct_name>()` are generated to handle memory allocation and deallocation.
///
#[proc_macro_attribute]
pub fn import_from_cpp_ffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let class_ident = &input_struct.ident;

    // Parse attribute arguments: #[import_from_cpp_ffi("fn1", "fn2", ...)]
    let method_lits = parse_macro_input!(attr with Punctuated::<LitStr, Token![,]>::parse_terminated);

    // Generate extern function declarations
    let create_fn = syn::Ident::new(&format!("create_{}", class_ident), class_ident.span());
    let free_fn = syn::Ident::new(&format!("free_{}", class_ident), class_ident.span());

    let method_decls = method_lits.iter().map(|lit| {
        let method_name = lit.value();
        let extern_fn_ident = syn::Ident::new(&format!("{}_{}", method_name, class_ident), lit.span());
        quote! {
            pub fn #extern_fn_ident(ptr: *mut c_void);
        }
    });

    let expanded = quote! {
        use std::ffi::c_void;

        extern "C" {
            pub fn #create_fn() -> *mut c_void;
            pub fn #free_fn(ptr: *mut c_void);
            #(#method_decls)*
        }
    };

    TokenStream::from(expanded)
}

/// Macro to generate a Rust struct that wraps C++ methods exposed via C FFI.
///
/// This macro declares:
/// - FFI bindings to the C++ object methods
/// - A Rust struct with:
///   - `new()` constructor that calls `create_<Struct>()`
///   - Rust methods that call the corresponding C functions
///   - `Drop` implementation that calls `free_<Struct>()`
///
/// # Usage
/// ```ignore
/// #[import_from_cpp("method1", "method2")]
/// pub struct MyClass;
/// ```
///
/// This expands into a Rust struct like:
/// ```ignore
/// pub struct MyClass {
///     ptr: *mut c_void,
/// }
///
/// impl MyClass {
///     pub fn new() -> Self { ... }
///     pub fn method1(&mut self) -> InvokeResult { ... }
///     pub fn method2(&mut self) -> InvokeResult { ... }
/// }
///
/// impl Drop for MyClass {
///     fn drop(&mut self) { ... }
/// }
/// ```
///
/// # Parameters
/// - `attr`: A comma-separated list of method names (as string literals).
/// - `item`: A Rust `struct` item to generate methods for.
///
/// # Requirements
/// The C++ side must provide C bindings for these functions using the macro
/// EXPOSE_OBJECT_TO_ORCHESTRATION()
///
#[proc_macro_attribute]
pub fn import_from_cpp(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse struct
    let input_struct = parse_macro_input!(item as ItemStruct);
    let class_ident = &input_struct.ident;

    // Parse attribute arguments: #[import_from_cpp("fn1", "fn2", ...)]
    let method_lits = parse_macro_input!(attr with Punctuated::<LitStr, Token![,]>::parse_terminated);

    // Generate extern function declarations
    let create_fn = syn::Ident::new(&format!("create_{}", class_ident), class_ident.span());
    let free_fn = syn::Ident::new(&format!("free_{}", class_ident), class_ident.span());

    let extern_method_decls = method_lits.iter().map(|lit| {
        let method_name = lit.value();
        let extern_fn_ident = syn::Ident::new(&format!("{}_{}", method_name, class_ident), lit.span());
        quote! {
            pub fn #extern_fn_ident(ptr: *mut c_void);
        }
    });

    let rust_method_definitions = method_lits.iter().map(|lit| {
        let method_name = lit.value();
        let method_ident = syn::Ident::new(method_name.as_str(), lit.span());
        let fn_ident = syn::Ident::new(&format!("{}_{}", method_name, class_ident), lit.span());
        quote! {
            pub fn #method_ident(&mut self) -> InvokeResult {
                unsafe {
                    #fn_ident(self.ptr);
                }
                Ok(())
            }
        }
    });

    let expanded = quote! {
        use std::ffi::c_void;

        extern "C" {
            pub fn #create_fn() -> *mut c_void;
            pub fn #free_fn(ptr: *mut c_void);
            #(#extern_method_decls)*
        }

        use orchestration::actions::invoke::InvokeResult;
        unsafe impl Send for #class_ident {}
        pub struct #class_ident {
            ptr: *mut c_void,
        }

        impl #class_ident {
            pub fn new() -> Self {
                Self {
                    ptr: unsafe { #create_fn() },
                }
            }
            #(#rust_method_definitions)*
        }

        impl Drop for #class_ident {
            fn drop(&mut self) {
                unsafe {
                    #free_fn(self.ptr);
                }
            }
        }
    };

    TokenStream::from(expanded)
}
