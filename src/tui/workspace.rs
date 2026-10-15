//! Registry browsing policy, separate from operation authorization.
//! Historical evidence determines where a feature is discoverable; the worker
//! still checks the connected model and firmware before any live operation.
use super::state::App;
use crate::features::{self, FeatureSpec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FeatureScope {
    Everyday,
    Research,
}
impl FeatureScope {
    fn includes(self, spec: &FeatureSpec) -> bool {
        let supported = spec.kind != features::FeatureKind::OfflineAnalysis
            && spec.verified(crate::models::supported_capabilities());
        match self {
            Self::Everyday => supported,
            Self::Research => !supported,
        }
    }
}
impl App {
    /// The registry entries shown by the active category
    /// tab and search filter, in listing order.
    pub(super) fn feature_list(&self) -> Vec<&'static FeatureSpec> {
        let t = self.t();
        let search = self.feature_search.to_lowercase();
        features::FEATURES
            .iter()
            .filter(|spec| {
                let in_category =
                    self.feature_tab == 0 || spec.category.index() + 1 == self.feature_tab;
                let matches = search.is_empty()
                    || spec.name(t).to_lowercase().contains(&search)
                    || spec.description(t).to_lowercase().contains(&search);
                in_category && matches && self.feature_scope.includes(spec)
            })
            .collect()
    }
    pub(super) fn selected_feature(&self) -> Option<&'static FeatureSpec> {
        self.feature_list().get(self.feature_index).copied()
    }
}
