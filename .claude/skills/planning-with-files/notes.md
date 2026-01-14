# Notes: AppType功能隐藏分析

## 需要隐藏/关闭的功能列表

### 一、涉及金融审查的敏感功能（需要从TRADING_PLATFORM和CHANNEL中隐藏）

#### 1. 推荐系统相关
**前端路由和组件**:
- `/recommender` - 推荐人管理 (已有 appType: AppType.DISCOUNT_DIRECT)
- `/users/recommender` - 用户管理中的推荐人标签页
- `packages/admin/src/views/panels/RecommenderPanel.vue`
- `packages/admin/src/views/panels/RecommenderSettlementPanel.vue`

**后端API**:
- `packages/server/src/api/admin/recommendationRoutes.ts`
- `packages/server/src/services/admin/recommendationService.ts`
- `packages/server/src/services/admin/recommenderService.ts`

**数据库表**:
- recommendations (推荐表)
- recommender_settlements (推荐人结算表)

#### 2. KPI管理系统
**前端路由和组件**:
- `/kpi-management` - KPI管理 (已有 appType: AppType.DISCOUNT_DIRECT)
- `packages/admin/src/views/KpiManagementView.vue`

**后端API**:
- `packages/server/src/api/admin/kpiRoutes.ts`
- `packages/server/src/services/admin/kpiService.ts`
- `packages/server/src/models/kpi.ts`
- `packages/server/src/config/kpiConfig.ts`

**数据库表**:
- kpi_records
- monthly_kpi_summaries

#### 3. 财务管理（Finance Management - 区别于财务中心）
**前端路由和组件**:
- `/finance-management` - 财务管理 (已有 appType: AppType.DISCOUNT_DIRECT)
- `packages/admin/src/views/FinanceManagementView.vue`

**后端API**:
- `packages/server/src/api/admin/financeMgmtRoutes.ts`
- `packages/server/src/services/admin/financeService.ts`
- `packages/server/src/models/financialOverview.ts`
- `packages/server/src/models/monthlyReceipt.ts`

#### 4. 积分钱包系统
**前端路由和组件**:
- `/finance/wallet` - 积分钱包 (在财务中心下)
- `packages/client/src/views/WalletView.vue`
- `packages/admin/src/views/panels/WalletPanel.vue`

**后端API**:
- `packages/server/src/api/walletRoutes.ts`
- `packages/server/src/api/admin/walletRoutes.ts`
- `packages/server/src/services/admin/walletConfigService.ts`
- `packages/server/src/models/walletConfig.ts`

**数据库表**:
- wallet_configs
- wallet_usage_records

#### 5. 推荐人结算
**前端路由和组件**:
- `/settlements/recommender` - 推荐人结算单 (在结算管理下)
- `packages/admin/src/views/panels/RecommenderSettlementPanel.vue`

**后端API**:
- `packages/server/src/api/admin/settlementRoutes.ts` 中的推荐人结算部分
- `packages/server/src/services/admin/settlementService.ts`

#### 6. 多空钱包
**前端**:
- 相关的长短仓钱包功能
- `packages/server/src/services/longShortWalletService.ts`
- `packages/server/src/models/longShortWallet.ts`

**数据库表**:
- long_short_wallets

#### 7. 盈利分成功能
**配置**:
- `packages/shared/src/types/config.ts` 中的 profit_sharing 配置
- `packages/server/src/config/defaultParams.ts` 中的默认配置

#### 8. 用户推荐字段
**数据库字段**:
- users.recommender_id (推荐人ID)
- users.contribution_level (贡献等级 - 已迁移到recommendations表)

#### 9. 其他推荐相关功能
- `packages/shared/src/types/recommendation.ts`
- `packages/shared/src/types/settlement.ts` 中的推荐人结算类型

---

## 当前已有AppType限制的功能（保留）

### 仅TRADING_PLATFORM可见
- `/channel-management` - 通道管理
- `/channel-fund` - 通道资金审核
- `/channel-orders` - 通道订单查询

### 仅CHANNEL可见
- `/users/leads` - 线索管理
- `/finance/platform-fund` - 平台资金管理

### 仅DISCOUNT_DIRECT可见
- `/cooperation-mode` - 合作模式管理 (已注释掉)
- `/recommender` - 推荐人管理
- `/kpi-management` - KPI管理
- `/finance-management` - 财务管理
- `/external-quotes` - 外部报价管理

---

## 实施方案

### 前端隐藏策略

#### 方法1: Router Meta属性 (推荐用于路由级控制)
在 `packages/admin/src/router/index.ts` 中添加或修改 `appType` 属性：
```typescript
{
  path: "recommender",
  name: "users-recommender",
  component: () => import("@/views/UsersView.vue"),
  meta: {
    permission: AdminPermission.FINANCE,
    subView: "recommender",
    appType: AppType.DISCOUNT_DIRECT, // 只在DISCOUNT_DIRECT版本显示
  },
}
```

#### 方法2: Layout条件渲染 (推荐用于菜单级控制)
在 `packages/admin/src/layouts/AdminLayout.vue` 中使用条件渲染：
```typescript
{
  path: "/kpi-management",
  icon: DataLine,
  label: "KPI管理",
  permission: AdminPermission.FINANCE,
  appType: AppType.DISCOUNT_DIRECT,
}
```

#### 方法3: 组件内部v-if (推荐用于组件级控制)
在具体组件中使用计算属性：
```vue
<template>
  <div v-if="isDiscountDirect">
    <!-- 敏感内容 -->
  </div>
</template>

<script setup lang="ts">
import { useAuthStore } from "@/stores/auth";
import { AppType } from "@packages/shared";

const authStore = useAuthStore();
const isDiscountDirect = computed(
  () => authStore.app_type === AppType.DISCOUNT_DIRECT
);
</script>
```

### 后端关闭策略

#### 方法1: API路由条件注册 (推荐)
在路由注册前检查AppType：
```typescript
// 只在DISCOUNT_DIRECT版本注册推荐相关路由
if (isDiscountDirect()) {
  router.registerRoutes(recommendationRoutes);
}
```

#### 方法2: Service层检查
在service函数开始处检查：
```typescript
export async function getRecommenderStats() {
  if (!isDiscountDirect()) {
    throw new AppError("功能在当前版本不可用", 403, "FEATURE_NOT_AVAILABLE");
  }
  // 继续处理
}
```

#### 方法3: Middleware拦截
创建专用的AppType检查中间件：
```typescript
export const requireDiscountDirect = (req, res, next) => {
  if (!isDiscountDirect()) {
    return res.status(403).json({
      error: "FEATURE_NOT_AVAILABLE",
      message: "此功能仅在折扣直通版中可用"
    });
  }
  next();
};
```

---

## 实施优先级

### P0 - 必须立即实施（核心敏感功能）
1. 推荐人管理 - `/recommender`, `/users/recommender`
2. KPI管理 - `/kpi-management`
3. 财务管理 - `/finance-management`
4. 推荐人结算 - `/settlements/recommender`

### P1 - 高优先级（建议同步实施）
1. 积分钱包 - `/finance/wallet`
2. 盈利分成配置
3. 推荐相关API后端路由

### P2 - 中优先级（后续优化）
1. 多空钱包功能
2. 用户推荐字段的UI隐藏
3. 推荐相关数据库表的访问控制

---

## 测试检查清单

### 前端测试
- [ ] TRADING_PLATFORM版本不显示推荐相关菜单和路由
- [ ] CHANNEL版本不显示推荐相关菜单和路由
- [ ] DISCOUNT_DIRECT版本正常显示所有功能
- [ ] 路由守卫正确拦截非法访问

### 后端测试
- [ ] TRADING_PLATFORM版本推荐相关API返回403
- [ ] CHANNEL版本推荐相关API返回403
- [ ] DISCOUNT_DIRECT版本推荐相关API正常工作
- [ ] 数据库查询不会意外返回敏感数据

### 数据安全测试
- [ ] 无法通过API绕过前端限制
- [ ] JWT token中的app_type正确设置
- [ ] 数据库连接隔离（如需要）
