<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>服务管理</h1>
    <p>服务是应用程序的运行实例,InfraMap 支持管理各类服务的信息、依赖关系和部署情况。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录服务的基本信息(名称、类型、端口等)</li>
        <li>管理服务的依赖关系(依赖哪些服务和中间件)</li>
        <li>记录服务的部署信息(部署在哪些服务器上)</li>
        <li>在拓扑图中查看服务的上下游依赖</li>
      </ul>
    </section>

    <section>
      <h2>服务类型</h2>
      <p>InfraMap 支持多种服务类型,每种类型在拓扑图中有不同的颜色标识:</p>
      <ul>
        <li>
          <strong>前端应用:</strong>
          Web 前端、移动端等
        </li>
        <li>
          <strong>后端服务:</strong>
          API 服务、微服务等
        </li>
        <li>
          <strong>网关:</strong>
          API 网关
        </li>
        <li>
          <strong>批处理任务:</strong>
          定时任务、数据处理任务等
        </li>
        <li>
          <strong>其他:</strong>
          其他类型的服务
        </li>
      </ul>
    </section>

    <section>
      <h2>如何添加服务</h2>
      <ol class="steps-list">
        <li>进入"服务"页面</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写服务信息(名称、类型、所属系统等)</li>
        <li>配置依赖关系(可选)</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>依赖关系管理</h2>
      <p>服务的依赖关系是拓扑图的核心数据。添加依赖关系后,拓扑图会自动显示服务间的连接。</p>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          依赖关系支持多种类型,如"调用"、"数据依赖"等。在拓扑图中,不同类型的依赖会用不同的线条样式表示。
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

      strong {
        color: var(--im-text-primary);
      }
    }
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
