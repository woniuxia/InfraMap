<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>系统管理</h1>
    <p>
      系统是业务逻辑的组织单元,一个系统通常包含多个服务。InfraMap
      支持管理系统的基本信息、负责人、技术栈等。
    </p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录系统的基本信息(名称、描述、负责人等)</li>
        <li>管理系统的技术栈和架构信息</li>
        <li>关联系统下的所有服务</li>
        <li>在拓扑图中按系统聚焦查看</li>
      </ul>
    </section>

    <section>
      <h2>如何添加系统</h2>
      <ol class="steps-list">
        <li>进入"系统"页面</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写系统信息(名称、环境、负责人等)</li>
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
            <td>系统名称</td>
            <td>是</td>
          </tr>
          <tr>
            <td>环境</td>
            <td>生产/测试/开发</td>
            <td>是</td>
          </tr>
          <tr>
            <td>负责人</td>
            <td>系统的业务或技术负责人</td>
            <td>否</td>
          </tr>
          <tr>
            <td>描述</td>
            <td>系统的功能描述</td>
            <td>否</td>
          </tr>
          <tr>
            <td>技术栈</td>
            <td>使用的主要技术(如 Java、Go、Python 等)</td>
            <td>否</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <h2>系统与服务的关系</h2>
      <p>
        在 InfraMap
        中,系统和服务是一对多的关系。一个系统可以包含多个服务,每个服务必须归属于一个系统。
      </p>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          在拓扑图的聚焦布局中,可以选择特定系统,只显示该系统下的服务及其依赖关系,方便进行系统级别的分析。
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
