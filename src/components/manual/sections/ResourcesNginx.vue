<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>网关管理</h1>
    <p>网关是流量入口,InfraMap 支持管理 Nginx 网关配置,记录路由规则和上游服务。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录 Nginx 配置的基本信息</li>
        <li>管理路由规则和上游服务映射</li>
        <li>在拓扑图中查看流量入口和路由关系</li>
      </ul>
    </section>

    <section>
      <h2>如何添加网关配置</h2>
      <ol class="steps-list">
        <li>进入"网关"页面</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写配置信息(名称、监听端口、路由规则等)</li>
        <li>配置上游服务</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>网关在拓扑图中的表示</h2>
      <p>网关节点在拓扑图中显示为绿色六边形,从网关到上游服务会显示连接线,表示流量路由关系。</p>
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

  p {
    line-height: 1.8;
    color: var(--im-text-secondary);
    margin-bottom: 16px;
  }

  section {
    margin-bottom: 48px;
  }

  ul,
  ol {
    margin: 16px 0;
    padding-left: 24px;
    color: var(--im-text-secondary);

    li {
      margin-bottom: 8px;
      line-height: 1.6;
    }
  }
}
</style>
