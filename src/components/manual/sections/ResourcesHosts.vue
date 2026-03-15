<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>服务器管理</h1>
    <p>服务器是基础设施的核心资源,InfraMap 支持管理物理机、虚拟机、容器等各类计算资源。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录服务器的基本信息(名称、IP、配置等)</li>
        <li>管理服务器的部署信息(操作系统、环境等)</li>
        <li>关联服务器上部署的服务和中间件</li>
        <li>查看服务器的依赖关系和使用情况</li>
      </ul>
    </section>

    <section>
      <h2>如何添加服务器</h2>
      <ol class="steps-list">
        <li>进入"服务器"页面</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写服务器信息(名称、IP、环境等)</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>字段说明</h2>
      <table class="field-table">
        <thead>
          <tr>
            <th>字段</th>
            <th>说明</th>
            <th>是否必填</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>名称</td>
            <td>服务器名称或主机名</td>
            <td>是</td>
          </tr>
          <tr>
            <td>环境</td>
            <td>生产/测试/开发</td>
            <td>是</td>
          </tr>
          <tr>
            <td>IP 地址</td>
            <td>服务器的 IP 地址</td>
            <td>否</td>
          </tr>
          <tr>
            <td>操作系统</td>
            <td>如 CentOS、Ubuntu 等</td>
            <td>否</td>
          </tr>
          <tr>
            <td>负责人</td>
            <td>服务器的运维负责人</td>
            <td>否</td>
          </tr>
          <tr>
            <td>备注</td>
            <td>其他说明信息</td>
            <td>否</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <h2>常见操作</h2>
      <div class="operation-list">
        <div class="operation-item">
          <h3>查看部署信息</h3>
          <p>点击服务器名称,在详情页查看该服务器上部署的所有服务和中间件</p>
        </div>
        <div class="operation-item">
          <h3>编辑服务器</h3>
          <p>点击操作列的"编辑"按钮,修改服务器信息</p>
        </div>
        <div class="operation-item">
          <h3>删除服务器</h3>
          <p>点击操作列的"删除"按钮。注意:删除前需先解除该服务器上的所有部署关系</p>
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

.steps-list {
  counter-reset: step-counter;

  li {
    position: relative;
    padding-left: 8px;
  }
}

.field-table {
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

.operation-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-top: 24px;
}

.operation-item {
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
</style>
