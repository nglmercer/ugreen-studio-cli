//! Registry browsing policy, separate from operation authorization.
//! Features lists every operation this build can run (plus offline tools);
//! Research lists the registered operations that stay disabled. The worker
//! still checks capabilities and firmware before any live operation.
use super::app::{App, Page};
use crate::features::{self, FeatureCategory, FeatureKind, FeatureSpec};
use crate::i18n::L;

impl App {
    /// Whether the active build may run this feature at all.
    pub(super) fn enabled(&self, spec: &FeatureSpec) -> bool {
        spec.verified(self.operation_capabilities())
    }
    fn in_scope(&self, spec: &FeatureSpec) -> bool {
        let enabled = spec.kind == FeatureKind::OfflineAnalysis || self.enabled(spec);
        (self.page == Page::Research) != enabled
    }
    /// Category tabs of the active browser: "All" plus every category
    /// that has at least one entry in this scope.
    pub(super) fn browser_tabs(&self) -> Vec<Option<FeatureCategory>> {
        let mut tabs = vec![None];
        tabs.extend(
            FeatureCategory::ALL
                .iter()
                .filter(|category| {
                    features::FEATURES
                        .iter()
                        .any(|spec| spec.category == **category && self.in_scope(spec))
                })
                .map(|category| Some(*category)),
        );
        tabs
    }
    pub(super) fn browser_tab_labels(&self) -> Vec<&'static str> {
        let t: &L = self.t();
        self.browser_tabs()
            .into_iter()
            .map(|tab| match tab {
                None => t.features_all,
                Some(category) => t.feature_categories[category.index()],
            })
            .collect()
    }
    /// Entries shown by the active tab and search filter, in order.
    pub(super) fn feature_list(&self) -> Vec<&'static FeatureSpec> {
        let t = self.t();
        let browser = self.browser();
        let tabs = self.browser_tabs();
        let category = tabs.get(browser.tab).copied().flatten();
        let search = browser.search.to_lowercase();
        features::FEATURES
            .iter()
            .filter(|spec| {
                self.in_scope(spec)
                    && category.is_none_or(|c| spec.category == c)
                    && (search.is_empty()
                        || spec.name(t).to_lowercase().contains(&search)
                        || spec.description(t).to_lowercase().contains(&search))
            })
            .collect()
    }
    pub(super) fn selected_feature(&self) -> Option<&'static FeatureSpec> {
        self.feature_list().get(self.browser().index).copied()
    }
}
