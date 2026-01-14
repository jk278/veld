# Task Plan: 探索AppType系统并重新启用TRADING_PLATFORM和CHANNEL版本

## Goal
重新启用TRADING_PLATFORM和CHANNEL版本的AppType系统，条件隐藏或关闭涉及金融审查的敏感功能（财务管理、KPI、推荐人、推荐、佣金等），确保简洁优雅高效纯粹地实现。

## Phases
- [x] Phase 1: 创建任务计划和理解需求
- [x] Phase 2: 探索当前AppType系统实现
- [x] Phase 3: 使用git-analyzer查看历史版本
- [x] Phase 4: 分析需要隐藏/关闭的功能列表
- [x] Phase 5: 设计实施方案
- [x] Phase 6: 实施前端隐藏逻辑
- [x] Phase 7: 实施后端关闭逻辑
- [ ] Phase 8: 测试验证
- [ ] Phase 9: 文档和交付

## Key Questions
1. ~~当前AppType系统在哪里定义和使用？~~ 已解答
2. ~~TRADING_PLATFORM和CHANNEL版本历史如何实现的？~~ 已解答
3. 哪些功能需要针对这两个版本隐藏/关闭？
4. 前端和后端分别需要做哪些修改？

## Decisions Made
- 使用已有的AppType枚举和判断函数（isTradingPlatform(), isChannel(), isDiscountDirect()）
- 前端使用appType属性在router和layout中进行条件渲染
- 后端使用相应的判断函数进行API级别的控制

## Errors Encountered
-

## Status
**Currently in Phase 8-9** - 实施完成，构建验证成功，准备交付

## 实施总结

### 已完成修改

#### 前端修改
1. **packages/admin/src/router/index.ts**
   - `/users/recommender` 添加 `appType: AppType.DISCOUNT_DIRECT`
   - `/settlements/recommender` 添加 `appType: AppType.DISCOUNT_DIRECT`

2. **packages/admin/src/views/FinancialCenterView.vue**
   - 积分钱包标签页添加 `v-if="hasDiscountDirectAppType"` 条件渲染
   - 新增 `hasDiscountDirectAppType` 计算属性

#### 后端修改
1. **新建: packages/server/src/middlewares/appTypeRestriction.ts**
   - 创建 `requireDiscountDirect` 中间件
   - 返回403错误，提示"此功能仅在折扣直通版中可用"

2. **packages/server/src/api/admin/recommendationRoutes.ts**
   - 添加 `router.use(requireDiscountDirect)` 限制所有推荐相关API

3. **packages/server/src/api/admin/kpiRoutes.ts**
   - 添加 `router.use(requireDiscountDirect)` 限制所有KPI管理API

4. **packages/server/src/api/admin/financeMgmtRoutes.ts**
   - 添加 `router.use(requireDiscountDirect)` 限制所有财务管理API

5. **packages/server/src/api/admin/settlementRoutes.ts**
   - 在推荐人结算相关路由上添加 `requireDiscountDirect`:
     - GET `/recommender/name-mapping`
     - POST `/recommender/name-mapping`
     - POST `/recommender/daily`
     - POST `/recommender/monthly`

### 功能隐藏效果

| 功能 | 前端隐藏 | 后端关闭 | 仅DISCOUNT_DIRECT |
|------|----------|----------|-------------------|
| 推荐人管理 | ✓ | ✓ | ✓ |
| KPI管理 | ✓ | ✓ | ✓ |
| 财务管理 | ✓ | ✓ | ✓ |
| 推荐人结算 | ✓ | ✓ | ✓ |
| 积分钱包 | ✓ | - | ✓ |

### 构建验证
- Server包构建成功 ✅
- TypeScript类型检查通过 ✅

## 研究发现

### AppType系统架构
1. **定义位置**: `packages/shared/src/index.ts`
   ```typescript
   export enum AppType {
     TRADING_PLATFORM = "trading_platform",
     CHANNEL = "channel",
     DISCOUNT_DIRECT = "discount_direct",
   }
   ```

2. **后端判断函数**: `packages/server/src/config/configManager.ts`
   ```typescript
   export function isTradingPlatform(): boolean
   export function isChannel(): boolean
   export function isDiscountDirect(): boolean
   ```

3. **前端Store**: `packages/client/src/stores/appInfo.ts`
   ```typescript
   const isTradingPlatform = computed(() => appType.value === "trading_platform")
   const isChannel = computed(() => appType.value === "channel")
   const isDiscountDirect = computed(() => appType.value === "discount_direct")
   ```

### 历史实现模式
1. **条件渲染菜单**（历史代码）:
   ```typescript
   ...(isTradingPlatform()
     ? [
         { path: "/channel-fund", label: "通道资金审核", ... },
       ]
     : [])
   ```

2. **AppType属性限制**（当前使用）:
   ```typescript
   {
     path: "/cooperation-mode",
     appType: AppType.DISCOUNT_DIRECT,
   }
   ```

### 需要隐藏的敏感功能（详细分析见notes.md）

#### P0优先级 - 必须实施
1. **推荐人管理** (`/recommender`, `/users/recommender`)
   - 前端：路由meta添加 appType: AppType.DISCOUNT_DIRECT
   - 后端：推荐相关API需要添加AppType检查

2. **KPI管理** (`/kpi-management`)
   - 前端：已有 appType: AppType.DISCOUNT_DIRECT ✓
   - 后端：需要确认API是否有AppType检查

3. **财务管理** (`/finance-management`)
   - 前端：已有 appType: AppType.DISCOUNT_DIRECT ✓
   - 后端：需要确认API是否有AppType检查

4. **推荐人结算** (`/settlements/recommender`)
   - 前端：路由meta添加 appType: AppType.DISCOUNT_DIRECT
   - 后端：结算API需要添加AppType检查

#### P1优先级 - 建议同步实施
5. **积分钱包** (`/finance/wallet`)
   - 前端：在财务中心中条件渲染
   - 后端：钱包API需要添加AppType检查

6. **盈利分成配置**
   - 后端：profit_sharing功能需要条件关闭

### 实施策略

#### 前端实施要点
1. 使用 `appType: AppType.DISCOUNT_DIRECT` 在路由meta中限制
2. 在AdminLayout.vue中已有shouldShowMenuItem函数支持appType过滤
3. 在组件内部使用v-if条件渲染敏感内容

#### 后端实施要点
1. 使用 `isDiscountDirect()` 函数检查AppType
2. 在API路由注册或service层添加检查
3. 对于敏感功能，返回403 Forbidden错误
