use super::state::AppState;

/// One-tick-deferred requests raised by a client's input.
///
/// They live on the shared `AppState` and are not part of `ClientView`, so they
/// survive view swaps: without harvesting they would execute against whichever
/// client's view happens to be loaded when the deferred batch drains, forcing an
/// unrelated client into a new workspace or dropping a modal submit whose payload
/// only exists in the requesting client's view. The headless loop takes them
/// while the raising client still owns the loaded view, then replays each set
/// against that same view.
#[derive(Default)]
pub(crate) struct DeferredClientRequests {
    complete_onboarding: bool,
    new_workspace: bool,
    new_workspace_cwd: Option<std::path::PathBuf>,
    new_tab: bool,
    new_tab_name: Option<String>,
    new_linked_worktree: Option<usize>,
    open_existing_worktree: Option<usize>,
    remove_linked_worktree: Option<usize>,
    submit_worktree_create: bool,
    submit_worktree_open: bool,
    submit_worktree_remove: bool,
}

impl DeferredClientRequests {
    pub(crate) fn is_empty(&self) -> bool {
        !self.complete_onboarding
            && !self.new_workspace
            && self.new_workspace_cwd.is_none()
            && !self.new_tab
            && self.new_tab_name.is_none()
            && self.new_linked_worktree.is_none()
            && self.open_existing_worktree.is_none()
            && self.remove_linked_worktree.is_none()
            && !self.submit_worktree_create
            && !self.submit_worktree_open
            && !self.submit_worktree_remove
    }
}

impl AppState {
    pub(crate) fn take_deferred_client_requests(&mut self) -> DeferredClientRequests {
        DeferredClientRequests {
            complete_onboarding: std::mem::take(&mut self.request_complete_onboarding),
            new_workspace: std::mem::take(&mut self.request_new_workspace),
            new_workspace_cwd: self.request_new_workspace_cwd.take(),
            new_tab: std::mem::take(&mut self.request_new_tab),
            new_tab_name: self.requested_new_tab_name.take(),
            new_linked_worktree: self.request_new_linked_worktree.take(),
            open_existing_worktree: self.request_open_existing_worktree.take(),
            remove_linked_worktree: self.request_remove_linked_worktree.take(),
            submit_worktree_create: std::mem::take(&mut self.request_submit_worktree_create),
            submit_worktree_open: std::mem::take(&mut self.request_submit_worktree_open),
            submit_worktree_remove: std::mem::take(&mut self.request_submit_worktree_remove),
        }
    }

    pub(crate) fn restore_deferred_client_requests(&mut self, requests: DeferredClientRequests) {
        self.request_complete_onboarding = requests.complete_onboarding;
        self.request_new_workspace = requests.new_workspace;
        self.request_new_workspace_cwd = requests.new_workspace_cwd;
        self.request_new_tab = requests.new_tab;
        self.requested_new_tab_name = requests.new_tab_name;
        self.request_new_linked_worktree = requests.new_linked_worktree;
        self.request_open_existing_worktree = requests.open_existing_worktree;
        self.request_remove_linked_worktree = requests.remove_linked_worktree;
        self.request_submit_worktree_create = requests.submit_worktree_create;
        self.request_submit_worktree_open = requests.submit_worktree_open;
        self.request_submit_worktree_remove = requests.submit_worktree_remove;
    }
}
