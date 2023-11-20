use eskf::{Builder, ESKF};
use nalgebra::Point3;
use nav_types::{ECEF, ENU};

use crate::utils::log_to_browser;

pub(super) struct PositionFusion {
    reference_position: Option<ECEF<f32>>,
    kalman_filter: Option<ESKF>,
}

impl PositionFusion {
    pub(super) const fn new() -> Self {
        PositionFusion {
            reference_position: None,
            kalman_filter: None,
        }
    }

    pub(super) fn update_global_position(&mut self, pos: ECEF<f32>) {
        (self.reference_position, self.kalman_filter) = if let (Some(mut kalman), Some(ref_pos)) =
            (self.kalman_filter, self.reference_position)
        {
            let rel_pos = ref_pos - pos;
            let rel_pos = Point3::new(rel_pos.east(), rel_pos.north(), rel_pos.up());

            kalman
                .observe_position(rel_pos, ESKF::variance_from_element(0.1))
                .expect("Filter update failed");
            (Some(ref_pos), Some(kalman))
        } else {
            let mut kalman = Builder::new().build();

            kalman
                .observe_position(
                    Point3::new(0.0f32, 0.0, 0.0),
                    eskf::ESKF::variance_from_element(0.1),
                )
                .expect("Filter update failed");
            (Some(pos), Some(kalman))
        };
    }

    pub(super) fn reset(&mut self) {
        self.reference_position = None;
        self.kalman_filter=None;
        log_to_browser("Position reset".to_string());
    }

    pub(super) fn get_global_position(&self) -> Option<ECEF<f32>> {
        if let (Some(kalman), Some(ref_pos)) = (self.kalman_filter, self.reference_position) {
            let rel_pos = ENU::new(kalman.position[0], kalman.position[1], kalman.position[2]);
            Some(ref_pos + rel_pos)
        } else {
            None
        }
    }
}

impl Default for PositionFusion {
    fn default() -> Self {
        Self::new()
    }
}
