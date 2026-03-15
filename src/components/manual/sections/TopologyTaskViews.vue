<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>拓扑图任务视图</h1>
    <p>
      InfraMap 提供了多种任务视图模式,每种模式针对不同的使用场景进行了优化,帮助您更高效地完成工作。
    </p>

    <section>
      <h2>视图模式概览</h2>
      <div class="view-modes">
        <div class="view-card">
          <div class="view-header">
            <el-icon :size="28" color="var(--im-accent)">
              <View />
            </el-icon>
            <h3>探索模式</h3>
          </div>
          <div class="view-content">
            <p class="view-desc">全局依赖关系浏览,展示所有组件之间的连接关系</p>
            <div class="view-usage">
              <strong>适用场景:</strong>
              <ul>
                <li>了解系统架构全貌</li>
                <li>梳理服务依赖关系</li>
                <li>新人熟悉系统结构</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="view-card">
          <div class="view-header">
            <el-icon :size="28" color="var(--im-warning)">
              <Warning />
            </el-icon>
            <h3>故障排查模式</h3>
          </div>
          <div class="view-content">
            <p class="view-desc">查看节点健康状态和故障证据,快速定位问题根因</p>
            <div class="view-usage">
              <strong>适用场景:</strong>
              <ul>
                <li>线上故障快速定位</li>
                <li>分析服务异常原因</li>
                <li>查看依赖链健康状态</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="view-card">
          <div class="view-header">
            <el-icon :size="28" color="var(--im-danger)">
              <Connection />
            </el-icon>
            <h3>影响面模式</h3>
          </div>
          <div class="view-content">
            <p class="view-desc">评估节点变更或故障的影响范围,识别受影响的下游服务</p>
            <div class="view-usage">
              <strong>适用场景:</strong>
              <ul>
                <li>变更前风险评估</li>
                <li>故障影响面分析</li>
                <li>下游服务识别</li>
              </ul>
            </div>
          </div>
        </div>

        <div class="view-card">
          <div class="view-header">
            <el-icon :size="28" color="var(--im-success)">
              <CircleCheck />
            </el-icon>
            <h3>变更评估模式</h3>
          </div>
          <div class="view-content">
            <p class="view-desc">发布前评估变更风险,查看上下游依赖关系</p>
            <div class="view-usage">
              <strong>适用场景:</strong>
              <ul>
                <li>发布前影响分析</li>
                <li>变更风险评估</li>
                <li>依赖链完整性检查</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>模式对比</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>特性</th>
            <th>探索模式</th>
            <th>故障排查模式</th>
            <th>影响面模式</th>
            <th>变更评估模式</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>显示范围</td>
            <td>全局</td>
            <td>全局</td>
            <td>选定节点及下游</td>
            <td>选定节点及上下游</td>
          </tr>
          <tr>
            <td>健康状态</td>
            <td>-</td>
            <td>✓</td>
            <td>✓</td>
            <td>✓</td>
          </tr>
          <tr>
            <td>故障证据</td>
            <td>-</td>
            <td>✓</td>
            <td>-</td>
            <td>-</td>
          </tr>
          <tr>
            <td>影响范围高亮</td>
            <td>-</td>
            <td>-</td>
            <td>✓</td>
            <td>✓</td>
          </tr>
          <tr>
            <td>路径追踪</td>
            <td>✓</td>
            <td>✓</td>
            <td>✓</td>
            <td>✓</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <h2>切换视图模式</h2>
      <p>
        在拓扑图页面顶部的工具栏中,点击"任务视图"下拉菜单,即可切换不同的视图模式。切换后,画布会根据所选模式自动调整显示内容和交互方式。
      </p>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          不同视图模式下,右键菜单的可用操作也会有所不同。例如,在故障排查模式下,可以查看节点的健康证据;在影响面模式下,可以快速查看受影响的下游服务列表。
        </div>
      </div>
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

.view-modes {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-top: 24px;
}

.view-card {
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  overflow: hidden;
  transition: all var(--im-duration-base) var(--im-ease-standard);

  &:hover {
    border-color: var(--im-accent);
    box-shadow: var(--im-shadow-sm);
  }
}

.view-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px;
  background: var(--im-surface-1);
  border-bottom: 1px solid var(--im-border);

  h3 {
    margin: 0;
  }
}

.view-content {
  padding: 20px;
}

.view-desc {
  margin-bottom: 16px;
  font-size: 14px;
}

.view-usage {
  strong {
    display: block;
    margin-bottom: 8px;
    color: var(--im-text-primary);
    font-size: 14px;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    color: var(--im-text-secondary);
    font-size: 14px;

    li {
      margin-bottom: 4px;
    }
  }
}

.comparison-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 24px;
  font-size: 14px;

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

.tip-box {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: var(--im-accent-soft);
  border: 1px solid var(--im-accent-dim);
  border-radius: var(--im-radius-md);
  margin-top: 24px;

  strong {
    color: var(--im-text-primary);
  }

  div {
    flex: 1;
    font-size: 14px;
    line-height: 1.6;
    color: var(--im-text-secondary);
  }
}
</style>
