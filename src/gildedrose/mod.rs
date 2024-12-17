pub mod quality_behavior;
mod item;

#[allow(unused_imports)]
pub use item::{GenericItem, Item, ItemV2};

// We could use dynamic dispatch to work with both Item and ItemV2 at the same time
// We could also use reflection (not implemented in rust, we would need to implement it ourselves using a derive macro or use reflection crate)
// We could also use a enum
// enum ItemType {
//     V1(Item),
//     V2(ItemV2),
// }

pub struct GildedRose<T: GenericItem> {
    pub items: Vec<T>,
}

impl<T: GenericItem> GildedRose<T> {

    pub fn new(items: Vec<T>) -> Self {
        GildedRose { items }
    }

    pub fn update_quality(&mut self) {
        self.items.iter_mut().for_each(|item| item.update_quality())
    }
  
}

#[cfg(test)]
mod tests {
    use crate::gildedrose::{
        quality_behavior::{QualityBehavior, TimeSensitiveIncreaseQualityBehaviorThresholds},
        Item, ItemV2, GildedRose
    };

    #[test]
    fn test_item_creation() {
        let item = Item::new("Test Item", 10, 20);
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.sell_in, 10);
        assert_eq!(item.quality, 20);
    }

    #[test]
    fn test_item_display() {
        let item = Item::new("Test Item", 10, 20);
        assert_eq!(format!("{}", item), "Test Item, 10, 20");
    }

    #[test]
    fn test_itemv2_creation_and_timestamp() {
        let behavior = QualityBehavior::standard_decrease();
        let item = ItemV2::new("Test Item", 10, 20, behavior);
        
        assert!(item.timestamp > 0);
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.sell_in, 10);
        assert_eq!(item.quality, 20);
    }

    #[test]
    fn test_gilded_rose_batch_update() {
        let items = vec![
            Item::new("Normal Item", 10, 20),
            Item::new("Aged Brie", 5, 30),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 15, 25),
            Item::new("Sulfuras, Hand of Ragnaros", 0, 80),
            Item::new("Conjured", 3, 15),
        ];
        let mut rose = GildedRose::new(items);
        rose.update_quality();

        // regular item
        assert_eq!(rose.items[0].quality, 19);
        assert_eq!(rose.items[0].sell_in, 9);

        // Aged Brie
        assert_eq!(rose.items[1].quality, 31);
        assert_eq!(rose.items[1].sell_in, 4);

        // Backstage passes
        assert_eq!(rose.items[2].quality, 26);
        assert_eq!(rose.items[2].sell_in, 14);

        // Sulfuras
        assert_eq!(rose.items[3].quality, 80);
        assert_eq!(rose.items[3].sell_in, 0);

        // Conjured
        assert_eq!(rose.items[4].quality, 13);
        assert_eq!(rose.items[4].sell_in, 2);
    }

    #[test]
    fn test_quality_boundaries() {
        let items = vec![
            Item::new("Normal Item", 5, 0),
            Item::new("Aged Brie", 5, 50),
            Item::new("Conjured", 5, 1),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 5, 49),
        ];

        let mut rose = GildedRose::new(items);
        rose.update_quality();

        assert_eq!(rose.items[0].quality, 0);
        assert_eq!(rose.items[1].quality, 50);
        assert_eq!(rose.items[2].quality, 0);
        assert_eq!(rose.items[3].quality, 50);
    }

    #[test]
    fn test_sell_in_effects() {
        let items = vec![
            Item::new("Normal Item", 0, 10),
            Item::new("Aged Brie", 0, 45),
            Item::new("Conjured", 0, 10),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 0, 50),
        ];

        let mut rose = GildedRose::new(items);
        rose.update_quality();

        assert_eq!(rose.items[0].quality, 8);
        assert_eq!(rose.items[1].quality, 46);
        assert_eq!(rose.items[2].quality, 6);
        assert_eq!(rose.items[3].quality, 0);
    }

    #[test]
    fn test_backstage_passes_thresholds() {
        let items = vec![
            Item::new("Backstage passes to a TAFKAL80ETC concert", 11, 20),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 10, 20),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 5, 20),
            Item::new("Backstage passes to a TAFKAL80ETC concert", 0, 20),
        ];

        let mut rose = GildedRose::new(items);
        rose.update_quality();

        assert_eq!(rose.items[0].quality, 21);
        assert_eq!(rose.items[1].quality, 22);
        assert_eq!(rose.items[2].quality, 23);
        assert_eq!(rose.items[3].quality, 0);
    }

    #[test]
    fn test_custom_quality_behavior() {
        let custom_behavior = QualityBehavior::TimeSensitiveIncrease {
            min_quality: 0,
            max_quality: 100,
            thresholds: vec![
                TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 15, increase_rate: 1 },
                TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 10, increase_rate: 3 },
                TimeSensitiveIncreaseQualityBehaviorThresholds { days_left: 5, increase_rate: 5 },
            ],
            drop_quality_after: 0,
        };

        let items = vec![
            ItemV2::new("Custom Item", 16, 20, custom_behavior.clone()),
            ItemV2::new("Custom Item", 11, 20, custom_behavior.clone()),
            ItemV2::new("Custom Item", 6, 20, custom_behavior.clone()),
            ItemV2::new("Custom Item", 1, 20, custom_behavior.clone()),
        ];

        let mut rose = GildedRose::new(items);
        rose.update_quality();

        assert_eq!(rose.items[0].quality, 21);
        assert_eq!(rose.items[1].quality, 21);
        assert_eq!(rose.items[2].quality, 23);
        assert_eq!(rose.items[3].quality, 25);
    }

    #[test]
    fn test_invalid_quality_values() {
        let items = vec![
            Item::new("Normal Item", 5, -10),
            Item::new("Aged Brie", 5, 100),
            Item::new("Sulfuras, Hand of Ragnaros", 5, 80),
        ];

        let mut rose = GildedRose::new(items);
        rose.update_quality();

        assert_eq!(rose.items[0].quality, 0);
        assert_eq!(rose.items[1].quality, 50);
        assert_eq!(rose.items[2].quality, 80);
    }
}