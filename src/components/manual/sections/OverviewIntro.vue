<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>什么是 InfraMap</h1>
    <p>
      InfraMap 是一款完全离线的基础设施依赖管理平台,专为 DevOps
      工程师、开发者和架构师设计,帮助您可视化管理和分析基础设施组件之间的依赖关系。
    </p>

    <section>
      <h2>核心价值</h2>
      <div class="value-grid">
        <div class="value-card">
          <el-icon :size="40" color="var(--im-accent)">
            <View />
          </el-icon>
          <h3>可视化依赖关系</h3>
          <p>通过拓扑图直观展示服务、中间件、网关之间的依赖关系,一目了然</p>
        </div>
        <div class="value-card">
          <el-icon :size="40" color="var(--im-success)">
            <CircleCheck />
          </el-icon>
          <h3>快速故障定位</h3>
          <p>提供故障排查模式和影响分析工具,帮助快速定位问题根因</p>
        </div>
        <div class="value-card">
          <el-icon :size="40" color="var(--im-warning)">
            <Warning />
          </el-icon>
          <h3>变更风险评估</h3>
          <p>在变更前评估影响范围,识别潜在风险,保障系统稳定性</p>
        </div>
        <div class="value-card">
          <el-icon :size="40" color="var(--im-info)">
            <Lock />
          </el-icon>
          <h3>完全离线运行</h3>
          <p>无需联网,数据完全本地存储,保障企业信息安全</p>
        </div>
      </div>
    </section>

    <section>
      <h2>适用场景</h2>
      <div class="scenario-list">
        <div class="scenario-item">
          <div class="scenario-number">1</div>
          <div class="scenario-content">
            <h3>架构梳理</h3>
            <p>新人入职时快速了解系统架构,老员工梳理复杂的依赖关系</p>
          </div>
        </div>
        <div class="scenario-item">
          <div class="scenario-number">2</div>
          <div class="scenario-content">
            <h3>故障排查</h3>
            <p>线上故障时快速定位问题节点,分析故障影响范围</p>
          </div>
        </div>
        <div class="scenario-item">
          <div class="scenario-number">3</div>
          <div class="scenario-content">
            <h3>变更管理</h3>
            <p>发布前评估变更影响,识别受影响的下游服务</p>
          </div>
        </div>
        <div class="scenario-item">
          <div class="scenario-number">4</div>
          <div class="scenario-content">
            <h3>容量规划</h3>
            <p>分析服务依赖关系,合理规划资源分配和扩容策略</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>核心特性</h2>
      <ul class="feature-list">
        <li>
          <strong>多环境支持:</strong>
          生产、测试、开发环境独立管理,互不干扰
        </li>
        <li>
          <strong>多种布局算法:</strong>
          力导向布局和聚焦布局,适应不同分析需求
        </li>
        <li>
          <strong>强大的分析工具:</strong>
          路径追踪、影响分析、故障排查报告
        </li>
        <li>
          <strong>灵活的筛选搜索:</strong>
          支持按类型、系统、标签等多维度筛选
        </li>
        <li>
          <strong>批量数据导入:</strong>
          支持 Excel、CSV 批量导入,快速录入数据
        </li>
        <li>
          <strong>数据健康检查:</strong>
          自动检测数据完整性和一致性问题
        </li>
        <li>
          <strong>快照导入导出:</strong>
          支持数据快照备份和恢复
        </li>
      </ul>
    </section>

    <section>
      <h2>技术架构</h2>
      <div class="tech-stack">
        <div class="tech-category">
          <h3>前端技术</h3>
          <ul>
            <li>Vue 3 + TypeScript</li>
            <li>Element Plus UI 组件库</li>
            <li>Cytoscape.js 图可视化</li>
            <li>Pinia 状态管理</li>
          </ul>
        </div>
        <div class="tech-category">
          <h3>桌面运行时</h3>
          <ul>
            <li>Tauri 2.x</li>
            <li>Rust 后端</li>
            <li>SQLite 数据库</li>
            <li>本地文件存储</li>
          </ul>
        </div>
      </div>
    </section>

    <section>
      <div class="info-box">
        <el-icon :size="20" color="var(--im-accent)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>开始使用:</strong>
          建议先阅读"拓扑图使用指南"章节,了解核心功能的使用方法。如有问题,可查看"常见问题"章节。
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

.value-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-top: 24px;
}

.value-card {
  padding: 24px;
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

.scenario-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 24px;
}

.scenario-item {
  display: flex;
  gap: 16px;
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
}

.scenario-number {
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

.scenario-content {
  flex: 1;

  h3 {
    margin: 0 0 4px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.feature-list {
  margin: 24px 0;
  padding-left: 20px;
  color: var(--im-text-secondary);

  li {
    margin-bottom: 12px;
    line-height: 1.6;

    strong {
      color: var(--im-text-primary);
    }
  }
}

.tech-stack {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-top: 24px;
}

.tech-category {
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);

  h3 {
    margin: 0 0 12px;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    color: var(--im-text-secondary);

    li {
      margin-bottom: 8px;
    }
  }
}

.info-box {
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
