use gpui::prelude::*;
use gpui::*;
use crate::molecules::Sidebar;

#[derive(IntoElement)]
pub struct SidebarOrganism {
    active_tab: SharedString,
    collapsed: bool,
    on_select: Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
}

impl SidebarOrganism {
    pub fn new(active_tab: impl Into<SharedString>) -> Self {
        Self {
            active_tab: active_tab.into(),
            collapsed: false,
            on_select: None,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SidebarOrganism {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut sidebar = Sidebar::new(self.active_tab).collapsed(self.collapsed);
        if let Some(on_select) = self.on_select {
            sidebar = sidebar.on_select(move |tab, win, app| on_select(tab, win, app));
        }

        sidebar
    }
}
