<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>核心功能</h1>
    <p>InfraMap 提供了完整的基础设施管理功能,涵盖资源管理、拓扑可视化、数据治理等多个方面。</p>

    <section>
      <h2>资源管理</h2>
      <div class="feature-grid">
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <Monitor />
          </el-icon>
          <h3>服务器管理</h3>
          <p>管理物理机、虚拟机、容器等计算资源,记录 IP、配置、负责人等信息</p>
        </div>
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <Collection />
          </el-icon>
          <h3>系统管理</h3>
          <p>管理业务系统,记录系统负责人、技术栈、部署信息等</p>
        </div>
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <Menu />
          </el-icon>
          <h3>服务管理</h3>
          <p>管理应用服务,记录服务类型、端口、依赖关系等</p>
        </div>
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <Connection />
          </el-icon>
          <h3>中间件管理</h3>
          <p>管理数据库、缓存、消息队列等中间件资源</p>
        </div>
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <SetUp />
          </el-icon>
          <h3>网关管理</h3>
          <p>管理 Nginx 网关配置,记录路由规则和上游服务</p>
        </div>
        <div class="feature-card">
          <el-icon :size="28" color="var(--im-accent)">
            <User />
          </el-icon>
          <h3>联系人管理</h3>
          <p>管理团队成员信息,关联负责的系统和服务</p>
        </div>
      </div>
    </section>

    <section>
      <h2>拓扑可视化</h2>
      <div class="topology-features">
        <div class="topology-feature">
          <h3>多种视图模式</h3>
          <p>提供探索、故障排查、影响面、变更评估四种任务视图,针对不同场景优化</p>
        </div>
        <div class="topology-feature">
          <h3>智能布局算法</h3>
          <p>支持力导向布局和聚焦布局,自动优化节点位置,减少边交叉</p>
        </div>
        <div class="topology-feature">
          <h3>路径追踪</h3>
          <p>查找两个节点间的所有连接路径,分析服务调用链路</p>
        </div>
        <div class="topology-feature">
          <h3>影响分析</h3>
          <p>评估节点故障或变更的影响范围,识别受影响的下游服务</p>
        </div>
        <div class="topology-feature">
          <h3>筛选搜索</h3>
          <p>支持按类型、系统、标签等多维度筛选,快速定位目标节点</p>
        </div>
        <div class="topology-feature">
          <h3>交互操作</h3>
          <p>支持拖拽、缩放、右键菜单等丰富的交互方式</p>
        </div>
      </div>
    </section>

    <section>
      <h2>数据治理</h2>
      <div class="governance-list">
        <div class="governance-item">
          <div class="governance-icon">
            <el-icon :size="24" color="var(--im-success)">
              <UploadFilled />
            </el-icon>
          </div>
          <div class="governance-content">
            <h3>批量录入</h3>
            <p>支持 Excel、CSV 批量导入,快速录入大量数据。提供模板下载和数据校验功能</p>
          </div>
        </div>
        <div class="governance-item">
          <div class="governance-icon">
            <el-icon :size="24" color="var(--im-info)">
              <Files />
            </el-icon>
          </div>
          <div class="governance-content">
            <h3>任务中心</h3>
            <p>查看批量导入任务的执行状态,支持任务重试和错误日志查看</p>
          </div>
        </div>
        <div class="governance-item">
          <div class="governance-icon">
            <el-icon :size="24" color="var(--im-warning)">
              <DataAnalysis />
            </el-icon>
          </div>
          <div class="governance-content">
            <h3>数据健康</h3>
            <p>自动检测数据完整性和一致性问题,如孤立节点、循环依赖、缺失信息等</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>系统设置</h2>
      <div class="settings-list">
        <div class="settings-item">
          <h3>存储路径</h3>
          <p>配置数据库和备份文件的存储位置,支持自定义路径</p>
        </div>
        <div class="settings-item">
          <h3>备份策略</h3>
          <p>配置自动备份策略,定期备份数据,防止数据丢失</p>
        </div>
        <div class="settings-item">
          <h3>快照导入导出</h3>
          <p>支持数据快照的导入和导出,方便数据迁移和版本管理</p>
        </div>
      </div>
    </section>

    <section>
      <h2>功能导航</h2>
      <table class="navigation-table">
        <thead>
          <tr>
            <th>功能模块</th>
            <th>入口位置</th>
            <th>主要用途</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>仪表盘</td>
            <td>左侧菜单 → 仪表盘</td>
            <td>查看资源统计和健康概览</td>
          </tr>
          <tr>
            <td>拓扑图</td>
            <td>左侧菜单 → 拓扑图</td>
            <td>可视化依赖关系分析</td>
          </tr>
          <tr>
            <td>资源管理</td>
            <td>左侧菜单 → 各资源类型</td>
            <td>增删改查基础设施资源</td>
          </tr>
          <tr>
            <td>批量录入</td>
            <td>左侧菜单 → 更多 → 批量录入</td>
            <td>批量导入数据</td>
          </tr>
          <tr>
            <td>数据健康</td>
            <td>左侧菜单 → 更多 → 数据健康</td>
            <td>检查数据质量问题</td>
          </tr>
          <tr>
            <td>系统设置</td>
            <td>左侧菜单底部 → 系统设置</td>
            <td>配置系统参数</td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style scoped lang="scss">
.manual-section {
  h1 {
    font-size: 32px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--im-text-primary);
  }

  h2 {
    font-size: 24px;
    font-weight: 600;
    margin: 32px 0 16px;
    color: var(--im-text-primary);
  }

  h3 {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 8px;
    color: var(--im-text-primary);
  }

  p {
    line-height: 1.8;
    color: var(--im-text-secondary);
    margin-bottom: 16px;
  }

  section {
    margin-bottom: 48px;
  }
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.feature-card {
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  text-align: center;
  transition: all var(--im-duration-base) var(--im-ease-standard);

  &:hover {
    border-color: var(--im-accent);
    box-shadow: var(--im-shadow-sm);
  }

  h3 {
    margin: 12px 0 8px;
  }

  p {
    font-size: 14px;
    margin: 0;
  }
}

.topology-features {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.topology-feature {
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);

  h3 {
    margin: 0 0 8px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.governance-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 24px;
}

.governance-item {
  display: flex;
  gap: 16px;
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
}

.governance-icon {
  flex-shrink: 0;
}

.governance-content {
  flex: 1;

  h3 {
    margin: 0 0 4px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.settings-list {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.settings-item {
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);

  h3 {
    margin: 0 0 8px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.navigation-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 24px;

  th,
  td {
    padding: 12px 16px;
    text-align: left;
    border-bottom: 1px solid var(--im-border);
  }

  th {
    font-weight: 600;
    color: var(--im-text-primary);
    background: var(--im-surface-0);
  }

  td {
    color: var(--im-text-secondary);
  }

  tbody tr:hover {
    background: var(--im-surface-1);
  }
}
</style>
