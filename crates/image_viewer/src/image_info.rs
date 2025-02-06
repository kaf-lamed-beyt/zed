use gpui::{div, Context, Entity, IntoElement, ParentElement, Render, Subscription};
use project::image_store::ImageMetadata;
use settings::Settings;
use ui::{prelude::*, Button, LabelSize, Window};
use workspace::{ItemHandle, StatusItemView, Workspace};

use crate::{ImageFileSizeUnit, ImageView, ImageViewerSettings};

pub struct ImageInfo {
    metadata: Option<ImageMetadata>,
    _observe_active_image: Option<Subscription>,
    observe_image_item: Option<Subscription>,
}

impl ImageInfo {
    pub fn new(_workspace: &Workspace) -> Self {
        Self {
            metadata: None,
            _observe_active_image: None,
            observe_image_item: None,
        }
    }

    fn update_metadata(&mut self, image_view: &Entity<ImageView>, cx: &mut Context<Self>) {
        let image_item = image_view.read(cx).image_item.clone();
        let current_metadata = image_item.read(cx).image_metadata.clone();
        if current_metadata.is_some() {
            self.metadata = current_metadata;
            cx.notify();
        } else {
            self.observe_image_item = Some(cx.observe(&image_item, |this, item, cx| {
                this.metadata = item.read(cx).image_metadata.clone();
                cx.notify();
            }));
        }
    }

    fn format_file_size(&self, image_unit_type: ImageFileSizeUnit) -> Option<String> {
        self.metadata.as_ref().map(|metadata| {
            let size = metadata.file_size;
            match image_unit_type {
                ImageFileSizeUnit::Binary => {
                    if size < 1024 {
                        format!("{}B", size)
                    } else if size < 1024 * 1024 {
                        format!("{:.1}KB", size as f64 / 1024.0)
                    } else {
                        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                    }
                }
                ImageFileSizeUnit::Decimal => {
                    if size < 1000 {
                        format!("{}B", size)
                    } else if size < 1000 * 1000 {
                        format!("{:.1}KB", size as f64 / 1000.0)
                    } else {
                        format!("{:.1}MB", size as f64 / (1000.0 * 1000.0))
                    }
                }
            }
        })
    }
}

impl Render for ImageInfo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = ImageViewerSettings::get_global(cx);
        let unit = settings.unit;

        let components = [
            self.metadata
                .as_ref()
                .map(|metadata| format!("{}x{}", metadata.width, metadata.height)),
            self.format_file_size(unit),
            self.metadata
                .as_ref()
                .map(|metadata| metadata.color_type.to_string()),
            self.metadata.as_ref().map(|meta| meta.format.clone()),
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self._observe_active_image = None;
        self.observe_image_item = None;

        if let Some(image_view) = active_pane_item.and_then(|item| item.act_as::<ImageView>(cx)) {
            self.update_metadata(&image_view, cx);

            self._observe_active_image = Some(cx.observe(&image_view, |this, view, cx| {
                this.update_metadata(&view, cx);
            }));
        } else {
            self.metadata = None;
        }
        cx.notify();
    }
}
