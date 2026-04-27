use crate::model::stable_id::SurfaceId;
use limestudio_core::transaction::{TransactionMetadata, Author};
use glam::Vec2;

pub struct LineageTimeline {
    pub selected_node: Option<SurfaceId>,
    pub history: Vec<TransactionMetadata>,
}

impl LineageTimeline {
    pub fn new() -> Self {
        Self {
            selected_node: None,
            history: Vec::new(),
        }
    }

    pub fn to_widget(&self) -> crate::ui_ir::SurfaceWidget {
        use crate::ui_ir::SurfaceWidget::*;

        if self.history.is_empty() {
            return Label { text: "No History".to_string(), is_secondary: true };
        }

        let mut children = Vec::new();
        children.push(Label { text: "PATCH LINEAGE".to_string(), is_secondary: false });

        for (i, meta) in self.history.iter().rev().enumerate() {
            let color_name = match meta.author {
                Author::User => "User",
                Author::HostAutomation(_) => "Auto",
                Author::Script(_) => "Script",
                Author::System => "System",
            };

            let row = Row {
                children: vec![
                    Label { text: format!("#{} ", self.history.len() - i), is_secondary: true },
                    Label { text: color_name.to_string(), is_secondary: false },
                ]
            };
            children.push(row);

            let time_str = meta.timestamp.format("%H:%M:%S").to_string();
            children.push(Label { text: time_str, is_secondary: true });
            children.push(Label { text: format!("{:?}", meta.intent), is_secondary: true });
        }

        Column { children }
    }
}
