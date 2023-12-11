use eskf::{Builder, ESKF};
use nalgebra::{Matrix2, Point3, Vector2, Vector3};
use nav_types::{ECEF, ENU};
use std::time::Instant;

use crate::utils::log_to_browser;

pub(super) struct PositionFusion {
    reference_position: Option<ECEF<f32>>,
    kalman_filter: Option<ESKF>,
    last_prediction:Option<Instant>,

}

impl PositionFusion {
    pub(super) const fn new() -> Self {
        PositionFusion {
            reference_position: None,
            kalman_filter: None,
            last_prediction: None,
        }
    }

    pub(super) fn update_global_position(
        &mut self,
        pos: ECEF<f32>,
        velocity_2d: Option<Vector2<f32>>,
    ) {
        (self.reference_position, self.kalman_filter) = if let (Some(mut kalman), Some(ref_pos)) =
            (self.kalman_filter, self.reference_position)
        {
            let rel_pos = ref_pos - pos;
            let rel_pos = Point3::new(rel_pos.east(), rel_pos.north(), rel_pos.up());
            observe_position(&mut kalman, rel_pos, 0.1, velocity_2d, 0.1);
            (Some(ref_pos), Some(kalman))
        } else {
            let mut kalman = Builder::new().build();
            observe_position(
                &mut kalman,
                Point3::new(0.0f32, 0.0, 0.0),
                0.1,
                velocity_2d,
                0.1,
            );
            (Some(pos), Some(kalman))
        };
    }

    pub(super) fn reset(&mut self) {
        self.reference_position = None;
        self.kalman_filter = None;
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

    pub(super) fn predict( &mut self,acceleration:Vector3<f32>,rotation:Vector3<f32>){

        if let (Some(mut kalman),Some(last_prediction) )= (self.kalman_filter,self.last_prediction)  {
            let delta=Instant::now()- last_prediction;
            self.last_prediction=Some(Instant::now());
            kalman.predict(acceleration, rotation, delta)
        }
    }


}

impl Default for PositionFusion {
    fn default() -> Self {
        Self::new()
    }
}

fn observe_position(
    kalman: &mut ESKF,
    position: Point3<f32>,
    pos_variance: f32,
    velocity: Option<Vector2<f32>>,
    vel_variance: f32,
) {
    if let Some(vel) = velocity {
        kalman
            .observe_position_velocity2d(
                position,
                ESKF::variance_from_element(pos_variance),
                vel,
                Matrix2::from_diagonal_element(vel_variance),
            )
            .expect("Filter update failed");
    } else {
        kalman
            .observe_position(position, eskf::ESKF::variance_from_element(pos_variance))
            .expect("Filter update failed");
    }
}
