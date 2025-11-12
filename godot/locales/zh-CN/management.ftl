### Management Decisions (player-selected changes)

## Decisions

mgmt--add_tables =
    .title = 添加桌子
    .desc = 增加食堂内的桌子数量，提升接待能力。
    .flavor = 更多的桌子意味着更多的顾客可以入座用餐
    .effects =
        • 桌子数量 +{ $num_tables }

mgmt--remove_tables =
    .title = 移除桌子
    .desc = 减少食堂内的桌子数量，腾出更多空间。
    .flavor = 有时，空间比桌子更重要。
    .effects =
        • 桌子数量 -{ $num_tables }

mgmt--disarrange_tables =
    .title = 调整桌子布局
    .desc = 重新安排桌子的位置，优化空间利用率。
    .flavor = 更好的布局带来更好的用餐体验。但是，是随机的。
    .effects =
        • 随机改变 { $num_tables } 张桌子的位置
        • 优化空间利用率（？）

mgmt--open_window =
    .title = 开设新窗口
    .desc = 随机增加一个新的服务窗口，分担现有窗口的压力。
    .flavor = 更多的窗口意味着更快的服务速度。
    .effects =
        • 窗口数量 +1

mgmt--close_window =
    .title = 关闭一个窗口
    .desc = 随机关闭一个服务窗口，节省运营成本。
    .flavor = 有时，关闭一个窗口可以提高整体效率。
    .effects =
        • 窗口数量 -1

mgmt--change_window_service =
    .title = 更换窗口服务类型
    .desc = 更换一个窗口的服务类型，以适应顾客需求变化。
    .flavor = 灵活的服务类型可以吸引更多的顾客。
    .effects =
        • 随机更换一个窗口的服务类型

## Music Decisions

mgmt--play_music_relaxing =
    .title = 播放舒缓音乐
    .desc = 在食堂播放轻柔的背景音乐，营造优雅氛围。
    .flavor = "舒缓的旋律回荡在食堂里，食客们放慢了节奏，细嚼慢咽……翻台率也随之下降。"
    .effects =
        • 进食速度 -{ $speed_change }%（音乐太舒缓，吃得更慢了）
        • 满意度 { $satisfaction_change }（至少气氛上档次了）

mgmt--play_music_upbeat =
    .title = 播放快节奏音乐
    .desc = 在食堂播放动感的背景音乐，加快就餐节奏。
    .flavor = "欢快的节拍在空中跳动，食客们的咀嚼频率不自觉地跟上了鼓点。食堂变成了效率至上的战场。"
    .effects =
        • 进食速度 +{ $speed_change }%（赶时间的福音）
        • 满意度 { $satisfaction_change }（有人觉得吵，有人觉得燃）

mgmt--play_music_anthem =
    .title = 播放校歌
    .desc = 在食堂播放学校的校歌，激发师生的归属感与荣誉感。
    .flavor = "熟悉的旋律响起，有人肃然起敬，有人默默翻白眼。校歌能激励校友的心，也能劝退新生的胃。"
    .effects =
        • 进食速度 { $speed_change }%（影响因人而异）
        • 满意度 { $satisfaction_change }（爱恨两极分化）

## Campaign Decisions

mgmt--advertise_canteen =
    .title = 全食堂广告宣传
    .desc = 在校园各处投放食堂广告，全面提升知名度。
    .flavor = "宣传海报贴满了教学楼："本食堂荣获ISO9001质量认证"——虽然没人知道这个证书是从哪买的。初期人流暴增，但热度来得快去得也快。"
    .effects =
        • 客流量 +{ $boost }%（持续 { $days } 天）
        • 每日衰减 { $decay }%（热度总会过去）

mgmt--advertise_window =
    .title = 窗口定向推广
    .desc = 针对某个特定窗口进行重点推广，吸引相关顾客。
    .flavor = "精准营销，在吃货论坛疯狂安利某个窗口。虽然效果不如全食堂轰炸，但胜在持久，毕竟'酒香不怕巷子深'……才怪，还是得靠宣传。"
    .effects =
        • 目标窗口客流量 +{ $boost }%（持续 { $days } 天）
        • 每日衰减 { $decay }%（细水长流型）

## Slogan Decisions

mgmt--slogan_hardship =
    .title = 张贴励志标语
    .desc = 在食堂墙上张贴传统励志标语："嚼得菜根，做得大事"。
    .flavor = "巨大的红色横幅高高悬挂，古朴的八个大字映入眼帘。有人看到后斗志昂扬，有人却觉得这是在讽刺食堂的菜品质量……效果可叠加，墙上标语越来越多。"
    .effects =
        • 信任度 > { $threshold }%: 满意度 { $boost }（被激励了）
        • 信任度 ≤ { $threshold }%: 满意度 { $penalty }（精神污染）
        • 可与已有标语叠加效果

## Incidents

mgmt--mislabel_price =
    .title = 价格标示错误
    .desc = 一些菜品的价格标签贴错了，导致顾客困惑。
    .flavor = 你问我？那我问你？
    .effects =
        • 顾客满意度 -5%
        • 抱怨概率 +10%
