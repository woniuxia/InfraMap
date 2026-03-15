<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>常见问题</h1>
    <p>这里汇总了使用 InfraMap 过程中的常见问题和解决方案。</p>

    <section>
      <h2>安装与启动</h2>
      <div class="faq-item">
        <h3>Q: Windows 提示"无法验证发布者"怎么办?</h3>
        <p>
          A: 这是因为应用没有数字签名。点击"更多信息" → "仍要运行"即可。InfraMap
          是开源软件,代码完全透明,可以放心使用。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 启动后界面显示空白怎么办?</h3>
        <p>
          A: 尝试以下方法:1) 重启应用;2) 清除缓存(删除存储目录下的 cache 文件夹);3) 重新安装应用。
        </p>
      </div>
    </section>

    <section>
      <h2>数据管理</h2>
      <div class="faq-item">
        <h3>Q: 如何备份我的数据?</h3>
        <p>
          A: 有两种方式:1) 在系统设置中配置自动备份;2)
          使用快照导出功能手动导出数据。建议两种方式结合使用。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 误删了数据怎么恢复?</h3>
        <p>
          A: InfraMap
          使用软删除机制,删除的数据不会立即物理删除。可以联系技术支持恢复数据,或从备份中恢复。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 可以同时管理多个环境吗?</h3>
        <p>
          A: 可以。InfraMap 支持生产、测试、开发三个环境,数据完全隔离。在左侧边栏顶部切换环境即可。
        </p>
      </div>
    </section>

    <section>
      <h2>拓扑图</h2>
      <div class="faq-item">
        <h3>Q: 拓扑图节点太多,看不清怎么办?</h3>
        <p>
          A: 建议:1) 使用聚焦布局,只显示特定系统的依赖;2) 使用筛选功能,隐藏部分节点类型;3)
          启用性能优化选项。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 拓扑图加载很慢怎么办?</h3>
        <p>
          A: 当节点数超过 200 时,建议:1) 使用聚焦布局代替力导向布局;2) 启用边缘简化选项;3)
          按环境分别管理,避免单个环境节点过多。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 如何表示服务间的不同依赖类型?</h3>
        <p>
          A:
          在添加依赖关系时,可以选择依赖类型(如"调用"、"数据依赖"等)。不同类型在拓扑图中会用不同的线条样式表示。
        </p>
      </div>
    </section>

    <section>
      <h2>批量导入</h2>
      <div class="faq-item">
        <h3>Q: 批量导入失败怎么办?</h3>
        <p>
          A: 1) 在任务中心查看详细错误日志;2) 检查 Excel 文件格式是否符合模板要求;3)
          确保必填字段都已填写;4) 检查引用的资源是否存在。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 可以导入哪些格式的文件?</h3>
        <p>
          A: 目前支持 Excel (.xlsx) 和 CSV (.csv) 格式。建议使用 Excel 格式,支持更丰富的数据类型。
        </p>
      </div>
    </section>

    <section>
      <h2>性能优化</h2>
      <div class="faq-item">
        <h3>Q: 数据量大时应用变慢怎么办?</h3>
        <p>
          A: 1) 定期运行数据健康检查,清理无用数据;2) 按环境分别管理,避免单个环境数据过多;3)
          使用聚焦布局代替全局视图;4) 升级到更高配置的电脑。
        </p>
      </div>
    </section>

    <section>
      <h2>其他问题</h2>
      <div class="faq-item">
        <h3>Q: InfraMap 是否支持多人协作?</h3>
        <p>
          A: InfraMap
          是单机应用,不支持实时多人协作。但可以通过快照导入导出功能在团队成员间共享数据。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 数据存储在哪里?是否安全?</h3>
        <p>
          A: 数据存储在本地 SQLite 数据库中,完全离线,不会上传到任何服务器。数据安全由您自己掌控。
        </p>
      </div>
      <div class="faq-item">
        <h3>Q: 如何获取技术支持?</h3>
        <p>
          A: 可以通过以下方式获取帮助:1) 查看本使用手册;2) 访问项目 GitHub 仓库提交 Issue;3)
          联系开发团队。
        </p>
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
}

.faq-item {
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);
  margin-bottom: 16px;

  h3 {
    margin: 0 0 12px;
    color: var(--im-accent);
  }

  p {
    margin: 0;
    font-size: 14px;
  }
}
</style>
