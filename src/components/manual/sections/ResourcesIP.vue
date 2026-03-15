<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>IP地址管理</h1>
    <p>IP地址管理功能用于记录和管理服务器的 IP 地址,支持一台服务器绑定多个 IP。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录 IP 地址的基本信息(IP、类型、用途等)</li>
        <li>管理 IP 与服务器的绑定关系</li>
        <li>支持内网 IP 和公网 IP 分类管理</li>
        <li>查看 IP 的使用情况</li>
      </ul>
    </section>

    <section>
      <h2>如何添加 IP 地址</h2>
      <ol class="steps-list">
        <li>进入"IP地址"页面(左侧菜单 → 更多 → IP地址)</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写 IP 信息(IP 地址、类型、环境等)</li>
        <li>选择绑定的服务器(可选)</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>IP 类型</h2>
      <ul>
        <li>
          <strong>内网 IP:</strong>
          用于内部通信的私有 IP
        </li>
        <li>
          <strong>公网 IP:</strong>
          可从互联网访问的公共 IP
        </li>
        <li>
          <strong>VIP:</strong>
          虚拟 IP,用于负载均衡等场景
        </li>
      </ul>
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
