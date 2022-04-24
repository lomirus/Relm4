use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{Error, Path, Type, Visibility};

use crate::widgets::{PropertyType, ReturnedWidget, TopLevelWidget, ViewWidgets, Widget};

#[derive(Default)]
pub(crate) struct TokenStreams {
    /// Initialize the root widget.
    pub init_root: TokenStream2,
    /// Rename root to the actual widget name.
    pub rename_root: TokenStream2,
    /// The tokens for the struct fields -> name: Type,
    pub struct_fields: TokenStream2,
    /// The tokens initializing the widgets.
    pub init: TokenStream2,
    /// The tokens initializing the properties.
    pub assign: TokenStream2,
    /// The tokens for connecting events.
    pub connect: TokenStream2,
    /// The tokens for the returned struct fields -> name,
    pub return_fields: TokenStream2,
    /// For destructuring the widget struct field
    pub destructure_fields: TokenStream2,
    /// The view tokens (watch! macro)
    pub update_view: TokenStream2,
}

impl ViewWidgets {
    pub fn generate_streams(
        &self,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
        standalone_view: bool,
    ) -> TokenStreams {
        let mut streams = TokenStreams::default();

        for top_level_widget in &self.top_level_widgets {
            top_level_widget.generate_streams(
                &mut streams,
                vis,
                model_type,
                relm4_path,
                standalone_view,
            );
        }

        streams
    }

    /// Generate root type for `Root` parameter in `Component` impl
    pub fn root_type(&self) -> TokenStream2 {
        for top_level_widget in &self.top_level_widgets {
            if top_level_widget.root_attr.is_some() {
                return top_level_widget.inner.func_type_token_stream();
            }
        }
        Error::new(
            self.span,
            "You need to specify the root widget using the `#[root]` attribute.",
        )
        .to_compile_error()
    }
}

impl TopLevelWidget {
    fn generate_streams(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
        standalone_view: bool,
    ) {
        self.inner.init_token_generation(
            streams,
            vis,
            model_type,
            relm4_path,
            !standalone_view && self.root_attr.is_some(),
        );
    }
}

impl Widget {
    pub(super) fn init_token_generation(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
        generate_init_root_stream: bool,
    ) {
        let name = &self.name;
        let name_span = name.span();

        // Initialize the root
        if generate_init_root_stream {
            // For the `component` macro
            self.init_stream(&mut streams.init_root);
            name.to_tokens(&mut streams.init_root);
        } else {
            // For the `view!` macro
            self.init_stream(&mut streams.init);
        }

        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.return_stream(&mut streams.return_fields);
        self.destructure_stream(&mut streams.destructure_fields);

        // Rename the `root` to the actual widget name
        streams.rename_root.extend(quote_spanned! {
            name_span => let #name = root.clone();
        });

        for prop in &self.properties.properties {
            prop.init_stream(&mut streams.init);
            prop.assign_stream(&mut streams.assign, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);
            prop.return_stream(&mut streams.return_fields);
            prop.destructure_stream(&mut streams.destructure_fields);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
            }
        }
    }

    fn generate_component_tokens_recursively(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) {
        for prop in &self.properties.properties {
            prop.init_stream(&mut streams.init);
            prop.assign_stream(&mut streams.assign, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);
            prop.return_stream(&mut streams.return_fields);
            prop.destructure_stream(&mut streams.destructure_fields);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);

                if let Some(returned_widget) = &widget.returned_widget {
                    returned_widget.generate_component_tokens_recursively(
                        streams, vis, model_type, relm4_path,
                    );
                }
            }
        }
    }
}

impl ReturnedWidget {
    fn generate_component_tokens_recursively(
        &self,
        streams: &mut TokenStreams,
        vis: &Option<Visibility>,
        model_type: &Type,
        relm4_path: &Path,
    ) {
        self.struct_fields_stream(&mut streams.struct_fields, vis);
        self.return_stream(&mut streams.return_fields);
        self.destructure_stream(&mut streams.destructure_fields);

        for prop in &self.properties.properties {
            prop.init_stream(&mut streams.init);
            prop.assign_stream(&mut streams.assign, &self.name, relm4_path);
            prop.connect_signals_stream(&mut streams.connect, &self.name, relm4_path);
            prop.update_view_stream(&mut streams.update_view, &self.name, relm4_path);

            prop.struct_fields_stream(&mut streams.struct_fields, vis, relm4_path);
            prop.return_stream(&mut streams.return_fields);
            prop.destructure_stream(&mut streams.destructure_fields);

            if let PropertyType::Widget(widget) = &prop.ty {
                widget.generate_component_tokens_recursively(streams, vis, model_type, relm4_path);
            }
        }
    }
}
