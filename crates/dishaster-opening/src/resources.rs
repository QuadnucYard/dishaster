//! Resources for opening animation simulation

use dishrupt_core::asset::PrefabReference;
use serde::{Deserialize, Serialize};

use crate::{prelude::*, protocol::SimEvent};

/// Configuration for opening animation
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct OpeningConfig {
    /// World bounds (meters)
    pub world_bound: Rect,
    /// Spawn interval for food icons (seconds)
    pub dish_spawn_interval: f32,
    /// Spawn interval for face icons (seconds)
    pub emoji_spawn_interval: f32,
    /// Spawn interval for review texts (seconds)
    pub text_spawn_interval: f32,
    /// Maximum number of food icons
    pub max_foods: usize,
    /// Maximum number of face icons
    pub max_emojis: usize,
    /// Maximum number of review texts
    pub max_texts: usize,
    /// Gravity acceleration (m/s²)
    pub gravity: f32,
}

impl Default for OpeningConfig {
    fn default() -> Self {
        // World centered at origin: [-10, 10] x [-6, 6]
        Self {
            world_bound: Rect::new(-10.0, -6.0, 10.0, 6.0),
            dish_spawn_interval: 0.2,
            emoji_spawn_interval: 0.3,
            text_spawn_interval: 2.0,
            max_foods: 25,
            max_emojis: 8,
            max_texts: 4,
            gravity: 9.8,
        }
    }
}

/// Assets and configurable content for opening animation
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct OpeningAssets {
    /// Universal prefab for all foods (presenter will adjust appearance later)
    pub food_prefab: PrefabReference,
    /// Universal prefab for all faces (presenter will adjust appearance later)
    pub face_prefab: PrefabReference,
    /// Prefab used for review text display (label prefab)
    pub text_prefab: PrefabReference,
    /// Number of food sprite variants available
    pub food_variant_count: u8,
    /// Number of face sprite variants available
    pub face_variant_count: u8,
    /// Review text corpus to display
    pub review_texts: Vec<String>,
}

impl Default for OpeningAssets {
    fn default() -> Self {
        Self {
            food_prefab: PrefabReference::new("opening/food"),
            face_prefab: PrefabReference::new("opening/face"),
            text_prefab: PrefabReference::new("opening/text"),
            food_variant_count: 110,
            face_variant_count: 63,
            review_texts: vec![
                // 讽刺夸张类
                "此食堂当为世界第八大奇迹！".into(),
                "吃完感觉灵魂得到了升华！".into(),
                "建议直接列入非物质文化遗产".into(),
                "食材新鲜度：刚从侏罗纪挖出来".into(),
                "厨师是不是在用爱发电？".into(),
                "食堂教会我，生活就是一场赌博".into(),
                "薛定谔的食堂：打开饭盒前，什么都有可能".into(),
                "吃完后我看见了第五维度".into(),
                "每个窗口都通往不同的平行宇宙".into(),
                // 哲学思考类
                "在排队中思考人生的意义".into(),
                "食堂是检验勇气的唯一标准".into(),
                "今天的菜，明天的回忆".into(),
                "饥饿是最好的调味品".into(),
                "吃饭不是目的，活着才是".into(),
                "每一口都是对命运的挑战".into(),
                // 戏谑吐槽类
                "阿姨的手抖得恰到好处".into(),
                "价格合理，分量感人".into(),
                "这个味道，有点东西".into(),
                "我来食堂不是为了吃饱".into(),
                "打饭阿姨：手有点抖，别见怪".into(),
                "菜品创新：昨天的肉今天的汤".into(),
                "食堂三宝：油水盐".into(),
                "排队半小时，吃完三分钟".into(),
                "这个价位，还要什么自行车".into(),
                "阿姨说：这是特供".into(),
                // 玄学类
                "今日运势：适合下馆子".into(),
                "量子纠缠的打饭手法".into(),
                "测不准原理：饭量版".into(),
                "这是一道会思考的菜".into(),
                "食堂传送门已激活".into(),
                "时空扭曲的味蕾体验".into(),
                "多元宇宙中的食堂".into(),
                // 神秘推荐类
                "别问，问就是特色".into(),
                "阿姨良心推荐".into(),
                "这个你值得拥有".into(),
                "今日限定，过期不候".into(),
                "本店招牌，童叟无欺".into(),
                "秘制配方，祖传手艺".into(),
                "食材直送，新鲜到家".into(),
                // 情感共鸣类
                "想家的时候就来食堂".into(),
                "这里有妈妈的味道（并没有）".into(),
                "青春就是每天吃食堂".into(),
                "毕业后最怀念的地方".into(),
                "和室友一起吐槽的日子".into(),
                "食堂见证了我的成长".into(),
                // 励志类
                "今天也要努力干饭！".into(),
                "为了梦想，吃饱再说".into(),
                "干饭人，干饭魂".into(),
                "饭可以乱吃，话不能乱说".into(),
                "人是铁，饭是钢".into(),
                "吃饱了才有力气吐槽".into(),
                // 科技感类
                "AI都做不出这个味道".into(),
                "未来食品的雏形".into(),
                "这是什么黑科技".into(),
                "分子料理，懂的都懂".into(),
                "食堂2.0测试版".into(),
                // 抽象艺术类
                "这是一种态度".into(),
                "理解不了就对了".into(),
                "艺术来源于生活".into(),
                "这就是行为艺术".into(),
                "先锋实验料理".into(),
                "你品，你细品".into(),
                // 终极吐槽类
                "再见了，食堂君".into(),
                "感谢食堂让我学会外卖".into(),
                "食堂教我学会了做饭".into(),
                "外卖它不香吗".into(),
                "我与食堂的爱恨情仇".into(),
                "食堂，我们下辈子再见".into(),
            ],
        }
    }
}

/// Spawn timers resource
#[derive(Resource, Debug, Default)]
pub struct SpawnTimers {
    /// Timer for food spawning
    pub food: f32,
    /// Timer for face spawning
    pub face: f32,
    /// Timer for text spawning
    pub text: f32,
}

/// Simple time resource for delta-time updates
#[derive(Resource, Debug, Default)]
pub struct DeltaTime {
    /// Delta time in seconds
    pub delta: f32,
}

/// Event queue for simulation events
pub type EventQueue = MessageQueue<SimEvent>;
