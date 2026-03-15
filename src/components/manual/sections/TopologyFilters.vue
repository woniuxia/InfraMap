<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>拓扑图筛选与搜索</h1>
    <p>当拓扑图包含大量节点时,筛选和搜索功能可以帮助您快速定位目标节点,提高工作效率。</p>

    <section>
      <h2>环境切换</h2>
      <div class="filter-card">
        <div class="filter-icon">
          <el-icon :size="32" color="var(--im-accent)">
            <Switch />
          </el-icon>
        </div>
        <div class="filter-content">
          <h3>生产/测试/开发环境独立视图</h3>
          <p>InfraMap 支持多环境管理,不同环境的拓扑图相互独立,避免混淆。</p>
          <div class="filter-detail">
            <h4>切换方式</h4>
            <p>
              在左侧边栏顶部的环境选择器中,点击切换到目标环境。切换后,拓扑图会自动加载该环境的数据。
            </p>
            <h4>环境标识</h4>
            <ul>
              <li>
                <span class="env-badge prod">生产</span>
                红色,代表线上正式环境
              </li>
              <li>
                <span class="env-badge test">测试</span>
                橙色,代表预发布验证环境
              </li>
              <li>
                <span class="env-badge dev">开发</span>
                蓝色,代表开发调试环境
              </li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>节点类型筛选</h2>
      <div class="filter-card">
        <div class="filter-icon">
          <el-icon :size="32" color="var(--im-success)">
            <Filter />
          </el-icon>
        </div>
        <div class="filter-content">
          <h3>按服务/中间件/网关筛选</h3>
          <p>在拓扑图工具栏中,可以选择只显示特定类型的节点。</p>
          <div class="filter-detail">
            <h4>筛选选项</h4>
            <table class="filter-table">
              <thead>
                <tr>
                  <th>节点类型</th>
                  <th>说明</th>
                  <th>适用场景</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><strong>服务</strong></td>
                  <td>应用服务节点</td>
                  <td>分析服务间调用关系</td>
                </tr>
                <tr>
                  <td><strong>中间件</strong></td>
                  <td>数据库、缓存、消息队列等</td>
                  <td>排查中间件依赖问题</td>
                </tr>
                <tr>
                  <td><strong>网关</strong></td>
                  <td>Nginx 网关配置</td>
                  <td>分析流量入口和路由</td>
                </tr>
                <tr>
                  <td><strong>全部</strong></td>
                  <td>显示所有类型节点</td>
                  <td>全局视图</td>
                </tr>
              </tbody>
            </table>
            <h4>组合筛选</h4>
            <p>可以同时选择多个类型,例如只显示"服务 + 中间件",隐藏网关节点。</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>关键字搜索</h2>
      <div class="filter-card">
        <div class="filter-icon">
          <el-icon :size="32" color="var(--im-info)">
            <Search />
          </el-icon>
        </div>
        <div class="filter-content">
          <h3>快速定位节点</h3>
          <p>使用搜索功能可以快速找到目标节点,并自动聚焦到该节点。</p>
          <div class="filter-detail">
            <h4>搜索方式</h4>
            <ul>
              <li>点击工具栏的搜索图标</li>
              <li>
                使用快捷键
                <kbd>Ctrl</kbd>
                +
                <kbd>F</kbd>
              </li>
            </ul>
            <h4>搜索范围</h4>
            <ul>
              <li>节点名称(支持模糊匹配)</li>
              <li>节点描述</li>
              <li>节点标签</li>
            </ul>
            <h4>搜索技巧</h4>
            <ul>
              <li>
                <strong>模糊搜索:</strong>
                输入部分关键字即可匹配,如"user"可匹配"user-service"
              </li>
              <li>
                <strong>大小写不敏感:</strong>
                搜索时忽略大小写
              </li>
              <li>
                <strong>多结果导航:</strong>
                如果有多个匹配结果,可以使用上下箭头切换
              </li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>高级筛选</h2>
      <div class="filter-card">
        <div class="filter-icon">
          <el-icon :size="32" color="var(--im-warning)">
            <Operation />
          </el-icon>
        </div>
        <div class="filter-content">
          <h3>组合条件筛选</h3>
          <p>在侧边栏的筛选面板中,可以设置更复杂的筛选条件。</p>
          <div class="filter-detail">
            <h4>可用筛选条件</h4>
            <ul>
              <li>
                <strong>所属系统:</strong>
                按系统分组筛选
              </li>
              <li>
                <strong>部署主机:</strong>
                按服务器筛选
              </li>
              <li>
                <strong>健康状态:</strong>
                只显示异常/警告节点
              </li>
              <li>
                <strong>标签:</strong>
                按自定义标签筛选
              </li>
            </ul>
            <h4>筛选逻辑</h4>
            <p>多个条件之间是"与"关系,即同时满足所有条件的节点才会显示。</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>性能优化开关</h2>
      <div class="filter-card">
        <div class="filter-icon">
          <el-icon :size="32" color="var(--im-danger)">
            <Setting />
          </el-icon>
        </div>
        <div class="filter-content">
          <h3>大规模图谱优化</h3>
          <p>当拓扑图包含大量节点时,可以启用性能优化选项。</p>
          <div class="filter-detail">
            <h4>优化选项</h4>
            <ul>
              <li>
                <strong>边缘简化:</strong>
                合并重复的依赖边,减少渲染负担
              </li>
              <li>
                <strong>节点聚合:</strong>
                将同类型节点聚合显示
              </li>
              <li>
                <strong>延迟加载:</strong>
                只加载可视区域的节点
              </li>
            </ul>
            <h4>启用方式</h4>
            <p>在拓扑图控制栏中,点击"性能优化"开关。建议在节点数超过 100 时启用。</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>筛选功能汇总</h2>
      <table class="summary-table">
        <thead>
          <tr>
            <th>功能</th>
            <th>位置</th>
            <th>快捷键</th>
            <th>适用场景</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>环境切换</td>
            <td>侧边栏顶部</td>
            <td>-</td>
            <td>切换生产/测试/开发环境</td>
          </tr>
          <tr>
            <td>节点类型筛选</td>
            <td>拓扑图工具栏</td>
            <td>-</td>
            <td>只显示特定类型节点</td>
          </tr>
          <tr>
            <td>关键字搜索</td>
            <td>拓扑图工具栏</td>
            <td><kbd>Ctrl+F</kbd></td>
            <td>快速定位节点</td>
          </tr>
          <tr>
            <td>高级筛选</td>
            <td>侧边栏筛选面板</td>
            <td>-</td>
            <td>组合条件筛选</td>
          </tr>
          <tr>
            <td>性能优化</td>
            <td>拓扑图控制栏</td>
            <td>-</td>
            <td>大规模图谱优化</td>
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
          <strong>使用建议:</strong>
          在大规模拓扑图中,建议先使用"节点类型筛选"缩小范围,再使用"关键字搜索"精确定位。这样可以获得最佳的性能和体验。
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
    margin: 0 0 8px;
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

.filter-card {
  display: flex;
  gap: 20px;
  padding: 24px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  margin-top: 24px;
}

.filter-icon {
  flex-shrink: 0;
}

.filter-content {
  flex: 1;
}

.filter-detail {
  margin-top: 16px;

  ul {
    margin: 0 0 16px;
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

.env-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;

  &.prod {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }

  &.test {
    background: rgba(242, 182, 69, 0.15);
    color: #f2b645;
  }

  &.dev {
    background: rgba(92, 163, 255, 0.15);
    color: #5ca3ff;
  }
}

.filter-table,
.summary-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 16px;

  th,
  td {
    padding: 12px 16px;
    text-align: left;
    border-bottom: 1px solid var(--im-border);
  }

  th {
    font-weight: 600;
    color: var(--im-text-primary);
    background: var(--im-surface-1);
  }

  td {
    color: var(--im-text-secondary);
  }

  tbody tr:hover {
    background: var(--im-surface-1);
  }
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
