#[derive(Clone, PartialEq, Debug)]
pub struct TimeSensitiveIncreaseQualityBehaviorThresholds { 
    pub days_left: i32,
    pub increase_rate: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum QualityBehavior  {
    Constant,
    Decrease { rate: i32, min_quality: i32, max_quality: i32 },
    Increase { rate: i32, min_quality: i32, max_quality: i32 },
    TimeSensitiveIncrease {
        min_quality: i32,
        max_quality: i32,
        thresholds: Vec<TimeSensitiveIncreaseQualityBehaviorThresholds>,
        drop_quality_after: i32
    }
}
impl QualityBehavior {

    pub fn standard_decrease() -> QualityBehavior {
        Self::decrease_default_quality(1)
    }

    pub fn standard_increase() -> QualityBehavior {
        Self::increase_default_quality(1)
    }

    pub fn conjured_items() -> QualityBehavior {
        Self::decrease_default_quality(2)
    }

    pub fn backstage_passes_items() -> QualityBehavior {
        Self::new_time_sensitive_default_quality(vec![
            TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 10, increase_rate: 2 },
            TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 5, increase_rate: 3 },
        ])
    }

    pub fn decrease_default_quality(rate: i32) -> QualityBehavior {
        QualityBehavior::Decrease { 
            rate, 
            min_quality: 0, 
            max_quality: 50 
        }
    }

    pub fn increase_default_quality(rate: i32) -> QualityBehavior {
        QualityBehavior::Increase {
            rate, 
            min_quality: 0, 
            max_quality: 50 
        }
    }

    pub fn new_time_sensitive_default_quality(thresholds: Vec<TimeSensitiveIncreaseQualityBehaviorThresholds>) -> QualityBehavior {
        QualityBehavior::TimeSensitiveIncrease { 
            min_quality: 0, 
            max_quality: 50,
            thresholds,
            drop_quality_after: 0
        }
    }
}