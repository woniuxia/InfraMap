<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>备份策略</h1>
    <p>InfraMap 支持自动备份功能,定期备份数据库,防止数据丢失。</p>

    <section>
      <h2>自动备份配置</h2>
      <p>在系统设置中,可以配置自动备份的频率和保留策略:</p>
      <ul>
        <li>
          <strong>备份频率:</strong>
          每天/每周/每月
        </li>
        <li>
          <strong>备份时间:</strong>
          指定备份的具体时间点
        </li>
        <li>
          <strong>保留数量:</strong>
          保留最近 N 个备份文件
        </li>
        <li>
          <strong>备份位置:</strong>
          默认在存储目录的 backups 子目录
        </li>
      </ul>
    </section>

    <section>
      <h2>如何配置自动备份</h2>
      <ol class="steps-list">
        <li>进入"系统设置"页面</li>
        <li>切换到"备份策略"选项卡</li>
        <li>启用"自动备份"开关</li>
        <li>设置备份频率和保留数量</li>
        <li>点击"保存"应用配置</li>
      </ol>
    </section>

    <section>
      <h2>手动备份</h2>
      <p>除了自动备份,您也可以随时手动创建备份:</p>
      <ol class="steps-list">
        <li>在"备份策略"页面,点击"立即备份"按钮</li>
        <li>系统会创建当前数据库的完整备份</li>
        <li>备份文件会保存在备份目录中</li>
      </ol>
    </section>

    <section>
      <h2>恢复备份</h2>
      <p>如果需要恢复到之前的备份:</p>
      <ol class="steps-list">
        <li>在备份列表中找到要恢复的备份</li>
        <li>点击"恢复"按钮</li>
        <li>系统会提示确认操作(恢复会覆盖当前数据)</li>
        <li>确认后,系统会自动恢复数据并重启应用</li>
      </ol>
      <div class="warning-box">
        <el-icon :size="20" color="var(--im-danger)">
          <Warning />
        </el-icon>
        <div>
          <strong>警告:</strong>
          恢复备份会覆盖当前所有数据,操作前请确认。建议先手动备份当前数据,以防需要回退。
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
