//! Page routing is independent from feature and operation authorization.
use super::{
    state::{App, Intent, ObservationView},
    worker::Action,
};
use crate::models::Model;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Route {
    Home,
    Audio,
    Devices,
    Settings,
    Diagnostics,
    Research,
}
impl Route {
    pub const ALL: [Self; 6] = [
        Self::Home,
        Self::Audio,
        Self::Devices,
        Self::Settings,
        Self::Diagnostics,
        Self::Research,
    ];
    pub const fn index(self) -> usize {
        self as usize
    }
}
impl App {
    pub(super) fn paired_scroll(&self, height: u16) -> usize {
        self.paired_index
            .saturating_sub(usize::from(height.saturating_sub(2)).max(1) - 1)
    }

    pub(super) fn navigation_available(&self) -> bool {
        self.busy.is_none()
            && self.worker_alive
            && self.review.is_none()
            && self.feature_form.is_none()
            && self.picker.is_none()
            && self.address_edit.is_none()
            && !self.model_confirmation
            && !self.quit_confirmation
            && !self.options
            && !self.lang_picker
            && !self.help
            && !self.log_open
    }
    pub(super) fn navigate(&mut self, route: Route) -> Intent {
        if !self.navigation_available() {
            return Intent::None;
        }
        let intent = match route {
            Route::Home => self.open_page(2),
            Route::Audio => {
                self.open_page(2);
                self.open_features();
                self.feature_tab = crate::features::FeatureCategory::Audio.index() + 1;
                Intent::None
            }
            Route::Devices => self.open_page(0),
            Route::Settings => self.open_page(2),
            Route::Diagnostics => self.open_page(5),
            Route::Research => self.open_page(4),
        };
        self.route = route;
        intent
    }
    pub(super) fn open_page(&mut self, index: usize) -> Intent {
        if self.busy.is_some() || !self.worker_alive {
            return Intent::None;
        }
        self.route = match index {
            0 => Route::Devices,
            1 => Route::Audio,
            2 => Route::Settings,
            3 => Route::Devices,
            4 => Route::Research,
            5 => Route::Diagnostics,
            _ => return Intent::None,
        };
        self.view_picker = None;
        self.observation_view = None;
        self.observation_scroll = 0;
        self.features = false;
        self.feature_details = false;
        self.feature_form = None;
        self.review = None;
        self.paired = None;
        self.help = false;
        self.log_open = false;
        self.options = false;
        self.lang_picker = false;
        match index {
            0 => Intent::Request(Action::Paired),
            1 => {
                self.observation_view = Some(ObservationView::Audio);
                Intent::Request(Action::Audio {
                    address: self.address.clone(),
                })
            }
            2 => Intent::None,
            3 => {
                self.observation_view = Some(ObservationView::Multipoint);
                if self.control_state.is_connected() {
                    self.submit_feature(crate::features::Operation::Peers)
                } else {
                    Intent::None
                }
            }
            4 => {
                self.open_features();
                self.feature_scope = super::workspace::FeatureScope::Research;
                self.route = Route::Research;
                Intent::None
            }
            5 => {
                self.observation_view = Some(ObservationView::Diagnostics);
                Intent::Request(Action::Doctor {
                    address: (!self.address.is_empty()).then(|| self.address.clone()),
                    model: self.model_confirmed.then_some(Model::StudioPro),
                })
            }
            _ => Intent::None,
        }
    }
}
