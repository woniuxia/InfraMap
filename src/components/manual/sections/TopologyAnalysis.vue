<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>拓扑图分析功能</h1>
    <p>
      InfraMap 提供了强大的分析工具,帮助您深入理解基础设施的依赖关系、评估变更影响、快速定位故障。
    </p>

    <section>
      <h2>路径追踪</h2>
      <div class="feature-card">
        <div class="feature-header">
          <el-icon :size="28" color="var(--im-accent)">
            <Guide />
          </el-icon>
          <h3>查找节点间的所有连接路径</h3>
        </div>
        <div class="feature-content">
          <p>路径追踪功能可以帮助您找到两个节点之间的所有可能路径,了解服务间的调用链路。</p>

          <h4>使用步骤</h4>
          <ol class="steps-list">
            <li>在拓扑图中右键点击起始节点</li>
            <li>在弹出菜单中选择"路径追踪"</li>
            <li>在节点列表中选择目标节点</li>
            <li>系统会高亮显示所有可能的路径</li>
          </ol>

          <h4>应用场景</h4>
          <ul>
            <li>分析服务调用链路</li>
            <li>排查请求流转路径</li>
            <li>识别关键依赖节点</li>
            <li>优化服务间通信</li>
          </ul>

          <div class="example-box">
            <strong>示例:</strong>
            从"用户服务"追踪到"订单数据库",可以看到请求经过了"订单服务"和"订单缓存"两条路径。
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>影响分析</h2>
      <div class="feature-card">
        <div class="feature-header">
          <el-icon :size="28" color="var(--im-danger)">
            <Warning />
          </el-icon>
          <h3>评估节点故障或变更的影响范围</h3>
        </div>
        <div class="feature-content">
          <p>影响分析功能可以快速识别某个节点故障或变更时,会影响到哪些下游服务和系统。</p>

          <h4>使用步骤</h4>
          <ol class="steps-list">
            <li>在拓扑图中右键点击要分析的节点</li>
            <li>在弹出菜单中选择"影响分析"</li>
            <li>系统会高亮显示所有受影响的下游节点</li>
            <li>在右侧面板查看详细的影响列表</li>
          </ol>

          <h4>分析维度</h4>
          <ul>
            <li>
              <strong>直接影响:</strong>
              直接依赖该节点的服务
            </li>
            <li>
              <strong>间接影响:</strong>
              通过依赖链受影响的服务
            </li>
            <li>
              <strong>影响层级:</strong>
              显示影响的深度(1级、2级、3级...)
            </li>
            <li>
              <strong>影响数量:</strong>
              统计受影响的节点总数
            </li>
          </ul>

          <div class="example-box">
            <strong>示例:</strong>
            分析"Redis 缓存"节点,发现直接影响 5 个服务,间接影响 12 个服务,总计 17 个节点受影响。
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>故障排查报告</h2>
      <div class="feature-card">
        <div class="feature-header">
          <el-icon :size="28" color="var(--im-warning)">
            <DocumentChecked />
          </el-icon>
          <h3>查看节点健康证据和风险提示</h3>
        </div>
        <div class="feature-content">
          <p>在故障排查模式下,可以查看每个节点的健康状态、故障证据和风险提示,快速定位问题根因。</p>

          <h4>使用步骤</h4>
          <ol class="steps-list">
            <li>切换到"故障排查模式"</li>
            <li>点击任意节点</li>
            <li>在右侧详情面板查看健康报告</li>
            <li>根据证据和提示进行问题排查</li>
          </ol>

          <h4>报告内容</h4>
          <ul>
            <li>
              <strong>健康状态:</strong>
              正常/警告/异常
            </li>
            <li>
              <strong>故障证据:</strong>
              监控指标、日志片段、告警信息
            </li>
            <li>
              <strong>依赖健康:</strong>
              上游依赖的健康状态
            </li>
            <li>
              <strong>风险提示:</strong>
              潜在的风险点和建议
            </li>
          </ul>

          <div class="warning-box">
            <el-icon :size="18" color="var(--im-warning)">
              <InfoFilled />
            </el-icon>
            <div>
              <strong>注意:</strong>
              故障排查报告依赖于系统中录入的监控数据和健康检查配置。如果节点没有配置健康检查,报告可能不完整。
            </div>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>分析功能对比</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>功能</th>
            <th>路径追踪</th>
            <th>影响分析</th>
            <th>故障排查报告</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>分析对象</td>
            <td>两个节点间的路径</td>
            <td>单个节点的下游影响</td>
            <td>单个节点的健康状态</td>
          </tr>
          <tr>
            <td>输出结果</td>
            <td>所有可能路径</td>
            <td>受影响节点列表</td>
            <td>健康报告和证据</td>
          </tr>
          <tr>
            <td>适用场景</td>
            <td>调用链分析</td>
            <td>变更风险评估</td>
            <td>故障根因定位</td>
          </tr>
          <tr>
            <td>需要模式</td>
            <td>任意模式</td>
            <td>影响面/变更评估模式</td>
            <td>故障排查模式</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <Lightbulb />
        </el-icon>
        <div>
          <strong>最佳实践:</strong>
          在进行重要变更前,建议依次使用"影响分析"评估影响范围、"路径追踪"确认关键路径、"故障排查报告"检查依赖健康状态,确保变更安全。
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
    margin: 20px 0 12px;
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

.feature-card {
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  overflow: hidden;
  margin-top: 24px;
}

.feature-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px;
  background: var(--im-surface-1);
  border-bottom: 1px solid var(--im-border);
}

.feature-content {
  padding: 24px;
}

.steps-list {
  margin: 0 0 20px;
  padding-left: 24px;
  color: var(--im-text-secondary);

  li {
    margin-bottom: 8px;
    line-height: 1.6;
  }
}

ul {
  margin: 0 0 20px;
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

.example-box {
  padding: 16px;
  background: var(--im-accent-soft);
  border-left: 3px solid var(--im-accent);
  border-radius: var(--im-radius-sm);
  margin-top: 20px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--im-text-secondary);

  strong {
    color: var(--im-text-primary);
  }
}

.warning-box {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: rgba(242, 182, 69, 0.1);
  border: 1px solid rgba(242, 182, 69, 0.3);
  border-radius: var(--im-radius-md);
  margin-top: 20px;

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
