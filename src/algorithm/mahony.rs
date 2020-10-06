#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};

pub struct Mahony {
    sample_interval: f32,
    kp: f32,
    ki: f32,
    magnetic_north: Vector3<f32>,
    error_integral: Vector3<f32>,
    quaternion: UnitQuaternion<f32>,
}

#[derive(Copy, Clone)]
pub enum MagnetismOrHeading {
    Magnetism(Vector3<f32>),
    Heading(f32),
}

impl Mahony {
    pub fn new(sample_rate: f32, kp: f32, ki: f32, magnetic_declination: f32) -> Self {
        let declination = magnetic_declination.to_radians();
        Self {
            sample_interval: 1.0 / sample_rate,
            kp,
            ki,
            magnetic_north: Vector3::new(declination.sin(), declination.cos(), 0.0),
            error_integral: Vector3::new(0.0, 0.0, 0.0),
            quaternion: UnitQuaternion::new_unchecked(Quaternion::new(1.0, 0.0, 0.0, 0.0)),
        }
    }

    pub fn quaternion(&self) -> UnitQuaternion<f32> {
        self.quaternion
    }

    fn magnetism_error(&self, magnetism_or_heading: MagnetismOrHeading) -> Vector3<f32> {
        let q = &self.quaternion;
        match magnetism_or_heading {
            MagnetismOrHeading::Magnetism(magnetism) => {
                if let Some(normalized) = magnetism.try_normalize(0.0) {
                    let mut estimated = q.transform_vector(&normalized);
                    estimated[2] = 0.0;
                    if let Some(estimated) = estimated.try_normalize(0.01) {
                        let error = self.magnetic_north.cross(&estimated);
                        return q.inverse_transform_vector(&error);
                    }
                }
            }
            MagnetismOrHeading::Heading(heading) => {
                let mut estimated = q.transform_vector(&Vector3::new(0.0, 1.0, 0.0));
                estimated[2] = 0.0;
                if let Some(estimated) = estimated.try_normalize(0.01) {
                    let rad = heading.to_radians();
                    let actual = Vector3::new(rad.sin(), rad.cos(), 0.0);
                    let error = actual.cross(&estimated);
                    return q.inverse_transform_vector(&error);
                }
            }
        }
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn update(
        &mut self,
        gyro: &Vector3<f32>,
        acceleration: &Vector3<f32>,
        magnetism: Option<MagnetismOrHeading>,
    ) -> Option<UnitQuaternion<f32>> {
        let acceleration = match acceleration.try_normalize(0.0) {
            Some(a) => a,
            None => return None,
        };

        let q = self.quaternion;
        let v = Vector3::new(
            2.0 * (q.i * q.k - q.j * q.w),
            2.0 * (q.w * q.i + q.j * q.k),
            q.k * q.k + q.w * q.w - q.i * q.i - q.j * q.j,
        );
        let mut error = acceleration.cross(&v);
        if let Some(magnetism_or_heading) = magnetism {
            error += self.magnetism_error(magnetism_or_heading);
        }

        if self.ki > 0.0 {
            self.error_integral += error * self.sample_interval;
        } else {
            self.error_integral = Vector3::new(0.0, 0.0, 0.0);
        }

        let gyro = gyro - error * self.kp - self.error_integral * self.ki;
        let q_derivate = 0.5 * q.into_inner() * Quaternion::from_parts(0.0, gyro);
        let q_intergral = q_derivate * self.sample_interval;

        self.quaternion = UnitQuaternion::new_normalize(q.into_inner() + q_intergral);
        Some(self.quaternion)
    }
}

mod test {
    #[test]
    fn test_mahony_course() {
        use nalgebra::Vector3;

        use crate::datastructures::measurement::euler::Euler;

        use super::{MagnetismOrHeading, Mahony};

        let gyro = Vector3::new(0.0, 0.0, 0.0);
        let accel = Vector3::new(0.0, 0.0, -1.0);

        let mut mahony = Mahony::new(10.0, 10.0, 0.0, 0.0);

        let course: f32 = 270.0;
        let magnetism = Some(MagnetismOrHeading::Heading(course));
        for _ in 0..10 {
            mahony.update(&gyro, &accel, magnetism);
        }
        let euler: Euler = mahony.quaternion().into();
        let mut yaw = -euler.yaw.to_degrees() as isize;
        if yaw < 0 {
            yaw = 360 + yaw
        };
        assert_eq!(yaw, 270);
    }
}
