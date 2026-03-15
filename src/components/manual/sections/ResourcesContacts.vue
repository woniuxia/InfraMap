<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>联系人管理</h1>
    <p>联系人功能用于管理团队成员信息,并关联其负责的系统和服务。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>记录团队成员的基本信息(姓名、邮箱、电话等)</li>
        <li>关联负责的系统和服务</li>
        <li>在资源详情中快速查看负责人信息</li>
        <li>支持卡片视图和列表视图两种展示方式</li>
      </ul>
    </section>

    <section>
      <h2>如何添加联系人</h2>
      <ol class="steps-list">
        <li>进入"联系人"页面(左侧菜单 → 更多 → 联系人)</li>
        <li>点击右上角"新增"按钮</li>
        <li>填写联系人信息(姓名、邮箱、电话等)</li>
        <li>点击"保存"完成添加</li>
      </ol>
    </section>

    <section>
      <h2>关联系统和服务</h2>
      <p>在添加或编辑系统、服务时,可以选择负责人。选择后,该联系人会自动关联到对应的资源。</p>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          在联系人详情页,可以查看该成员负责的所有系统和服务,方便进行责任追踪。
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
