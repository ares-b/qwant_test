use {
    std::{
        fmt::{self, Display},
        time::{SystemTime, UNIX_EPOCH}
    },
    super::quality_behavior::QualityBehavior
};

fn get_unix_timestamp() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    duration.as_secs() as i64
}

pub trait GenericItem{
    fn get_behavior(&self) -> QualityBehavior;

    fn get_quality(&self) -> i32;
    fn set_quality(&mut self, quality: i32);

    fn get_sell_in(&self) -> i32;
    fn set_sell_in(&mut self, sell_in: i32);

    fn update_quality(&mut self) {
        let behavior = self.get_behavior();
        let new_quality = match behavior {
            QualityBehavior::Decrease { rate, min_quality, max_quality } => {
                let actual_rate = if self.get_sell_in() <= 0 { rate * 2 } else { rate };
                Some((self.get_quality() - actual_rate).clamp(min_quality, max_quality))
            },
            QualityBehavior::Increase { rate, min_quality, max_quality } => {
                // Should we decrease or keep increasing after concert ?
                Some((self.get_quality() + rate).clamp(min_quality, max_quality))
            },
            QualityBehavior::TimeSensitiveIncrease { 
                min_quality, 
                max_quality, 
                thresholds,
                drop_quality_after 
            } => {
                if self.get_sell_in() <= drop_quality_after {
                    Some(min_quality)
                } else {
                    let increase = thresholds.iter()
                        .filter(|t| self.get_sell_in() <= t.days_left)
                        .min_by_key(|t| t.days_left)
                        .map_or(1, |t| t.increase_rate);
                    
                    Some((self.get_quality() + increase).clamp(min_quality, max_quality))
                }
            },
            QualityBehavior::Constant => {
                None
            },
        };

        if let Some(quality) = new_quality {
            self.set_quality(quality);
            self.set_sell_in(self.get_sell_in() - 1)
        };

    }
}

pub struct Item {
    pub name: String,
    pub sell_in: i32,
    pub quality: i32,
}

impl Item {
    pub fn new(name: impl Into<String>, sell_in: i32, quality: i32) -> Item {
        Item {
            name: name.into(),
            sell_in,
            quality,
        }
    }
}

impl GenericItem for Item {
    fn get_behavior(&self) -> QualityBehavior {
        match self.name.as_str() {
            name if name.contains("Aged Brie") => QualityBehavior::standard_increase(),
            name if name.contains("Backstage passes to a TAFKAL80ETC concert") => QualityBehavior::backstage_passes_items(),
            name if name.contains("Sulfuras, Hand of Ragnaros") => QualityBehavior::Constant,
            name if name.contains("Conjured") => QualityBehavior::conjured_items(),
            _ => QualityBehavior::decrease_default_quality(1),
        }
    }
    fn get_quality(&self) -> i32 {
        self.quality
    }

    fn set_quality(&mut self, quality: i32) {
        self.quality = quality;
    }

    fn get_sell_in(&self) -> i32 {
        self.sell_in
    }

    fn set_sell_in(&mut self, sell_in: i32) {
        self.sell_in = sell_in;
    }

}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.name, self.sell_in, self.quality)
    }
}

pub struct ItemV2 {
    pub name: String,
    pub sell_in: i32,
    pub quality: i32,
    pub behavior: QualityBehavior,
    pub timestamp: i64
}
impl ItemV2 {
    pub fn new(
        name: impl Into<String>,
        sell_in: i32,
        quality: i32,
        behavior: QualityBehavior,
    ) -> ItemV2 {
        ItemV2 {
            name: name.into(),
            sell_in,
            quality,
            behavior,
            timestamp: get_unix_timestamp()
        }
    }

    pub fn from_item(item: Item) -> ItemV2 {
        ItemV2::new(
            item.name.clone(),
            item.sell_in,
            item.quality,
            item.get_behavior(),
        )
    }
}

impl GenericItem for ItemV2 {
    fn get_behavior(&self) -> QualityBehavior {
        self.behavior.clone()
    }

    fn get_quality(&self) -> i32 {
        self.quality
    }

    fn set_quality(&mut self, quality: i32) {
        self.quality = quality;
        self.timestamp = get_unix_timestamp()
    }

    fn get_sell_in(&self) -> i32 {
        self.sell_in
    }

    fn set_sell_in(&mut self, sell_in: i32) {
        self.sell_in = sell_in;
    }
}
impl Display for ItemV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.name, self.sell_in, self.quality)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::{GenericItem, Item, ItemV2, QualityBehavior},
        crate::gildedrose::quality_behavior::TimeSensitiveIncreaseQualityBehaviorThresholds
    };

    // quality type
    #[test]
    fn test_conjured_item_quality() {
        let item = Item::new("Conjured", 0, 20);
        assert_eq!(item.get_behavior(), QualityBehavior::conjured_items());
    }

    #[test]
    fn test_sulfuras_item_quality() {
        let item = Item::new("Sulfuras, Hand of Ragnaros", 0, 20);
        assert_eq!(item.get_behavior(), QualityBehavior::Constant);
    }

    #[test]
    fn test_backstagepasses_item_quality() {
        let item = Item::new("Backstage passes to a TAFKAL80ETC concert", 0, 20);
        assert_eq!(item.get_behavior(), QualityBehavior::backstage_passes_items());
    }

    #[test]
    fn test_agedbrie_item_quality() {
        let item = Item::new("Aged Brie", 0, 20);
        assert_eq!(item.get_behavior(), QualityBehavior::standard_increase());
    }

    #[test]
    fn test_other_item_quality() {
        let item = Item::new("foo", 0, 20);
        assert_eq!(item.get_behavior(), QualityBehavior::standard_decrease());
    }

    // quality update
    #[test]
    fn test_conjured_item_quality_update() {
        let mut item = Item::new("Conjured", 0, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 16);

        // max quality test
        item = Item::new("Conjured", 0, 100);
        item.update_quality();
        assert_eq!(item.get_quality(), 50);

        // min quality test
        item = Item::new("Conjured", 0, 0);
        item.update_quality();
        assert_eq!(item.get_quality(), 0);
    }

    #[test]
    fn test_sulfuras_item_quality_update() {
        let mut item = Item::new("Sulfuras, Hand of Ragnaros", 0, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 20);
    }

    #[test]
    fn test_backstagepasses_item_quality_update() {
        let mut item = Item::new("Backstage passes to a TAFKAL80ETC concert", 11, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 21);

        // 10 days or less
        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 10, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 22);

        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 6, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 22);

        // 5 days or less
        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 5, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 23);

        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 3, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 23);

        // after concert
        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 0, 26);
        item.update_quality();
        assert_eq!(item.get_quality(), 0);

        // max quality test
        item = Item::new("Backstage passes to a TAFKAL80ETC concert", 20, 51);
        item.update_quality();
        assert_eq!(item.get_quality(), 50);

    }

    #[test]
    fn test_agedbrie_item_quality_update() {
        let mut item = Item::new("Aged Brie", 11, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 21);

        // max quality test
        item = Item::new("Aged Brie", 10, 50);
        item.update_quality();
        assert_eq!(item.get_quality(), 50);

        // after concert
        item = Item::new("Aged Brie", 0, 0);
        item.update_quality();
        assert_eq!(item.get_quality(), 1);
    }

    #[test]
    fn test_other_item_quality_update() {
        let mut item = Item::new("foo", 5, 20);

        item.update_quality();
        assert_eq!(item.get_quality(), 19);

        // after concert
        item = Item::new("foo", 0, 20);
        item.update_quality();
        assert_eq!(item.get_quality(), 18);

        // min quality test
        item = Item::new("foo", 10, 0);
        item.update_quality();
        assert_eq!(item.get_quality(), 0);
    }

    #[test]
    fn test_itemv2_custom_behavior() {
        let custom_behavior = QualityBehavior::new_time_sensitive_default_quality(vec![
            TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 3, increase_rate: 18 },
            TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 20, increase_rate: 2 },
        ]);

        let mut item = ItemV2::new("Custom Item", 20, 10, custom_behavior.clone());
        item.update_quality();
        assert_eq!(item.quality, 12);

        item = ItemV2::new("Custom Item", 3, 20, custom_behavior);
        item.update_quality();
        assert_eq!(item.quality, 38);
    }
}