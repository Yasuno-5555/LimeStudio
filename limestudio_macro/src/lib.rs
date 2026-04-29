extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Block, Ident, LitStr, Result, Token,
};

struct PluginDefinition {
    name: String,
    vendor: String,
    state_version: u32,
    migrations: Vec<Ident>,
    params: Vec<ParamDefinition>,
    dsp_param_name: Ident,
    dsp_body: Block,
    ui_param_name: Ident,
    ui_body: Block,
}

struct ParamDefinition {
    name: Ident,
    _ty: Ident,
    default: f32,
    range: (f32, f32),
}

impl Parse for PluginDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = String::new();
        let mut vendor = String::new();
        let mut state_version = 0;
        let mut migrations = Vec::new();
        let mut params = Vec::new();
        let mut dsp_param_name = None;
        let mut dsp_body = None;
        let mut ui_param_name = None;
        let mut ui_body = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = lit.value();
                }
                "vendor" => {
                    let lit: LitStr = input.parse()?;
                    vendor = lit.value();
                }
                "state_version" => {
                    let lit: syn::LitInt = input.parse()?;
                    state_version = lit.base10_parse()?;
                }
                "migration" => {
                    let content;
                    bracketed!(content in input);
                    while !content.is_empty() {
                        migrations.push(content.parse()?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "params" => {
                    let content;
                    syn::braced!(content in input);
                    while !content.is_empty() {
                        let p_name: Ident = content.parse()?;
                        content.parse::<Token![:]>()?;
                        let p_ty: Ident = content.parse()?;
                        content.parse::<Token![=]>()?;
                        let p_default: syn::LitFloat = content.parse()?;

                        let range_content;
                        bracketed!(range_content in content);
                        let start: syn::LitFloat = range_content.parse()?;
                        range_content.parse::<Token![..]>()?;
                        let end: syn::LitFloat = range_content.parse()?;

                        params.push(ParamDefinition {
                            name: p_name,
                            _ty: p_ty,
                            default: p_default.base10_parse()?,
                            range: (start.base10_parse()?, end.base10_parse()?),
                        });

                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "dsp" => {
                    input.parse::<Token![|]>()?;
                    let p: Ident = input.parse()?;
                    input.parse::<Token![|]>()?;
                    dsp_param_name = Some(p);
                    dsp_body = Some(input.parse()?);
                }
                "ui" => {
                    input.parse::<Token![|]>()?;
                    let p: Ident = input.parse()?;
                    input.parse::<Token![|]>()?;
                    ui_param_name = Some(p);
                    ui_body = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown key in plugin!: {}", key),
                    ))
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PluginDefinition {
            name,
            vendor,
            state_version,
            migrations,
            params,
            dsp_param_name: dsp_param_name
                .ok_or_else(|| syn::Error::new(input.span(), "Missing dsp block"))?,
            dsp_body: dsp_body.ok_or_else(|| syn::Error::new(input.span(), "Missing dsp body"))?,
            ui_param_name: ui_param_name
                .ok_or_else(|| syn::Error::new(input.span(), "Missing ui block"))?,
            ui_body: ui_body.ok_or_else(|| syn::Error::new(input.span(), "Missing ui body"))?,
        })
    }
}

#[proc_macro]
pub fn plugin(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as PluginDefinition);

    let plugin_name_ident = quote::format_ident!("{}", def.name.replace(" ", ""));
    let params_struct_ident = quote::format_ident!("{}Params", plugin_name_ident);
    let ui_params_struct_ident = quote::format_ident!("{}UiParams", plugin_name_ident);

    let param_fields = def.params.iter().map(|p| {
        let name = &p.name;
        let name_str = name.to_string();
        quote! {
            #[id = #name_str]
            pub #name: FloatParam
        }
    });

    let param_init = def.params.iter().map(|p| {
        let name = &p.name;
        let label = name.to_string();
        let default = p.default;
        let (min, max) = p.range;
        quote! {
            #name: FloatParam::new(
                #label,
                #default,
                FloatRange::Linear { min: #min, max: #max },
            )
        }
    });

    let ui_param_fields = def.params.iter().map(|p| {
        let name = &p.name;
        quote! { pub #name: UiParam<'a> }
    });

    let ui_param_init = def.params.iter().map(|p| {
        let name = &p.name;
        let name_str = name.to_string();
        quote! {
            #name: UiParam {
                id: #name_str,
                param: &params.#name
            }
        }
    });

    let dsp_param_fields = def.params.iter().map(|p| {
        let name = &p.name;
        quote! { pub #name: ParamSource }
    });

    let dsp_param_init = def.params.iter().map(|p| {
        let name = &p.name;
        let name_str = name.to_string();
        quote! { #name: ParamSource::Parameter(#name_str.to_string()) }
    });

    let _param_sync = def.params.iter().map(|p| {
        let name = &p.name;
        let name_str = name.to_string();
        quote! {
            if let Some(prod) = &mut self.patch_producer {
                let _ = prod.push(PatchEvent::SetParameter {
                    param_id: #name_str.to_string(),
                    value: self.params.#name.value(),
                });
            }
        }
    });

    let _param_dump = def.params.iter().map(|p| {
        let name = &p.name;
        let name_str = name.to_string();
        quote! {
            let _ = writeln!(writer, "  {}: {}", #name_str, self.#name.value());
        }
    });

    let name = def.name;
    let vendor = def.vendor;
    let dsp_param_name = def.dsp_param_name;
    let dsp_body = def.dsp_body;
    let ui_param_name = def.ui_param_name;
    let ui_body = def.ui_body;
    let dsp_body_str = quote!(#dsp_body).to_string();
    let state_version = def.state_version;
    let migrations = &def.migrations;
    let _migration_indices = 0..migrations.len();

    let internal_mod_name = quote::format_ident!("__limestudio_plugin_{}", plugin_name_ident);

    let expanded = quote! {
        mod #internal_mod_name {
            use super::*;
            use nih_plug::prelude::*;
            use std::sync::Arc;
            use std::cell::RefCell;
            use limestudio_plugin::dsl::*;
            use limestudio_core::graph::{GraphBuilder, StableId, ParamSource};
            use limestudio_plugin::dirtydata_core as dirtydata;
            use limestudio_plugin::rtrb::{RingBuffer, Producer};
            use limestudio_core::engine::{VoiceManager, VoicePlan, VoiceEvent};
            use limestudio_core::PatchEvent;
            use limestudio_plugin::ui::{Knob, Slider, Toggle, NumberBox, Button, ListView, Envelope, Lens, Label, Badge, UiParam};
            use limestudio_plugin::observation::{self, ObservationProducer, ObservationConsumer, PeakMonitor};
            use limestudio_plugin::Widget;
            use limestudio_plugin::editor::SurfaceEditor;

            #[derive(Params)]
            pub struct #params_struct_ident {
                pub state_version: IntParam,
                #(#param_fields,)*
            }

            impl Default for #params_struct_ident {
                fn default() -> Self {
                    Self {
                        state_version: IntParam::new("State Version", #state_version as i32, IntRange::Linear { min: 0, max: 1000 }).hide(),
                        #(#param_init,)*
                    }
                }
            }

            pub struct #ui_params_struct_ident<'a> {
                #(#ui_param_fields,)*
            }

            pub struct DspBuilderCtx<'a> {
                pub input: Chainable<'a>,
                pub output: NodeHandle,
                pub Pitch: ParamSource,
                pub Velocity: ParamSource,
                pub Pressure: ParamSource,
                pub Tuning: ParamSource,
                pub Gate: ParamSource,
                #(#dsp_param_fields,)*
            }

            pub struct #plugin_name_ident {
                pub params: Arc<#params_struct_ident>,
                pub dsp_source_code: String,
            }

            impl Default for #plugin_name_ident {
                fn default() -> Self {
                    Self {
                        params: Arc::new(#params_struct_ident::default()),
                        dsp_source_code: #dsp_body_str.to_string(),
                    }
                }
            }

            impl LimeProcessor for #plugin_name_ident {
                type Params = #params_struct_ident;
                const NAME: &'static str = #name;
                const VENDOR: &'static str = #vendor;
                const URL: &'static str = "https://limestudio.dev";
                const EMAIL: &'static str = "info@limestudio.dev";
                const VERSION: &'static str = "0.1.0";

                fn params(&self) -> Arc<Self::Params> {
                    self.params.clone()
                }

                fn initialize(&mut self, sample_rate: f32) {
                    // Logic moved to Adapter/Core if possible, or kept here if plugin-specific
                }

                fn build_graph(&self, params: Arc<Self::Params>, builder: &RefCell<GraphBuilder>) {
                    let (input_node, output_node) = {
                        let b = builder.borrow();
                        (b.input_node(), b.output_node())
                    };

                    let input = Chainable {
                        current_node: input_node,
                        builder: builder,
                    };
                    let output = NodeHandle { id: output_node };

                    let mut #dsp_param_name = DspBuilderCtx {
                        input,
                        output,
                        Pitch: ParamSource::Parameter("pitch".to_string()),
                        Velocity: ParamSource::Parameter("velocity".to_string()),
                        Pressure: ParamSource::Parameter("pressure".to_string()),
                        Tuning: ParamSource::Parameter("tuning".to_string()),
                        Gate: ParamSource::Parameter("gate".to_string()),
                        #(#dsp_param_init,)*
                    };

                    #[allow(non_snake_case)]
                    let Gain = |p: ParamSource| {
                        let id = builder.borrow_mut().add_processor("gain", vec![("gain", p)]);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let Sine = |p: ParamSource| {
                        let id = builder.borrow_mut().add_processor("Sine", vec![("frequency", p)]);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let Oscillator = |p: ParamSource, wave: &str| {
                        let id = builder.borrow_mut().add_processor("Oscillator", vec![
                            ("frequency", p),
                            ("waveform", ParamSource::Constant(0.0)), // FIXME: Handle string or enum
                        ]);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let ADSR = |a: ParamSource, d: ParamSource, s: ParamSource, r: ParamSource, gate: ParamSource| {
                        let id = builder.borrow_mut().add_processor("Envelope", vec![
                            ("attack", a), ("decay", d), ("sustain", s), ("release", r), ("gate", gate)
                        ]);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let Multiply = |a: NodeHandle, b: NodeHandle| {
                        let id = builder.borrow_mut().add_processor("Multiply", vec![]);
                        builder.borrow_mut().connect(a.id, id);
                        builder.borrow_mut().connect(b.id, id);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let Add = |a: NodeHandle, b: NodeHandle| {
                        let id = builder.borrow_mut().add_processor("Add", vec![]);
                        builder.borrow_mut().connect(a.id, id);
                        builder.borrow_mut().connect(b.id, id);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let m2f = |p: ParamSource, t: ParamSource| {
                        let id = builder.borrow_mut().add_processor("MidiToHz", vec![("pitch", p), ("tuning", t)]);
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let SineVCO = |freq_node: NodeHandle| {
                        let id = builder.borrow_mut().add_processor("Sine", vec![]);
                        builder.borrow_mut().add_edge(freq_node.id, "out", id, "frequency");
                        NodeHandle { id }
                    };

                    #[allow(non_snake_case)]
                    let Filter = |p: ParamSource, q: ParamSource, kind: &str| {
                        let id = builder.borrow_mut().add_processor("Filter", vec![
                            ("frequency", p),
                            ("q", q),
                            // ("type", kind.into()) // FIXME: String config support
                        ]);
                        NodeHandle { id }
                    };

                    let _ = { #dsp_body };
                }

                fn build_ui(params: &#params_struct_ident, _obs_consumer: ObservationConsumer) -> Box<dyn Widget + '_> {
                    let #ui_param_name = #ui_params_struct_ident {
                        #(#ui_param_init,)*
                    };
                    Box::new({ #ui_body }) as Box<dyn Widget + '_>
                }
            }

            // The only framework-specific export
            nih_export_vst3!(limestudio_plugin::LimeAdapter<#plugin_name_ident>);
            nih_export_clap!(limestudio_plugin::LimeAdapter<#plugin_name_ident>);
        }

        pub use #internal_mod_name::#plugin_name_ident;
    };

    expanded.into()
}
