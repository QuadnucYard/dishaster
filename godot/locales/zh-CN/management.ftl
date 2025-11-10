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

## Incidents

mgmt--mislabel_price =
    .title = 价格标示错误
    .desc = 一些菜品的价格标签贴错了，导致顾客困惑。
    .flavor = 你问我？那我问你？
    .effects =
        • 顾客满意度 -5%
        • 抱怨概率 +10%
