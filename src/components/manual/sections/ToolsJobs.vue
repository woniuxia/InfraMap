<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>任务中心</h1>
    <p>任务中心用于查看和管理批量导入任务的执行状态,支持任务重试和错误日志查看。</p>

    <section>
      <h2>功能概述</h2>
      <ul>
        <li>查看所有批量导入任务的执行状态</li>
        <li>查看任务的详细执行日志</li>
        <li>下载失败任务的错误报告</li>
        <li>重试失败的任务</li>
      </ul>
    </section>

    <section>
      <h2>任务状态说明</h2>
      <table class="status-table">
        <thead>
          <tr>
            <th>状态</th>
            <th>说明</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><span class="status-badge pending">待执行</span></td>
            <td>任务已创建,等待执行</td>
          </tr>
          <tr>
            <td><span class="status-badge running">执行中</span></td>
            <td>任务正在执行</td>
          </tr>
          <tr>
            <td><span class="status-badge success">成功</span></td>
            <td>任务执行成功</td>
          </tr>
          <tr>
            <td><span class="status-badge failed">失败</span></td>
            <td>任务执行失败,可查看错误日志</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <h2>如何查看任务详情</h2>
      <ol class="steps-list">
        <li>进入"任务中心"页面(左侧菜单 → 更多 → 任务中心)</li>
        <li>在任务列表中找到目标任务</li>
        <li>点击任务名称或"查看详情"按钮</li>
        <li>在详情页查看执行日志和结果</li>
      </ol>
    </section>

    <section>
      <h2>处理失败任务</h2>
      <p>如果任务执行失败,可以:</p>
      <ul>
        <li>点击"下载错误日志"查看详细错误信息</li>
        <li>根据错误信息修正数据文件</li>
        <li>点击"重试"按钮重新执行任务</li>
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
    }
  }
}

.status-table {
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

.status-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;

  &.pending {
    background: rgba(92, 163, 255, 0.15);
    color: #5ca3ff;
  }

  &.running {
    background: rgba(242, 182, 69, 0.15);
    color: #f2b645;
  }

  &.success {
    background: rgba(65, 197, 138, 0.15);
    color: #41c58a;
  }

  &.failed {
    background: rgba(239, 107, 115, 0.15);
    color: #ef6b73;
  }
}
</style>
