<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>快照导入导出</h1>
    <p>快照功能用于数据的导入导出,方便数据迁移、版本管理和团队协作。</p>

    <section>
      <h2>什么是快照</h2>
      <p>
        快照是某个时间点的完整数据副本,包含所有资源、依赖关系和配置信息。快照文件是一个 JSON
        格式的文件,可以在不同的 InfraMap 实例间共享。
      </p>
    </section>

    <section>
      <h2>导出快照</h2>
      <ol class="steps-list">
        <li>进入"系统设置"页面</li>
        <li>切换到"快照导入导出"选项卡</li>
        <li>点击"导出快照"按钮</li>
        <li>选择要导出的环境(生产/测试/开发)</li>
        <li>选择保存位置</li>
        <li>系统会生成快照文件(格式: inframap-snapshot-YYYYMMDD-HHMMSS.json)</li>
      </ol>
    </section>

    <section>
      <h2>导入快照</h2>
      <ol class="steps-list">
        <li>在"快照导入导出"页面,点击"导入快照"按钮</li>
        <li>选择要导入的快照文件</li>
        <li>系统会显示快照的基本信息(创建时间、资源数量等)</li>
        <li>
          选择导入模式:
          <ul>
            <li>
              <strong>覆盖模式:</strong>
              清空当前环境数据,完全替换为快照数据
            </li>
            <li>
              <strong>合并模式:</strong>
              保留现有数据,只添加快照中的新数据
            </li>
          </ul>
        </li>
        <li>确认后开始导入</li>
      </ol>
    </section>

    <section>
      <h2>使用场景</h2>
      <div class="scenario-list">
        <div class="scenario-item">
          <h3>数据迁移</h3>
          <p>从旧电脑迁移到新电脑,或在多台设备间同步数据</p>
        </div>
        <div class="scenario-item">
          <h3>版本管理</h3>
          <p>定期导出快照,保留不同时期的数据版本</p>
        </div>
        <div class="scenario-item">
          <h3>团队协作</h3>
          <p>团队成员间共享基础设施数据</p>
        </div>
        <div class="scenario-item">
          <h3>环境复制</h3>
          <p>将生产环境数据复制到测试环境</p>
        </div>
      </div>
    </section>

    <section>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          快照文件是纯文本 JSON 格式,可以用文本编辑器打开查看或手动编辑。但建议通过 InfraMap
          界面进行操作,避免格式错误。
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

      strong {
        color: var(--im-text-primary);
      }
    }
  }
}

.scenario-list {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.scenario-item {
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
