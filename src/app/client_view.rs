use super::state::{
    AppState, ContextMenuState, CopyFeedback, CopyModeState, DragState, KeybindHelpState,
    MenuListState, Mode, NavigatorState, PaneFocusTarget, ProductAnnouncementState,
    ReleaseNotesState, SelectionAutoscroll, TabPressState, ViewState,
    WorkspacePressState, WorktreeCreateState, WorktreeOpenState, WorktreeRemoveState,
};
use crate::layout::PaneId;
use crate::selection::Selection;

#[cfg(test)]
#[path = "client_view_tests.rs"]
mod tests;

/// Per-client, workspace-relative view state swapped into AppState around render and input.
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
    pub name_input: String,
    pub name_input_replace_on_type: bool,
    pub creating_new_tab: bool,
    pub rename_pane_target: Option<PaneId>,
    pub worktree_create: Option<WorktreeCreateState>,
    pub worktree_open: Option<WorktreeOpenState>,
    pub worktree_remove: Option<WorktreeRemoveState>,
    pub release_notes: Option<ReleaseNotesState>,
    pub product_announcement: Option<ProductAnnouncementState>,
    pub keybind_help: KeybindHelpState,
    pub global_menu: MenuListState,
    pub copy_feedback: Option<CopyFeedback>,
    pub agent_panel_scroll: usize,
    pub collapsed_space_keys: std::collections::HashSet<String>,
}

impl AppState {
    fn active_workspace_id(&self) -> Option<String> {
        self.active
            .and_then(|idx| self.workspaces.get(idx))
            .map(|ws| ws.id.clone())
    }

    fn selected_workspace_id(&self) -> Option<String> {
        self.workspaces.get(self.selected).map(|ws| ws.id.clone())
    }

    pub(crate) fn workspace_index_by_id(&self, workspace_id: &str) -> Option<usize> {
        self.workspaces.iter().position(|ws| ws.id == workspace_id)
    }

    fn set_active_by_id(&mut self, workspace_id: Option<&str>) {
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
            name_input: self.name_input.clone(),
            name_input_replace_on_type: self.name_input_replace_on_type,
            creating_new_tab: self.creating_new_tab,
            rename_pane_target: self.rename_pane_target,
            worktree_create: self.worktree_create.clone(),
            worktree_open: self.worktree_open.clone(),
            worktree_remove: self.worktree_remove.clone(),
            release_notes: self.release_notes.clone(),
            product_announcement: self.product_announcement.clone(),
            keybind_help: self.keybind_help.clone(),
            global_menu: self.global_menu,
            copy_feedback: self.copy_feedback.clone(),
            agent_panel_scroll: self.agent_panel_scroll,
            collapsed_space_keys: self.collapsed_space_keys.clone(),
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
        self.name_input = client_view.name_input.clone();
        self.name_input_replace_on_type = client_view.name_input_replace_on_type;
        self.creating_new_tab = client_view.creating_new_tab;
        self.rename_pane_target = client_view.rename_pane_target;
        self.worktree_create = client_view.worktree_create.clone();
        self.worktree_open = client_view.worktree_open.clone();
        self.worktree_remove = client_view.worktree_remove.clone();
        self.release_notes = client_view.release_notes.clone();
        self.product_announcement = client_view.product_announcement.clone();
        self.keybind_help = client_view.keybind_help.clone();
        self.global_menu = client_view.global_menu;
        self.copy_feedback = client_view.copy_feedback.clone();
        self.agent_panel_scroll = client_view.agent_panel_scroll;
        self.collapsed_space_keys = client_view.collapsed_space_keys.clone();
    }
}
