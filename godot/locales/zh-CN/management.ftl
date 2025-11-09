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

## Incidents

mgmt--mislabel_price =
    .title = 价格标示错误
    .desc = 一些菜品的价格标签贴错了，导致顾客困惑。
    .flavor = 价格标签有误会让顾客感到不安。
    .effects =
        • 顾客满意度 -5%
        • 抱怨概率 +10%
