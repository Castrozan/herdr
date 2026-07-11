use super::state::{
    AppState, ContextMenuState, CopyModeState, DragState, Mode, NavigatorState, PaneFocusTarget,
    SelectionAutoscroll, TabPressState, ViewState, WorkspacePressState,
};
use crate::selection::Selection;

#[cfg(test)]
#[path = "client_view_tests.rs"]
mod tests;

pub(crate) struct ClientView {
    pub active_workspace_id: Option<String>,
    pub selected_workspace_id: Option<String>,
    pub mode: Mode,
    pub previous_pane_focus: Option<PaneFocusTarget>,
    pub view: ViewState,
    pub navigator: NavigatorState,
    pub copy_mode: Option<CopyModeState>,
    pub selection: Option<Selection>,
    pub selection_autoscroll: Option<SelectionAutoscroll>,
    pub context_menu: Option<ContextMenuState>,
    pub drag: Option<DragState>,
    pub workspace_press: Option<WorkspacePressState>,
    pub tab_press: Option<TabPressState>,
    pub workspace_scroll: usize,
    pub tab_scroll: usize,
    pub tab_scroll_follow_active: bool,
    pub mobile_switcher_scroll: usize,
}

impl Default for ClientView {
    fn default() -> Self {
        Self {
            active_workspace_id: None,
            selected_workspace_id: None,
            mode: Mode::Navigate,
            previous_pane_focus: None,
            view: ViewState::default(),
            navigator: NavigatorState::default(),
            copy_mode: None,
            selection: None,
            selection_autoscroll: None,
            context_menu: None,
            drag: None,
            workspace_press: None,
            tab_press: None,
            workspace_scroll: 0,
            tab_scroll: 0,
            tab_scroll_follow_active: true,
            mobile_switcher_scroll: 0,
        }
    }
}

impl AppState {
    pub(crate) fn active_workspace_id(&self) -> Option<String> {
        self.active
            .and_then(|idx| self.workspaces.get(idx))
            .map(|workspace| workspace.id.clone())
    }

    pub(crate) fn selected_workspace_id(&self) -> Option<String> {
        self.workspaces
            .get(self.selected)
            .map(|workspace| workspace.id.clone())
    }

    fn workspace_index_by_id(&self, workspace_id: &str) -> Option<usize> {
        self.workspaces
            .iter()
            .position(|workspace| workspace.id == workspace_id)
    }

    pub(crate) fn set_active_by_id(&mut self, workspace_id: Option<&str>) {
        self.active = match workspace_id {
            Some(workspace_id) => self
                .workspace_index_by_id(workspace_id)
                .or_else(|| (!self.workspaces.is_empty()).then_some(0)),
            None => None,
        };
    }

    pub(crate) fn snapshot_client_view(&self) -> ClientView {
        ClientView {
            active_workspace_id: self.active_workspace_id(),
            selected_workspace_id: self.selected_workspace_id(),
            mode: self.mode,
            previous_pane_focus: self.previous_pane_focus.clone(),
            view: self.view.clone(),
            navigator: self.navigator.clone(),
            copy_mode: self.copy_mode.clone(),
            selection: self.selection.clone(),
            selection_autoscroll: self.selection_autoscroll.clone(),
            context_menu: self.context_menu.clone(),
            drag: self.drag.clone(),
            workspace_press: self.workspace_press.clone(),
            tab_press: self.tab_press.clone(),
            workspace_scroll: self.workspace_scroll,
            tab_scroll: self.tab_scroll,
            tab_scroll_follow_active: self.tab_scroll_follow_active,
            mobile_switcher_scroll: self.mobile_switcher_scroll,
        }
    }

    pub(crate) fn restore_client_view(&mut self, client_view: &ClientView) {
        self.set_active_by_id(client_view.active_workspace_id.as_deref());
        self.selected = client_view
            .selected_workspace_id
            .as_deref()
            .and_then(|workspace_id| self.workspace_index_by_id(workspace_id))
            .or(self.active)
            .unwrap_or(0);
        self.mode = client_view.mode;
        if self.mode == Mode::Terminal && self.active.is_none() {
            self.mode = Mode::Navigate;
        }
        self.previous_pane_focus = client_view.previous_pane_focus.clone();
        self.view = client_view.view.clone();
        self.navigator = client_view.navigator.clone();
        self.copy_mode = client_view.copy_mode.clone();
        self.selection = client_view.selection.clone();
        self.selection_autoscroll = client_view.selection_autoscroll.clone();
        self.context_menu = client_view.context_menu.clone();
        self.drag = client_view.drag.clone();
        self.workspace_press = client_view.workspace_press.clone();
        self.tab_press = client_view.tab_press.clone();
        self.workspace_scroll = client_view.workspace_scroll;
        self.tab_scroll = client_view.tab_scroll;
        self.tab_scroll_follow_active = client_view.tab_scroll_follow_active;
        self.mobile_switcher_scroll = client_view.mobile_switcher_scroll;
    }
}
