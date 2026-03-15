<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>拓扑图布局模式</h1>
    <p>InfraMap 提供了两种布局算法,帮助您以不同的方式组织和查看拓扑图。</p>

    <section>
      <h2>力导向布局</h2>
      <div class="layout-card">
        <div class="layout-header">
          <el-icon :size="32" color="var(--im-accent)">
            <Magnet />
          </el-icon>
          <div>
            <h3>自动优化的全局视图</h3>
            <p class="subtitle">模拟物理引力,自动计算最优节点位置</p>
          </div>
        </div>
        <div class="layout-content">
          <div class="layout-feature">
            <h4>工作原理</h4>
            <p>
              力导向布局算法将节点视为带电粒子,边视为弹簧。通过模拟物理力的作用,自动计算出节点的最优位置,使得整个图谱达到视觉平衡。
            </p>
          </div>
          <div class="layout-feature">
            <h4>优势特点</h4>
            <ul>
              <li>
                <strong>全局视图:</strong>
                展示所有节点和依赖关系
              </li>
              <li>
                <strong>自动优化:</strong>
                算法自动调整节点分布,减少边交叉
              </li>
              <li>
                <strong>关系清晰:</strong>
                相关性强的节点会自然聚集在一起
              </li>
              <li>
                <strong>适应性强:</strong>
                适合各种规模的拓扑图
              </li>
            </ul>
          </div>
          <div class="layout-feature">
            <h4>适用场景</h4>
            <ul>
              <li>初次浏览系统架构</li>
              <li>大规模图谱(100+ 节点)</li>
              <li>需要全局视角的分析任务</li>
              <li>探索未知的依赖关系</li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>聚焦布局</h2>
      <div class="layout-card">
        <div class="layout-header">
          <el-icon :size="32" color="var(--im-success)">
            <Aim />
          </el-icon>
          <div>
            <h3>专注核心依赖链</h3>
            <p class="subtitle">选择特定系统或服务,仅展示其上下游依赖</p>
          </div>
        </div>
        <div class="layout-content">
          <div class="layout-feature">
            <h4>工作原理</h4>
            <p>
              聚焦布局允许您选择一个或多个系统/服务作为焦点,系统会自动过滤出与焦点相关的节点和边,隐藏无关的组件,让您专注于核心依赖链。
            </p>
          </div>
          <div class="layout-feature">
            <h4>优势特点</h4>
            <ul>
              <li>
                <strong>减少干扰:</strong>
                隐藏无关节点,专注核心依赖
              </li>
              <li>
                <strong>层次清晰:</strong>
                上下游关系一目了然
              </li>
              <li>
                <strong>快速定位:</strong>
                快速找到特定服务的依赖链
              </li>
              <li>
                <strong>多焦点支持:</strong>
                可同时聚焦多个系统
              </li>
            </ul>
          </div>
          <div class="layout-feature">
            <h4>适用场景</h4>
            <ul>
              <li>单系统深度分析</li>
              <li>故障排查和影响面评估</li>
              <li>变更前依赖检查</li>
              <li>新服务上线前的依赖梳理</li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>如何切换布局</h2>
      <div class="steps">
        <div class="step">
          <div class="step-number">1</div>
          <div class="step-content">
            <h4>打开拓扑图页面</h4>
            <p>从左侧菜单进入"拓扑图"模块</p>
          </div>
        </div>
        <div class="step">
          <div class="step-number">2</div>
          <div class="step-content">
            <h4>找到布局切换按钮</h4>
            <p>在拓扑图工具栏中,找到"布局模式"下拉菜单</p>
          </div>
        </div>
        <div class="step">
          <div class="step-number">3</div>
          <div class="step-content">
            <h4>选择布局模式</h4>
            <p>点击选择"力导向布局"或"聚焦布局"</p>
          </div>
        </div>
        <div class="step">
          <div class="step-number">4</div>
          <div class="step-content">
            <h4>配置聚焦参数(可选)</h4>
            <p>如果选择聚焦布局,需要在侧边栏选择要聚焦的系统或服务</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>布局对比</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>特性</th>
            <th>力导向布局</th>
            <th>聚焦布局</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>显示范围</td>
            <td>全部节点</td>
            <td>焦点及其依赖链</td>
          </tr>
          <tr>
            <td>节点数量</td>
            <td>不限</td>
            <td>通常 10-50 个</td>
          </tr>
          <tr>
            <td>布局方式</td>
            <td>自动优化</td>
            <td>层次化排列</td>
          </tr>
          <tr>
            <td>适合场景</td>
            <td>全局浏览、架构梳理</td>
            <td>单点分析、故障排查</td>
          </tr>
          <tr>
            <td>性能</td>
            <td>大图谱时较慢</td>
            <td>快速响应</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-warning)">
          <WarnTriangleFilled />
        </el-icon>
        <div>
          <strong>性能提示:</strong>
          当拓扑图包含超过 200
          个节点时,建议使用聚焦布局以获得更好的性能和可读性。力导向布局在大规模图谱下可能需要较长的计算时间。
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
    font-size: 18px;
    font-weight: 600;
    margin: 0;
    color: var(--im-text-primary);
  }

  h4 {
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

.layout-card {
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  overflow: hidden;
  margin-top: 24px;
}

.layout-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px;
  background: var(--im-surface-1);
  border-bottom: 1px solid var(--im-border);

  .subtitle {
    margin: 4px 0 0;
    font-size: 14px;
    color: var(--im-text-muted);
  }
}

.layout-content {
  padding: 24px;
}

.layout-feature {
  margin-bottom: 24px;

  &:last-child {
    margin-bottom: 0;
  }

  p {
    margin-bottom: 12px;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    color: var(--im-text-secondary);

    li {
      margin-bottom: 8px;
      line-height: 1.6;

      strong {
        color: var(--im-text-primary);
      }
    }
  }
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 24px;
}

.step {
  display: flex;
  gap: 16px;
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
}

.step-number {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--im-accent);
  color: white;
  font-weight: 600;
  border-radius: 50%;
}

.step-content {
  flex: 1;

  h4 {
    margin: 0 0 4px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.comparison-table {
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

.tip-box {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: rgba(242, 182, 69, 0.1);
  border: 1px solid rgba(242, 182, 69, 0.3);
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
