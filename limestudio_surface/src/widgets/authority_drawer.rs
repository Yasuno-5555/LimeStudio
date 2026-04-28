use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfaceWidget, SurfaceId, DisplaySignal, TreeNode, FrameStyle};
use dirtydata_dsp_fft::{FftProcessor, WindowType};


/// §SSS: Authority Drawer — The Accountable Truth.
/// "Persistent panel = accountable truth. 逃がすな。"
pub struct AuthorityDrawer {
    pub id: SurfaceId,
    pub position: Vec2,
    pub size: Vec2,
    pub is_open: bool,
    pub scroll_y: f32,
    /// Currently selected line index (1-based).
    pub selected_line: Option<usize>,
}

impl AuthorityDrawer {

    pub fn new(id: SurfaceId, screen_size: Vec2) -> Self {

        let width = 400.0;
        Self {
            id,
            position: Vec2::new(screen_size.x - width, 0.0),
            size: Vec2::new(width, screen_size.y),
            is_open: true,
            scroll_y: 0.0,
            selected_line: None,
        }
    }

    pub fn build_widget(
        &self, 
        node_id: SurfaceId, 
        node_name: &str, 
        node_state: Option<&dirtydata_runtime::nodes::base::NodeState>,
        code_fragment: Option<&crate::authority::visible_compiler::CodeFragment>,
    ) -> SurfaceWidget {
        use crate::ui_ir::SurfaceWidget::*;

        let mut children = vec![
            Label { text: format!("AUTHORITY: {}", node_name), is_secondary: false },
            Label { text: format!("ID: {:?}", node_id), is_secondary: true },
        ];

        // Code Section (Highest priority in Authority view)
        children.push(Label { text: "SOURCE CODE".to_string(), is_secondary: false });
        if let Some(frag) = code_fragment {
            children.push(CodeView {
                code: frag.source.clone(),
                language: frag.language.clone(),
            });
            children.push(Button {
                id: SurfaceId::from_seed(&format!("compile:{}", node_id.0)),
                label: "RECOMPILE (HOT RELOAD)".to_string(),
                is_active: true,
            });
        } else {

            children.push(Label { text: "// Source unavailable or in-flight...".to_string(), is_secondary: true });
        }
        
        children.push(Row { children: vec![] }); // Spacer

        // Analysis Section
        children.push(Label { text: "SIGNAL ANALYSIS".to_string(), is_secondary: false });
        
        // Time Domain (Waveform)
        let fft_size = 128;
        let mock_data: Vec<f32> = (0..fft_size).map(|i| {
            let s1 = (i as f32 * 0.2).sin() * 0.5;
            let s2 = (i as f32 * 0.5).sin() * 0.3;
            s1 + s2
        }).collect();
        
        children.push(Waveform {
            id: format!("wave:{}", node_id.0),
            data: mock_data.clone(),
        });

        // Frequency Domain (Spectrum) using dirtydata-dsp-fft
        let mut fft_proc = FftProcessor::new(fft_size, WindowType::Hann);
        let mut spectrum_complex = vec![rustfft::num_complex::Complex::default(); fft_size];
        fft_proc.forward(&mock_data, &mut spectrum_complex);
        
        // Only take the first half (Nyquist) and calculate magnitude
        let spectrum_data: Vec<f32> = spectrum_complex.iter()
            .take(fft_size / 2)
            .map(|c: &rustfft::num_complex::Complex<f32>| (c.norm() / fft_size as f32) * 5.0) // Scale for visibility
            .collect();


        children.push(Spectrum {
            id: format!("spec:{}", node_id.0),
            data: spectrum_data,
        });


        if let Some(state) = node_state {


            children.push(Label { text: "LIVE STATE".to_string(), is_secondary: false });
            
            match state {
                dirtydata_runtime::nodes::base::NodeState::Serialized(val) => {
                    if let Some(obj) = val.as_object() {
                        for (key, v) in obj {
                            let signal = if let Some(n) = v.as_f64() {
                                DisplaySignal::Linear(n as f32)
                            } else {
                                DisplaySignal::Static(0.0)
                            };

                            children.push(Row {
                                children: vec![
                                    Label { text: format!("{}:", key), is_secondary: true },
                                    Knob {
                                        id: SurfaceId::from_seed(&format!("knob:{}:{}", node_id.0, key)),
                                        label: "".to_string(),
                                        signal,
                                    }
                                ]
                            });

                        }
                    } else {
                        children.push(Label { text: format!("{:?}", val), is_secondary: true });
                    }
                }
                _ => {
                    children.push(Label { text: "Empty State".to_string(), is_secondary: true });
                }
            }
        }


        // Provenance Section
        children.push(Label { text: "PROVENANCE".to_string(), is_secondary: false });
        children.push(TreeView {
            id: SurfaceId::generate(),
            nodes: vec![
                TreeNode {
                    id: SurfaceId::generate(),
                    label: "Source: core_lib.rs".to_string(),
                    children: vec![],
                    is_expanded: false,
                },
                TreeNode {
                    id: SurfaceId::generate(),
                    label: "Verified by: Compiler Authority".to_string(),
                    children: vec![],
                    is_expanded: false,
                }
            ],
        });

        Box {
            style: crate::ui_ir::FrameStyle::AuthorityGlass,
            children: vec![
                Column { children }
            ],
        }
    }
}

