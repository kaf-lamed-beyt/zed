use crate::ImageView;
use anyhow;
use gpui::{div, AppContext, IntoElement, ParentElement, Render, Subscription, View, ViewContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsSources};
use ui::{prelude::*, Button, LabelSize};
use workspace::{ItemHandle, StatusItemView, Workspace};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImageFileSizeUnitType {
    #[default]
    Binary,
    Decimal,
}

impl Settings for ImageFileSizeUnitType {
    const KEY: Option<&'static str> = Some("image_file_unit_type");

    type FileContent = Self;

    fn load(
        sources: SettingsSources<Self::FileContent>,
        _: &mut AppContext,
    ) -> Result<Self, anyhow::Error> {
        sources.json_merge().or_else(|_| Ok(Self::Binary))
    }
}

pub struct ImageInfo {
    width: Option<u32>,
    height: Option<u32>,
    file_size: Option<u64>,
    format: Option<String>,
    color_type: Option<String>,
    _observe_active_image: Option<Subscription>,
}

impl ImageInfo {
    pub fn new(_workspace: &Workspace, cx: &mut AppContext) -> Self {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            ImageFileSizeUnitType::register(cx);
        });

        Self {
            width: None,
            height: None,
            file_size: None,
            format: None,
            color_type: None,
            _observe_active_image: None,
        }
    }

    fn update_metadata(&mut self, image_view: &View<ImageView>, cx: &mut ViewContext<Self>) {
        let image_item = image_view.read(cx).image_item.read(cx);

        if let Some(meta) = &image_item.image_meta {
            self.width = Some(meta.width);
            self.height = Some(meta.height);
            self.file_size = Some(meta.file_size);
            self.format = Some(meta.format.clone());
            self.color_type = Some(meta.color_type.to_string());
        } else {
            self.width = None;
            self.height = None;
            self.file_size = None;
            self.format = None;
            self.color_type = None
        }
        cx.notify();
    }

    fn format_file_size(&self, size: u64, image_unit_type: &ImageFileSizeUnitType) -> String {
        match image_unit_type {
            ImageFileSizeUnitType::Binary => {
                if size < 1024 {
                    format!("{}B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1}KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                }
            }
            ImageFileSizeUnitType::Decimal => {
                if size < 1000 {
                    format!("{}B", size)
                } else if size < 1000 * 1000 {
                    format!("{:.1}KB", size as f64 / 1000.0)
                } else {
                    format!("{:.1}MB", size as f64 / (1000.0 * 1000.0))
                }
            }
        }
    }
}

impl Render for ImageInfo {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let unit_type = ImageFileSizeUnitType::get_global(cx);

        let components = [
            self.width
                .and_then(|w| self.height.map(|h| format!("{}x{}", w, h))),
            self.file_size.map(|s| self.format_file_size(s, unit_type)),
            self.color_type.clone(),
            self.format.clone(),
        ];

        let text = components
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" • ");

        div().when(!text.is_empty(), |el| {
            el.child(Button::new("image-metadata", text).label_size(LabelSize::Small))
        })
    }
}

impl StatusItemView for ImageInfo {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        cx: &mut ViewContext<Self>,
    ) {
        if let Some(image_view) = active_pane_item.and_then(|item| item.act_as::<ImageView>(cx)) {
            self.update_metadata(&image_view, cx);
            self._observe_active_image = Some(cx.observe(&image_view, |this, view, cx| {
                this.update_metadata(&view, cx);
            }));
        } else {
            self.width = None;
            self.height = None;
            self.file_size = None;
            self.color_type = None;
            self.format = None;
            self._observe_active_image = None;
        }
        cx.notify();
    }
}
