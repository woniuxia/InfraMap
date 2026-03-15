<script setup lang="ts"></script>

<template>
  <div class="manual-section">
    <h1>数据健康</h1>
    <p>数据健康功能自动检测数据的完整性和一致性问题,帮助您维护高质量的数据。</p>

    <section>
      <h2>检查项目</h2>
      <div class="check-list">
        <div class="check-item">
          <el-icon :size="24" color="var(--im-warning)">
            <Warning />
          </el-icon>
          <div>
            <h3>孤立节点</h3>
            <p>检测没有任何依赖关系的服务或中间件节点</p>
          </div>
        </div>
        <div class="check-item">
          <el-icon :size="24" color="var(--im-danger)">
            <CircleClose />
          </el-icon>
          <div>
            <h3>循环依赖</h3>
            <p>检测服务间的循环依赖关系,可能导致系统不稳定</p>
          </div>
        </div>
        <div class="check-item">
          <el-icon :size="24" color="var(--im-info)">
            <InfoFilled />
          </el-icon>
          <div>
            <h3>缺失信息</h3>
            <p>检测资源的必要信息是否完整(如负责人、描述等)</p>
          </div>
        </div>
        <div class="check-item">
          <el-icon :size="24" color="var(--im-warning)">
            <Link />
          </el-icon>
          <div>
            <h3>引用完整性</h3>
            <p>检测依赖关系中引用的资源是否存在</p>
          </div>
        </div>
      </div>
    </section>

    <section>
      <h2>如何使用数据健康检查</h2>
      <ol class="steps-list">
        <li>进入"数据健康"页面(左侧菜单 → 更多 → 数据健康)</li>
        <li>点击"开始检查"按钮</li>
        <li>等待检查完成,查看检查结果</li>
        <li>点击问题项查看详细信息</li>
        <li>根据提示修复问题</li>
      </ol>
    </section>

    <section>
      <h2>问题修复建议</h2>
      <ul>
        <li>
          <strong>孤立节点:</strong>
          检查是否遗漏了依赖关系,或者该节点是否应该删除
        </li>
        <li>
          <strong>循环依赖:</strong>
          重新设计服务架构,打破循环依赖链
        </li>
        <li>
          <strong>缺失信息:</strong>
          补充完整的资源信息,便于后续管理
        </li>
        <li>
          <strong>引用完整性:</strong>
          删除无效的依赖关系,或补充缺失的资源
        </li>
      </ul>
    </section>

    <section>
      <div class="tip-box">
        <el-icon :size="20" color="var(--im-success)">
          <CircleCheck />
        </el-icon>
        <div>
          <strong>建议:</strong>
          定期运行数据健康检查(如每周一次),及时发现和修复数据质量问题,保持数据的准确性和完整性。
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
    margin-bottom: 4px;
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

.check-list {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-top: 24px;
}

.check-item {
  display: flex;
  gap: 12px;
  padding: 20px;
  background: var(--im-surface-0);
  border: 1px solid var(--im-border);
  border-radius: var(--im-radius-md);

  h3 {
    margin: 0 0 4px;
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
  background: rgba(65, 197, 138, 0.1);
  border: 1px solid rgba(65, 197, 138, 0.3);
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
