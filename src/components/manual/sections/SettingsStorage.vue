<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>存储路径</h1>
    <p>InfraMap 的所有数据存储在本地,您可以自定义数据库和备份文件的存储位置。</p>

    <section>
      <h2>默认存储位置</h2>
      <p>InfraMap 默认将数据存储在操作系统的标准应用数据目录:</p>
      <ul>
        <li>
          <strong>Windows:</strong>
          <code>C:\Users\[用户名]\AppData\Roaming\InfraMap</code>
        </li>
        <li>
          <strong>macOS:</strong>
          <code>~/Library/Application Support/InfraMap</code>
        </li>
        <li>
          <strong>Linux:</strong>
          <code>~/.local/share/InfraMap</code>
        </li>
      </ul>
    </section>

    <section>
      <h2>如何修改存储路径</h2>
      <ol class="steps-list">
        <li>进入"系统设置"页面(左侧菜单底部 → 系统设置)</li>
        <li>在"存储路径"选项卡中,点击"修改路径"</li>
        <li>选择新的存储目录</li>
        <li>系统会提示是否迁移现有数据</li>
        <li>确认后,系统会自动迁移数据到新位置</li>
      </ol>
    </section>

    <section>
      <h2>存储内容说明</h2>
      <table class="storage-table">
        <thead>
          <tr>
            <th>文件/目录</th>
            <th>说明</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><code>inframap.db</code></td>
            <td>SQLite 数据库文件,存储所有资源数据</td>
          </tr>
          <tr>
            <td><code>backups/</code></td>
            <td>自动备份目录,存储定期备份的数据库文件</td>
          </tr>
          <tr>
            <td><code>snapshots/</code></td>
            <td>快照目录,存储手动导出的数据快照</td>
          </tr>
          <tr>
            <td><code>logs/</code></td>
            <td>日志目录,存储应用运行日志</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <div class="warning-box">
        <el-icon :size="20" color="var(--im-danger)">
          <Warning />
        </el-icon>
        <div>
          <strong>注意:</strong>
          修改存储路径前,建议先备份数据。迁移过程中请勿关闭应用,以免数据损坏。
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

  code {
    padding: 2px 6px;
    background: var(--im-surface-1);
    border: 1px solid var(--im-border);
    border-radius: 4px;
    font-family: var(--im-font-mono);
    font-size: 13px;
    color: var(--im-accent);
  }
}

.storage-table {
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

.warning-box {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: rgba(239, 107, 115, 0.1);
  border: 1px solid rgba(239, 107, 115, 0.3);
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
