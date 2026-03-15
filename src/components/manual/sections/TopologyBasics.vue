<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>拓扑图基础操作</h1>
    <p>
      拓扑图是 InfraMap
      的核心功能,用于可视化展示基础设施组件之间的依赖关系。本章节介绍拓扑图的基本交互方式和节点类型识别。
    </p>

    <section>
      <h2>画布交互</h2>
      <div class="interaction-grid">
        <div class="interaction-card">
          <el-icon :size="32" color="var(--im-accent)">
            <Mouse />
          </el-icon>
          <h3>拖拽平移</h3>
          <p>按住鼠标左键在空白区域拖动,可以平移整个画布视图</p>
        </div>
        <div class="interaction-card">
          <el-icon :size="32" color="var(--im-accent)">
            <ZoomIn />
          </el-icon>
          <h3>缩放视图</h3>
          <p>使用鼠标滚轮或双指捏合手势,可以放大或缩小画布</p>
        </div>
        <div class="interaction-card">
          <el-icon :size="32" color="var(--im-accent)">
            <Pointer />
          </el-icon>
          <h3>选择节点</h3>
          <p>单击节点可查看详细信息,右侧面板会显示节点属性</p>
        </div>
        <div class="interaction-card">
          <el-icon :size="32" color="var(--im-accent)">
            <MoreFilled />
          </el-icon>
          <h3>右键菜单</h3>
          <p>右键点击节点,可执行编辑、路径追踪、影响分析等操作</p>
        </div>
      </div>
    </section>

    <section>
      <h2>节点类型识别</h2>
      <p>拓扑图中的节点通过不同的形状和颜色来区分类型:</p>
      <div class="node-types">
        <div class="node-type-card">
          <div class="node-shape circle" style="background: #5ca3ff"></div>
          <div class="node-info">
            <h3>服务节点</h3>
            <p>蓝色圆形,代表应用服务(如 API、Web 应用等)</p>
          </div>
        </div>
        <div class="node-type-card">
          <div class="node-shape diamond" style="background: #f2b645"></div>
          <div class="node-info">
            <h3>中间件节点</h3>
            <p>橙色菱形,代表中间件(如 Redis、MySQL、Kafka 等)</p>
          </div>
        </div>
        <div class="node-type-card">
          <div class="node-shape hexagon" style="background: #41c58a"></div>
          <div class="node-info">
            <h3>网关节点</h3>
            <p>绿色六边形,代表 Nginx 网关配置</p>
          </div>
        </div>
        <div class="node-type-card">
          <div class="node-shape circle dashed" style="border-color: #93a4c4"></div>
          <div class="node-info">
            <h3>外部节点</h3>
            <p>灰色虚线边框,代表外部依赖或第三方服务</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>快捷键</h2>
      <table class="shortcuts-table">
        <thead>
          <tr>
            <th>快捷键</th>
            <th>功能</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><kbd>R</kbd></td>
            <td>重置布局,恢复到初始视图</td>
          </tr>
          <tr>
            <td><kbd>F</kbd></td>
            <td>切换全屏模式</td>
          </tr>
          <tr>
            <td>
              <kbd>Ctrl</kbd>
              +
              <kbd>F</kbd>
            </td>
            <td>打开搜索框,快速定位节点</td>
          </tr>
          <tr>
            <td><kbd>Esc</kbd></td>
            <td>取消当前选择</td>
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

.interaction-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.interaction-card {
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

.node-types {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 24px;
}

.node-type-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
}

.node-shape {
  width: 48px;
  height: 48px;
  flex-shrink: 0;

  &.circle {
    border-radius: 50%;
  }

  &.diamond {
    transform: rotate(45deg);
    border-radius: 8px;
  }

  &.hexagon {
    clip-path: polygon(30% 0%, 70% 0%, 100% 50%, 70% 100%, 30% 100%, 0% 50%);
  }

  &.dashed {
    background: transparent;
    border: 2px dashed;
  }
}

.node-info {
  flex: 1;

  h3 {
    margin: 0 0 4px;
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}

.shortcuts-table {
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

  kbd {
    display: inline-block;
    padding: 4px 8px;
    font-family: var(--im-font-mono);
    font-size: 12px;
    background: var(--im-surface-1);
    border: 1px solid var(--im-border);
    border-radius: 4px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }
}
</style>
