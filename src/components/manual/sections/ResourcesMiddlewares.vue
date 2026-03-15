<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>中间件管理</h1>
    <p>中间件包括数据库、缓存、消息队列等基础组件,是服务运行的重要依赖。</p>

    <section>
      <h2>支持的中间件类型</h2>
      <ul>
        <li>
          <strong>数据库:</strong>
          MySQL、PostgreSQL、MongoDB 等
        </li>
        <li>
          <strong>缓存:</strong>
          Redis、Memcached 等
        </li>
        <li>
          <strong>消息队列:</strong>
          Kafka、RabbitMQ、RocketMQ 等
        </li>
        <li>
          <strong>搜索引擎:</strong>
          Elasticsearch 等
        </li>
        <li>
          <strong>其他:</strong>
          Zookeeper、Etcd 等
        </li>
      </ul>
    </section>

    <section>
      <h2>如何添加中间件</h2>
      <ol class="steps-list">
        <li>进入"中间件"页面</li>
        <li>点击右上角"新增"按钮</li>
        <li>选择中间件类型</li>
        <li>填写中间件信息(名称、版本、连接地址等)</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>中间件在拓扑图中的表示</h2>
      <p>
        中间件节点在拓扑图中显示为橙色菱形,便于与服务节点区分。当服务依赖某个中间件时,会显示从服务到中间件的连接线。
      </p>
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

      strong {
        color: var(--im-text-primary);
      }
    }
  }
}
</style>
