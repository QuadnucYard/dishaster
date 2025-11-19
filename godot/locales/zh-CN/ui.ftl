### UI Translations

app =
    .title = Dishaster 菜根传奇

## Start Menu and Commons

start =
    .label = 开始
credits =
    .label = 制作人员
quit =
    .label = 退出
back =
    .label = 返回
apply =
    .label = 应用
cancel =
    .label = 取消
confirm =
    .label = 确认
skip =
    .label = 跳过
exit =
    .label = 退出

roll-seed =
    .tooltip = 为当前玩家档案重新生成随机种子
clear-level =
    .tooltip = 清除当前关卡的所有进度
delete-profile =
    .tooltip = 删除当前玩家档案及其所有进度

## In Game

start-run =
    .label = 开始运营
end-run =
    .label = 结束运营
exit-level =
    .label = 退出关卡

phase-preparation =
    .label = 准备阶段
    .desc = 检查当前状态，调整价格，开始新的一天的经营！
phase-running =
    .label = 运营阶段
    .desc = 观察食堂运营情况，满足学生需求，努力提升声誉与收益！
day-display =
    .label = 第 { $day } 天

price-by-portion =
    .label = 按份计价
price-by-weight =
    .label = 按重量计价

dish-name =
    .label = 菜名
dish-price-adjust =
    .label = 调整价格
price-by-portion-display =
    .label = /份
price-by-weight-display =
    .label = /kg

manage-decision =
    .title = 选择运营决策
    .subtitle = 选择一项决策来改善明天的运营

effects-title =
    .label = 效果:
select-option =
    .label = 选择此项

continue-run =
    .label = 继续运营

inspector-visit-result =
    .title = 卫生检查结果
    .desc =
        您的食堂通过了卫生检查！
        但是……好事会一直持续下去吗？

inspector-visit-effects =
    • 信任度 +{ PCT($trust_boost, maxfd: 1) }（食品安全提升）
    • 声誉 +{ NUM($reputation_boost, maxfd: 1) }（公众形象改善）

trial-agreement = 赞同
trial-objection = 反对
trial-perjury = 伪证
trial-question = 疑问

## Settlement

settlement-title = 第 { $day } 天结算

settlement-stats =
    {"[b]营业统计[/b]"}
    • 到访人数：{ $total_visits } 人
    • 完成用餐：{ $completed_diners } 人（{ NUM($completion_rate, maxfd: 1) }%）
    • 营业收入：¥{ NUM($revenue, maxfd: 1) }
    • 食材消耗：{ NUM($consumption_kg, maxfd: 1) } kg
    • 平均备餐时间：{ NUM($avg_serving_time, maxfd: 1) } 秒
    • 平均用餐时间：{ NUM($avg_dining_time, maxfd: 1) } 秒
    {"[b]声誉与质量[/b]"}
    • 声誉：{ NUM($reputation, maxfd: 1) }（{ $reputation_delta }）
    • 食品安全风险指数：{ NUM($fsri, maxfd: 1) }
    • 食品质量：{ NUM($food_quality, maxfd: 1) }

settlement-guidance = 点击确认按钮继续进入管理决策阶段。
