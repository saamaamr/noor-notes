use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use adw::prelude::*;
use noor_domain::EditorMode;

use super::{
    AssistanceIssue, CloudAssistanceClient, GrammarService, PredictionModel, PredictionOverlay,
    ResolvedWritingAssistance, checkable_regions,
};

const GRAMMAR_DEBOUNCE: Duration = Duration::from_millis(450);
const PREDICTION_DEBOUNCE: Duration = Duration::from_millis(250);
const GRAMMAR_TAG: &str = "noor-writing-assistance-grammar";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssistanceStatus {
    #[default]
    Idle,
    Checking,
    Offline,
    Cloud,
    Unavailable,
}

impl AssistanceStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "Writing assistance ready",
            Self::Checking => "Checking writing…",
            Self::Offline => "Offline writing assistance",
            Self::Cloud => "Online writing assistance",
            Self::Unavailable => "Writing assistance unavailable",
        }
    }
}

#[derive(Clone)]
pub struct WritingAssistanceController {
    inner: Rc<ControllerInner>,
}

struct ControllerInner {
    buffer: sourceview5::Buffer,
    grammar: Arc<GrammarService>,
    language: RefCell<String>,
    mode: RefCell<EditorMode>,
    preferences: Cell<ResolvedWritingAssistance>,
    suppressed: Cell<bool>,
    generation: Cell<u64>,
    pending: RefCell<Option<gtk::glib::SourceId>>,
    pending_prediction: RefCell<Option<gtk::glib::SourceId>>,
    issues: RefCell<Vec<AssistanceIssue>>,
    status: Cell<AssistanceStatus>,
    status_label: RefCell<Option<gtk::Label>>,
    shutdown: Cell<bool>,
    prediction_overlay: RefCell<Option<PredictionOverlay>>,
    prediction_model: RefCell<Option<Arc<RwLock<PredictionModel>>>>,
    cloud_client: RefCell<Option<CloudAssistanceClient>>,
}

impl WritingAssistanceController {
    pub fn new(
        buffer: &sourceview5::Buffer,
        grammar: Arc<GrammarService>,
        language: &str,
        mode: EditorMode,
    ) -> Self {
        ensure_grammar_tag(buffer);
        Self {
            inner: Rc::new(ControllerInner {
                buffer: buffer.clone(),
                grammar,
                language: RefCell::new(language.to_owned()),
                mode: RefCell::new(mode),
                preferences: Cell::new(ResolvedWritingAssistance {
                    spelling: true,
                    grammar: true,
                    offline_prediction: true,
                    cloud: false,
                }),
                suppressed: Cell::new(false),
                generation: Cell::new(0),
                pending: RefCell::new(None),
                pending_prediction: RefCell::new(None),
                issues: RefCell::new(Vec::new()),
                status: Cell::new(AssistanceStatus::Idle),
                status_label: RefCell::new(None),
                shutdown: Cell::new(false),
                prediction_overlay: RefCell::new(None),
                prediction_model: RefCell::new(None),
                cloud_client: RefCell::new(None),
            }),
        }
    }

    pub fn set_status_label(&self, label: &gtk::Label) {
        self.inner.status_label.replace(Some(label.clone()));
        self.update_status(self.status());
    }

    pub fn set_preferences(&self, preferences: ResolvedWritingAssistance) {
        self.inner.preferences.set(preferences);
        if !preferences.grammar {
            self.clear_issues();
            self.update_status(AssistanceStatus::Idle);
        }
        if !preferences.offline_prediction {
            self.dismiss_prediction();
        }
    }

    pub fn set_prediction_overlay(&self, overlay: PredictionOverlay) {
        self.inner.prediction_overlay.replace(Some(overlay));
    }

    pub fn set_prediction_model(&self, model: Arc<RwLock<PredictionModel>>) {
        self.inner.prediction_model.replace(Some(model));
    }

    pub fn set_cloud_client(&self, client: Option<CloudAssistanceClient>) {
        self.inner.cloud_client.replace(client);
    }

    pub fn set_language(&self, language: &str) {
        self.inner.language.replace(language.to_owned());
        self.notify_content_changed();
    }

    pub fn set_mode(&self, mode: EditorMode) {
        self.inner.mode.replace(mode);
        self.notify_content_changed();
    }

    pub fn set_suppressed(&self, suppressed: bool) {
        self.inner.suppressed.set(suppressed);
        if suppressed {
            self.cancel_pending();
            self.cancel_pending_prediction();
            self.bump_generation();
            self.clear_issues();
            self.dismiss_prediction();
            self.update_status(AssistanceStatus::Idle);
        } else {
            self.notify_content_changed();
        }
    }

    pub fn notify_content_changed(&self) {
        if self.inner.shutdown.get() {
            return;
        }
        let generation = self.bump_generation();
        self.clear_issues();
        self.dismiss_prediction();
        self.cancel_pending();
        self.cancel_pending_prediction();
        if self.inner.suppressed.get() {
            self.update_status(AssistanceStatus::Idle);
            return;
        }
        if self.inner.preferences.get().grammar {
            self.update_status(AssistanceStatus::Checking);
            let controller = self.clone();
            let source = gtk::glib::timeout_add_local_once(GRAMMAR_DEBOUNCE, move || {
                controller.inner.pending.borrow_mut().take();
                controller.run_check(generation);
            });
            self.inner.pending.replace(Some(source));
        } else {
            self.update_status(AssistanceStatus::Idle);
        }
        if self.inner.preferences.get().offline_prediction || self.inner.preferences.get().cloud {
            let controller = self.clone();
            let source = gtk::glib::timeout_add_local_once(PREDICTION_DEBOUNCE, move || {
                controller.inner.pending_prediction.borrow_mut().take();
                controller.show_local_prediction(generation);
            });
            self.inner.pending_prediction.replace(Some(source));
        }
    }

    pub fn check_now(&self) {
        self.cancel_pending();
        self.cancel_pending_prediction();
        let generation = self.bump_generation();
        if !self.can_check() {
            self.clear_issues();
            self.update_status(AssistanceStatus::Idle);
            return;
        }
        self.update_status(AssistanceStatus::Checking);
        let issues = self.compute_issues();
        self.accept_result(generation, issues);
    }

    pub fn visible_issues(&self) -> Vec<AssistanceIssue> {
        self.inner.issues.borrow().clone()
    }

    pub fn status(&self) -> AssistanceStatus {
        self.inner.status.get()
    }

    pub fn apply_replacement(&self, issue_index: usize, replacement_index: usize) -> bool {
        let Some(issue) = self.inner.issues.borrow().get(issue_index).cloned() else {
            return false;
        };
        let Some(replacement) = issue.replacements.get(replacement_index) else {
            return false;
        };
        let mut start = self.inner.buffer.iter_at_offset(issue.range.start as i32);
        let mut end = self.inner.buffer.iter_at_offset(issue.range.end as i32);
        self.inner.buffer.begin_user_action();
        self.inner.buffer.delete(&mut start, &mut end);
        self.inner.buffer.insert(&mut start, replacement);
        self.inner.buffer.end_user_action();
        true
    }

    pub fn ignore_once(&self, issue_index: usize) {
        if issue_index < self.inner.issues.borrow().len() {
            self.inner.issues.borrow_mut().remove(issue_index);
            self.render_issues();
        }
    }

    pub fn shutdown(&self) {
        self.inner.shutdown.set(true);
        self.cancel_pending();
        self.cancel_pending_prediction();
        self.clear_issues();
        self.dismiss_prediction();
    }

    fn can_check(&self) -> bool {
        !self.inner.suppressed.get() && self.inner.preferences.get().grammar
    }

    fn run_check(&self, generation: u64) {
        if !self.can_check() || generation != self.inner.generation.get() {
            return;
        }
        let buffer = self.inner.buffer.clone();
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        let regions = checkable_regions(&buffer, self.inner.mode.borrow().clone());
        let language = self.inner.language.borrow().clone();
        let grammar = self.inner.grammar.clone();
        let task = tokio::task::spawn_blocking(move || grammar.check(&text, &language, &regions));
        let cloud = if self.inner.preferences.get().cloud {
            self.inner.cloud_client.borrow().clone()
        } else {
            None
        };
        let cloud_text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        let cursor = buffer.cursor_position().max(0) as usize;
        let cloud_language = self.inner.language.borrow().clone();
        let controller = self.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            match task.await {
                Ok(mut issues) => {
                    if let Some(cloud) = cloud {
                        if let Ok(mut cloud_issues) = cloud
                            .check_grammar(&cloud_text, cursor, Some(&cloud_language))
                            .await
                        {
                            issues.append(&mut cloud_issues);
                        }
                    }
                    controller.accept_result(generation, issues)
                }
                Err(_) if generation == controller.inner.generation.get() => {
                    controller.update_status(AssistanceStatus::Unavailable);
                }
                Err(_) => {}
            }
        });
    }

    fn compute_issues(&self) -> Vec<AssistanceIssue> {
        let text = self
            .inner
            .buffer
            .text(
                &self.inner.buffer.start_iter(),
                &self.inner.buffer.end_iter(),
                true,
            )
            .to_string();
        let regions = checkable_regions(&self.inner.buffer, self.inner.mode.borrow().clone());
        self.inner
            .grammar
            .check(&text, &self.inner.language.borrow(), &regions)
    }

    fn accept_result(&self, generation: u64, issues: Vec<AssistanceIssue>) {
        if generation != self.inner.generation.get()
            || self.inner.suppressed.get()
            || self.inner.shutdown.get()
        {
            return;
        }
        self.inner.issues.replace(issues);
        self.render_issues();
        self.update_status(if self.inner.preferences.get().cloud {
            AssistanceStatus::Cloud
        } else {
            AssistanceStatus::Offline
        });
    }

    fn render_issues(&self) {
        let start = self.inner.buffer.start_iter();
        let end = self.inner.buffer.end_iter();
        if let Some(tag) = self.inner.buffer.tag_table().lookup(GRAMMAR_TAG) {
            self.inner.buffer.remove_tag(&tag, &start, &end);
            let character_count = end.offset().max(0) as usize;
            for issue in self.inner.issues.borrow().iter() {
                if issue.range.start < issue.range.end && issue.range.end <= character_count {
                    let issue_start = self.inner.buffer.iter_at_offset(issue.range.start as i32);
                    let issue_end = self.inner.buffer.iter_at_offset(issue.range.end as i32);
                    self.inner.buffer.apply_tag(&tag, &issue_start, &issue_end);
                }
            }
        }
    }

    fn clear_issues(&self) {
        self.inner.issues.borrow_mut().clear();
        self.render_issues();
    }

    fn update_status(&self, status: AssistanceStatus) {
        self.inner.status.set(status);
        if let Some(label) = self.inner.status_label.borrow().as_ref() {
            label.set_text(status.label());
            label.update_property(&[gtk::accessible::Property::Label(status.label())]);
        }
    }

    fn bump_generation(&self) -> u64 {
        let value = self.inner.generation.get().wrapping_add(1);
        self.inner.generation.set(value);
        value
    }

    fn cancel_pending(&self) {
        if let Some(source) = self.inner.pending.borrow_mut().take() {
            source.remove();
        }
    }

    fn cancel_pending_prediction(&self) {
        if let Some(source) = self.inner.pending_prediction.borrow_mut().take() {
            source.remove();
        }
    }

    fn show_local_prediction(&self, generation: u64) {
        if generation != self.inner.generation.get()
            || self.inner.suppressed.get()
            || (!self.inner.preferences.get().offline_prediction
                && !self.inner.preferences.get().cloud)
        {
            return;
        }
        let Some(overlay) = self.inner.prediction_overlay.borrow().clone() else {
            return;
        };
        let model = self.inner.prediction_model.borrow().clone();
        if self.inner.buffer.selection_bounds().is_some() {
            overlay.dismiss();
            return;
        }
        let cursor = self.inner.buffer.cursor_position().max(0) as usize;
        let regions = checkable_regions(&self.inner.buffer, self.inner.mode.borrow().clone());
        if !regions
            .iter()
            .any(|region| region.start <= cursor && cursor <= region.end)
        {
            overlay.dismiss();
            return;
        }
        let text = self
            .inner
            .buffer
            .text(
                &self.inner.buffer.start_iter(),
                &self.inner.buffer.end_iter(),
                true,
            )
            .to_string();
        let prefix = text.chars().take(cursor).collect::<String>();
        let partial_start = prefix
            .char_indices()
            .rev()
            .find(|(_, character)| !character.is_alphanumeric() && *character != '_')
            .map_or(0, |(index, character)| index + character.len_utf8());
        let partial = &prefix[partial_start..];
        let context = &prefix[..partial_start];
        let suggestions = model
            .and_then(|model| {
                model
                    .read()
                    .ok()
                    .map(|model| model.suggest(context, partial, 3))
            })
            .unwrap_or_default()
            .into_iter()
            .filter_map(|suggestion| insertion_suffix(&suggestion, partial))
            .collect::<Vec<_>>();
        if generation == self.inner.generation.get() {
            overlay.show(&suggestions);
        }
        if self.inner.preferences.get().cloud {
            if let Some(cloud) = self.inner.cloud_client.borrow().clone() {
                let controller = self.clone();
                let overlay = overlay.clone();
                let partial = partial.to_owned();
                gtk::glib::MainContext::default().spawn_local(async move {
                    if let Ok(cloud_suggestions) = cloud.predict(&text, cursor).await {
                        if generation != controller.inner.generation.get() {
                            return;
                        }
                        let mut merged = overlay.suggestions();
                        merged.extend(
                            cloud_suggestions
                                .into_iter()
                                .filter_map(|suggestion| insertion_suffix(&suggestion, &partial)),
                        );
                        overlay.show(&merged);
                    }
                });
            }
        }
    }

    fn dismiss_prediction(&self) {
        if let Some(overlay) = self.inner.prediction_overlay.borrow().as_ref() {
            overlay.dismiss();
        }
    }
}

fn insertion_suffix(suggestion: &str, partial: &str) -> Option<String> {
    if partial.is_empty() {
        return Some(suggestion.to_owned());
    }
    let normalized = suggestion.to_lowercase();
    let partial_normalized = partial.to_lowercase();
    if !normalized.starts_with(&partial_normalized) {
        return None;
    }
    Some(suggestion.chars().skip(partial.chars().count()).collect())
}

fn ensure_grammar_tag(buffer: &sourceview5::Buffer) {
    if buffer.tag_table().lookup(GRAMMAR_TAG).is_some() {
        return;
    }
    let tag = gtk::TextTag::builder()
        .name(GRAMMAR_TAG)
        .underline(gtk::pango::Underline::Single)
        .foreground("#3584e4")
        .build();
    buffer.tag_table().add(&tag);
}
