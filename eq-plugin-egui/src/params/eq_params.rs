use super::*;
use std::ops::RangeInclusive;

#[derive(nice::Params)]
pub struct EqParams {
    #[id = "gain_db"]
    pub gain_db: nice::FloatParam,

    #[id = "frequency"]
    pub log_frequency: nice::FloatParam,

    #[id = "q"]
    pub q: nice::FloatParam,

    #[id = "eq_type"]
    pub eq_type: eq_type::Param,
}

impl EqParams {
    pub fn from_eq(
        names_suffix: &str,
        eq: &eq_params::eq::Eq<f32>,
        log_frequency_range: &RangeInclusive<f32>,
        db_range: &RangeInclusive<f32>,
        q_range: &RangeInclusive<f32>,
        smoothing_length_ms: f32,
    ) -> Self {
        Self {
            gain_db: nice::FloatParam::new(
                format!("Gain (dB){names_suffix}"),
                eq.gain.db(),
                nice::FloatRange::Linear {
                    min: *db_range.start(),
                    max: *db_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms))
            .with_unit(" dB"),
            log_frequency: nice::FloatParam::new(
                format!("Frequency (Hz){names_suffix}"),
                eq.frequency.log_hz(),
                nice::FloatRange::Linear {
                    min: *log_frequency_range.start(),
                    max: *log_frequency_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms))
            .with_unit(" Hz")
            .with_value_to_string(sync::Arc::new(egui_lib::utils::log_frequency_to_string))
            .with_string_to_value(sync::Arc::new(egui_lib::utils::string_to_log_frequency)),
            q: nice::FloatParam::new(
                format!("q{names_suffix}"),
                eq.q,
                nice::FloatRange::Linear {
                    min: *q_range.start(),
                    max: *q_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms)),
            eq_type: eq_type::Param::new(
                format!("Eq Type{names_suffix}"),
                eq_type::Wrapper::from(eq.eq_type),
            ),
        }
    }

    pub fn to_eq<F: audio_lib::utils::Float>(&self) -> eq::Eq<F> {
        eq::Eq {
            gain: eq::Gain::Db(F::from(self.gain_db.value()).unwrap()),
            frequency: eq::Frequency::LogHz(F::from(self.log_frequency.value()).unwrap()),
            q: F::from(self.q.value()).unwrap(),
            eq_type: self.eq_type.value().into(),
        }
    }

    pub fn set_gain_db<F: utils::Float>(&self, gain_db: F, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.gain_db);
        setter.set_parameter(&self.gain_db, gain_db.to_f32().unwrap());
        setter.end_set_parameter(&self.gain_db);
    }

    pub fn set_log_frequency<F: utils::Float>(
        &self,
        log_frequency: F,
        setter: &nice::ParamSetter<'_>,
    ) {
        setter.begin_set_parameter(&self.log_frequency);
        setter.set_parameter(&self.log_frequency, log_frequency.to_f32().unwrap());
        setter.end_set_parameter(&self.log_frequency);
    }

    pub fn set_q<F: utils::Float>(&self, q: F, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.q);
        setter.set_parameter(&self.q, q.to_f32().unwrap());
        setter.end_set_parameter(&self.q);
    }

    pub fn set_eq_type(&self, eq_type: eq::EqType, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.eq_type);
        setter.set_parameter(&self.eq_type, eq_type.into());
        setter.end_set_parameter(&self.eq_type);
    }
}
