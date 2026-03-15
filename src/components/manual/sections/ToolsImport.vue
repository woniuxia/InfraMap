<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>批量录入</h1>
    <p>批量录入功能支持通过 Excel 或 CSV 文件快速导入大量数据,大幅提升数据录入效率。</p>

    <section>
      <h2>支持的资源类型</h2>
      <ul>
        <li>服务器</li>
        <li>系统</li>
        <li>服务</li>
        <li>中间件</li>
        <li>网关配置</li>
        <li>联系人</li>
        <li>依赖关系</li>
      </ul>
    </section>

    <section>
      <h2>如何使用批量录入</h2>
      <ol class="steps-list">
        <li>进入"批量录入"页面(左侧菜单 → 更多 → 批量录入)</li>
        <li>选择要导入的资源类型</li>
        <li>下载对应的 Excel 模板</li>
        <li>按照模板格式填写数据</li>
        <li>上传填写好的文件</li>
        <li>系统会自动校验数据并显示校验结果</li>
        <li>确认无误后,点击"开始导入"</li>
      </ol>
    </section>

    <section>
      <h2>数据校验规则</h2>
      <p>系统会对上传的数据进行以下校验:</p>
      <ul>
        <li>
          <strong>必填字段检查:</strong>
          确保必填字段不为空
        </li>
        <li>
          <strong>格式校验:</strong>
          检查 IP 地址、邮箱等字段格式
        </li>
        <li>
          <strong>唯一性校验:</strong>
          检查名称等唯一字段是否重复
        </li>
        <li>
          <strong>引用完整性:</strong>
          检查关联的资源是否存在
        </li>
      </ul>
    </section>

    <section>
      <h2>查看导入结果</h2>
      <p>
        导入任务会在后台异步执行,可以在"任务中心"查看执行进度和结果。如果导入失败,可以下载错误日志查看详细原因。
      </p>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-info)">
          <InfoFilled />
        </el-icon>
        <div>
          <strong>提示:</strong>
          建议先导入基础资源(如服务器、系统),再导入依赖这些资源的数据(如服务、依赖关系),避免引用完整性错误。
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
