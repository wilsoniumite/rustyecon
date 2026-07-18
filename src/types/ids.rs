use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub u32);
        impl $name {
            pub fn idx(self) -> usize { self.0 as usize }
        }
    };
}

id_type!(GoodId);
id_type!(RecipeId);
id_type!(ComponentId);
id_type!(BuildingId);
id_type!(PopGroupId);
id_type!(PopPairId);
id_type!(MarketNodeId);
id_type!(ChannelId);
id_type!(RegionId);
id_type!(CountryId);
id_type!(MagicProducerId);
id_type!(InventoryId);
id_type!(RecipeInstanceId);

/// Anything that can hold an inventory or post market orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnerId {
    Building(BuildingId),
    RecipeInstance(RecipeInstanceId),
    PopGroup(PopGroupId),
    Government(RegionId),
    /// Test entity — goods appear from nothing; RemoveFromInventory is a no-op.
    MagicProducer(MagicProducerId),
}
